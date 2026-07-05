//! Real audio-device capture/playback (ARC-05), gated behind the `audio-device`
//! Cargo feature so it — and its ALSA C dependency — are absent from the default
//! cross-build. Built natively on the Pi to bridge the rig's USB audio codec.
//!
//! cpal's `Stream` is `!Send`, so each device runs on a dedicated thread that
//! builds the stream, starts it, and parks — the stream stays alive for the
//! process lifetime. Producers/consumers exchange samples through shared ring
//! buffers (`Send + Sync`), which is all the rest of the app ever touches.

// Sample-format conversions are intentional and bounded to [-1, 1].
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat};

use crate::audio::AudioSink;
use crate::spectrum::SampleSource;

/// Cap on buffered samples per ring (~1 s at 48 kHz) to bound latency/memory.
const MAX_BUFFERED: usize = 48_000;

type Ring = Arc<Mutex<VecDeque<f32>>>;

fn lock(ring: &Ring) -> std::sync::MutexGuard<'_, VecDeque<f32>> {
    ring.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Pick an input/output device whose name contains `want` (case-insensitive), or
/// the host default when `want` is `None`/unmatched.
fn select_device(
    devices: impl Iterator<Item = Device>,
    want: Option<&str>,
    default: Option<Device>,
) -> Option<Device> {
    if let Some(needle) = want.map(str::to_ascii_lowercase) {
        for device in devices {
            if let Ok(name) = device.name() {
                if name.to_ascii_lowercase().contains(&needle) {
                    return Some(device);
                }
            }
        }
    }
    default
}

/// A capture tap: a `SampleSource` view over one of the capture ring buffers.
pub struct CaptureTap {
    ring: Ring,
}

impl SampleSource for CaptureTap {
    fn next_block(&self, size: usize) -> Vec<f32> {
        let mut ring = lock(&self.ring);
        let mut out = Vec::with_capacity(size);
        for _ in 0..size {
            out.push(ring.pop_front().unwrap_or(0.0)); // silence on underrun
        }
        out
    }
}

/// Owns a live input stream on a dedicated thread and fans captured mono samples
/// into N ring buffers (one tap per consumer — e.g. spectrum + audio).
pub struct CpalCapture {
    taps: Vec<Ring>,
    _thread: thread::JoinHandle<()>,
}

impl CpalCapture {
    /// Open the input device (name-matched or default) and start capturing into
    /// `tap_count` independent ring buffers.
    ///
    /// # Errors
    /// Returns a message if no input device/config is available or the stream
    /// cannot be built.
    pub fn new(device_name: Option<String>, tap_count: usize) -> Result<Self, String> {
        let taps: Vec<Ring> = (0..tap_count.max(1))
            .map(|_| Arc::new(Mutex::new(VecDeque::new())))
            .collect();
        let taps_for_thread = taps.clone();
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();

        let handle = thread::spawn(move || {
            let build = || -> Result<cpal::Stream, String> {
                let host = cpal::default_host();
                let device = select_device(
                    host.input_devices().map_err(|e| e.to_string())?,
                    device_name.as_deref(),
                    host.default_input_device(),
                )
                .ok_or_else(|| "no input device".to_string())?;
                let supported = device.default_input_config().map_err(|e| e.to_string())?;
                let channels = supported.channels() as usize;
                let format = supported.sample_format();
                let config: cpal::StreamConfig = supported.into();
                let err = |e| eprintln!("audio capture error: {e}");
                let taps = taps_for_thread;
                let push = move |mono: f32| {
                    for ring in &taps {
                        let mut r = lock(ring);
                        r.push_back(mono);
                        while r.len() > MAX_BUFFERED {
                            r.pop_front();
                        }
                    }
                };
                let stream = match format {
                    SampleFormat::F32 => device.build_input_stream(
                        &config,
                        move |data: &[f32], _: &_| {
                            for frame in data.chunks(channels) {
                                push(frame.first().copied().unwrap_or(0.0));
                            }
                        },
                        err,
                        None,
                    ),
                    SampleFormat::I16 => device.build_input_stream(
                        &config,
                        move |data: &[i16], _: &_| {
                            for frame in data.chunks(channels) {
                                push(f32::from(frame.first().copied().unwrap_or(0)) / 32768.0);
                            }
                        },
                        err,
                        None,
                    ),
                    other => return Err(format!("unsupported input sample format: {other:?}")),
                }
                .map_err(|e| e.to_string())?;
                stream.play().map_err(|e| e.to_string())?;
                Ok(stream)
            };
            match build() {
                Ok(stream) => {
                    tx.send(Ok(())).ok();
                    loop {
                        thread::park(); // keep `stream` alive for the process lifetime
                        let _keep = &stream;
                    }
                }
                Err(e) => {
                    tx.send(Err(e)).ok();
                }
            }
        });

        rx.recv().map_err(|_| "capture thread died".to_string())??;
        Ok(Self {
            taps,
            _thread: handle,
        })
    }

    /// A `SampleSource` view over tap `index` (clamped to the available taps).
    #[must_use]
    pub fn tap(&self, index: usize) -> CaptureTap {
        let ring = self.taps[index.min(self.taps.len() - 1)].clone();
        CaptureTap { ring }
    }
}

/// Owns a live output stream on a dedicated thread and plays samples pushed via
/// [`AudioSink::accept`].
pub struct CpalSink {
    ring: Ring,
    _thread: thread::JoinHandle<()>,
}

impl CpalSink {
    /// Open the output device (name-matched or default) and start playback.
    ///
    /// # Errors
    /// Returns a message if no output device/config is available or the stream
    /// cannot be built.
    pub fn new(device_name: Option<String>) -> Result<Self, String> {
        let ring: Ring = Arc::new(Mutex::new(VecDeque::new()));
        let ring_for_thread = ring.clone();
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();

        let handle = thread::spawn(move || {
            let build = || -> Result<cpal::Stream, String> {
                let host = cpal::default_host();
                let device = select_device(
                    host.output_devices().map_err(|e| e.to_string())?,
                    device_name.as_deref(),
                    host.default_output_device(),
                )
                .ok_or_else(|| "no output device".to_string())?;
                let supported = device.default_output_config().map_err(|e| e.to_string())?;
                let channels = supported.channels() as usize;
                let format = supported.sample_format();
                let config: cpal::StreamConfig = supported.into();
                let err = |e| eprintln!("audio playback error: {e}");
                let ring = ring_for_thread;
                let next = move || lock(&ring).pop_front().unwrap_or(0.0); // silence on underrun
                let stream = match format {
                    SampleFormat::F32 => device.build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &_| {
                            for frame in data.chunks_mut(channels) {
                                let s = next();
                                for slot in frame {
                                    *slot = s;
                                }
                            }
                        },
                        err,
                        None,
                    ),
                    SampleFormat::I16 => device.build_output_stream(
                        &config,
                        move |data: &mut [i16], _: &_| {
                            for frame in data.chunks_mut(channels) {
                                let s = (next().clamp(-1.0, 1.0) * 32767.0) as i16;
                                for slot in frame {
                                    *slot = s;
                                }
                            }
                        },
                        err,
                        None,
                    ),
                    other => return Err(format!("unsupported output sample format: {other:?}")),
                }
                .map_err(|e| e.to_string())?;
                stream.play().map_err(|e| e.to_string())?;
                Ok(stream)
            };
            match build() {
                Ok(stream) => {
                    tx.send(Ok(())).ok();
                    loop {
                        thread::park();
                        let _keep = &stream;
                    }
                }
                Err(e) => {
                    tx.send(Err(e)).ok();
                }
            }
        });

        rx.recv()
            .map_err(|_| "playback thread died".to_string())??;
        Ok(Self {
            ring,
            _thread: handle,
        })
    }
}

impl AudioSink for CpalSink {
    fn accept(&self, pcm: &[i16]) {
        let mut ring = lock(&self.ring);
        for &s in pcm {
            ring.push_back(f32::from(s) / 32768.0);
            if ring.len() > MAX_BUFFERED {
                ring.pop_front();
            }
        }
    }
}
