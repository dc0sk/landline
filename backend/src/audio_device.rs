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

/// Pick a supported config range that covers `sample_rate_hz` and pin it to
/// exactly that rate.
///
/// Split out from the stream builders (which need real hardware) so the
/// range-matching itself is unit-testable: this is the logic that decides
/// whether the pipeline runs at the rate everything else assumes.
fn supported_at_rate<R: SupportedRange>(
    mut ranges: impl Iterator<Item = R>,
    sample_rate_hz: u32,
) -> Option<R::Config> {
    let target = cpal::SampleRate(sample_rate_hz);
    ranges
        .find(|range| range.min_rate() <= target && target <= range.max_rate())
        .map(|range| range.pin_to(target))
}

/// The shape of a cpal supported-config range, so [`supported_at_rate`] can be
/// tested against a stand-in instead of a sound card.
trait SupportedRange {
    type Config;
    fn min_rate(&self) -> cpal::SampleRate;
    fn max_rate(&self) -> cpal::SampleRate;
    fn pin_to(self, rate: cpal::SampleRate) -> Self::Config;
}

impl SupportedRange for cpal::SupportedStreamConfigRange {
    type Config = cpal::SupportedStreamConfig;
    fn min_rate(&self) -> cpal::SampleRate {
        self.min_sample_rate()
    }
    fn max_rate(&self) -> cpal::SampleRate {
        self.max_sample_rate()
    }
    fn pin_to(self, rate: cpal::SampleRate) -> Self::Config {
        self.with_sample_rate(rate)
    }
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
    /// `sample_rate_hz` is requested explicitly rather than accepting the
    /// device default: ALSA's default for a device whose range spans it is
    /// commonly 44.1 kHz, and silently capturing at a rate the rest of the
    /// pipeline does not know about pitch-shifts the audio and mislabels the
    /// spectrum's frequency axis. If the device cannot do the configured rate
    /// this fails loudly rather than running at the wrong one.
    ///
    /// # Errors
    /// Returns a message if no input device is available, the device cannot
    /// capture at `sample_rate_hz`, or the stream cannot be built.
    pub fn new(
        device_name: Option<String>,
        tap_count: usize,
        sample_rate_hz: u32,
    ) -> Result<Self, String> {
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
                let supported = supported_at_rate(
                    device
                        .supported_input_configs()
                        .map_err(|e| e.to_string())?,
                    sample_rate_hz,
                )
                .ok_or_else(|| {
                    format!("input device does not support {sample_rate_hz} Hz capture")
                })?;
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
    /// `sample_rate_hz` is requested explicitly — see [`CpalCapture::new`] for
    /// why the device default is not good enough.
    ///
    /// # Errors
    /// Returns a message if no output device is available, the device cannot
    /// play at `sample_rate_hz`, or the stream cannot be built.
    pub fn new(device_name: Option<String>, sample_rate_hz: u32) -> Result<Self, String> {
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
                let supported = supported_at_rate(
                    device
                        .supported_output_configs()
                        .map_err(|e| e.to_string())?,
                    sample_rate_hz,
                )
                .ok_or_else(|| {
                    format!("output device does not support {sample_rate_hz} Hz playback")
                })?;
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

#[cfg(test)]
mod tests {
    use super::{supported_at_rate, SupportedRange};

    /// A stand-in for a cpal supported-config range, so the rate matching can be
    /// tested without a sound card. Returns the rate it was pinned to.
    struct FakeRange {
        min: u32,
        max: u32,
    }

    impl SupportedRange for FakeRange {
        type Config = u32;
        fn min_rate(&self) -> cpal::SampleRate {
            cpal::SampleRate(self.min)
        }
        fn max_rate(&self) -> cpal::SampleRate {
            cpal::SampleRate(self.max)
        }
        fn pin_to(self, rate: cpal::SampleRate) -> u32 {
            rate.0
        }
    }

    fn ranges() -> Vec<FakeRange> {
        vec![
            FakeRange {
                min: 8_000,
                max: 44_100,
            },
            FakeRange {
                min: 8_000,
                max: 192_000,
            },
        ]
    }

    #[test]
    fn picks_a_range_covering_the_requested_rate() {
        // 48 kHz is outside the first range and inside the second: the matcher
        // must keep looking rather than settle for the first entry, which is
        // exactly the ALSA layout that made the device default 44.1 kHz.
        assert_eq!(
            supported_at_rate(ranges().into_iter(), 48_000),
            Some(48_000)
        );
    }

    #[test]
    fn pins_to_the_requested_rate_not_the_range_maximum() {
        // The chosen range spans up to 192 kHz; the stream must run at the rate
        // the rest of the pipeline assumes, not at whatever the range allows.
        assert_eq!(
            supported_at_rate(ranges().into_iter(), 16_000),
            Some(16_000)
        );
    }

    #[test]
    fn reports_no_match_rather_than_substituting_a_rate() {
        // Silently substituting a supported rate is the bug being fixed: the
        // caller must be able to fail loudly instead.
        assert_eq!(supported_at_rate(ranges().into_iter(), 384_000), None);
    }

    #[test]
    fn boundary_rates_are_inclusive() {
        assert_eq!(
            supported_at_rate(ranges().into_iter(), 44_100),
            Some(44_100)
        );
        assert_eq!(supported_at_rate(ranges().into_iter(), 8_000), Some(8_000));
    }
}
