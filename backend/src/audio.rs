//! ARC-05 audio pipeline — software core (action A31).
//!
//! This module holds the parts of the audio path that are pure and testable
//! without audio hardware or native codecs:
//!
//! - [`JitterBuffer`] — reorders audio frames by sequence number and conceals
//!   losses gracefully (FR-AUD-06).
//! - [`Codec`] — the encode/decode seam (FR-AUD-05). [`PcmCodec`] is the
//!   dependency-free default; a libopus-backed `OpusCodec` is a native adapter
//!   added behind a Cargo feature when building for the Pi, so the default
//!   aarch64 cross-build stays free of a C toolchain.
//!
//! The device ends (CPAL capture/playback on the Pi, Web Audio in the browser)
//! and the WebSocket audio transport plug in behind these seams and are
//! validated hardware-in-the-loop.

// Sample→PCM conversion casts are intentional and bounded (clamped to [-1, 1]).
#![allow(clippy::cast_possible_truncation)]

use std::collections::BTreeMap;

/// A transport-level audio frame: a monotonically increasing sequence number
/// and an opaque (encoded) payload.
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// Monotonic sequence number.
    pub seq: u64,
    /// Encoded payload bytes.
    pub payload: Vec<u8>,
}

/// The result of pulling one frame from the [`JitterBuffer`] for playout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Playout {
    /// A decoded-ready payload.
    Data(Vec<u8>),
    /// A frame was lost; the caller should conceal it (e.g. play silence or a
    /// packet-loss-concealment estimate) (FR-AUD-06).
    Lost,
}

/// A reordering jitter buffer with graceful loss concealment (FR-AUD-06).
///
/// Frames are buffered until `target_depth` are held, then played out in
/// sequence order. A missing frame stalls playout until either it arrives or the
/// backlog exceeds `max_depth`, at which point it is concealed ([`Playout::Lost`])
/// so a single lost packet cannot wedge the stream. Frames older than the next
/// expected sequence number are dropped as late.
pub struct JitterBuffer {
    target_depth: usize,
    max_depth: usize,
    next_seq: Option<u64>,
    started: bool,
    frames: BTreeMap<u64, Vec<u8>>,
}

impl JitterBuffer {
    /// Create a buffer that starts playout once `target_depth` frames are held
    /// and conceals a gap once the backlog exceeds `max_depth`.
    #[must_use]
    pub fn new(target_depth: usize, max_depth: usize) -> Self {
        Self {
            target_depth: target_depth.max(1),
            max_depth: max_depth.max(target_depth.max(1)),
            next_seq: None,
            started: false,
            frames: BTreeMap::new(),
        }
    }

    /// Insert a frame. Frames at or after the next expected sequence are
    /// buffered; late frames (already played past) are dropped.
    pub fn push(&mut self, frame: AudioFrame) {
        if let Some(next) = self.next_seq {
            if frame.seq < next {
                return; // late — already played past this point
            }
        }
        self.frames.insert(frame.seq, frame.payload);
    }

    /// Number of frames currently buffered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Whether the buffer holds no frames.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Pull the next frame for playout, or `None` while pre-buffering or on an
    /// underrun that has not yet reached the concealment threshold.
    pub fn pop(&mut self) -> Option<Playout> {
        if !self.started {
            if self.frames.len() < self.target_depth {
                return None;
            }
            self.started = true;
            self.next_seq = self.frames.keys().next().copied();
        }

        let next = self.next_seq?;
        if let Some(payload) = self.frames.remove(&next) {
            self.next_seq = Some(next + 1);
            return Some(Playout::Data(payload));
        }

        // The next frame is missing. Wait for it unless the backlog is too deep,
        // in which case conceal the gap and move on (FR-AUD-06).
        if self.frames.len() > self.max_depth {
            self.next_seq = Some(next + 1);
            Some(Playout::Lost)
        } else {
            None
        }
    }
}

/// Convert normalised f32 samples (roughly `[-1, 1]`) to 16-bit PCM, clamping
/// to avoid wrap on overflow. Bridges the shared [`crate::spectrum::SampleSource`]
/// (f32) to the PCM codec.
#[must_use]
pub fn f32_to_pcm16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16)
        .collect()
}

/// The audio codec seam (FR-AUD-05): encode PCM samples to a payload and back.
pub trait Codec: Send + Sync {
    /// Encode 16-bit PCM samples to a transport payload.
    fn encode(&self, samples: &[i16]) -> Vec<u8>;
    /// Decode a transport payload back to 16-bit PCM samples.
    fn decode(&self, payload: &[u8]) -> Vec<i16>;
}

/// A dependency-free passthrough codec: little-endian 16-bit PCM. Used as the
/// default and in tests; the WAN default (Opus) is a native adapter.
pub struct PcmCodec;

impl Codec for PcmCodec {
    fn encode(&self, samples: &[i16]) -> Vec<u8> {
        let mut out = Vec::with_capacity(samples.len() * 2);
        for sample in samples {
            out.extend_from_slice(&sample.to_le_bytes());
        }
        out
    }

    fn decode(&self, payload: &[u8]) -> Vec<i16> {
        payload
            .chunks_exact(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioFrame, Codec, JitterBuffer, PcmCodec, Playout};

    fn frame(seq: u64) -> AudioFrame {
        AudioFrame {
            seq,
            payload: vec![u8::try_from(seq).unwrap()],
        }
    }

    #[test]
    fn buffers_until_target_then_plays_in_order() {
        let mut jb = JitterBuffer::new(2, 8);
        jb.push(frame(0));
        assert_eq!(jb.pop(), None, "still pre-buffering");
        jb.push(frame(1));
        assert_eq!(jb.pop(), Some(Playout::Data(vec![0])));
        assert_eq!(jb.pop(), Some(Playout::Data(vec![1])));
        assert_eq!(jb.pop(), None, "underrun");
    }

    #[test]
    fn reorders_out_of_order_frames() {
        let mut jb = JitterBuffer::new(3, 8);
        jb.push(frame(0));
        jb.push(frame(2));
        jb.push(frame(1));
        assert_eq!(jb.pop(), Some(Playout::Data(vec![0])));
        assert_eq!(jb.pop(), Some(Playout::Data(vec![1])));
        assert_eq!(jb.pop(), Some(Playout::Data(vec![2])));
    }

    #[test]
    fn conceals_a_lost_frame_once_the_backlog_is_deep() {
        // target 1 so playout starts immediately; max 2 so a 3-frame backlog past
        // the gap triggers concealment.
        let mut jb = JitterBuffer::new(1, 2);
        jb.push(frame(0));
        assert_eq!(jb.pop(), Some(Playout::Data(vec![0])));
        // seq 1 is lost; 2,3,4 arrive.
        jb.push(frame(2));
        jb.push(frame(3));
        assert_eq!(jb.pop(), None, "waiting for the missing frame 1");
        jb.push(frame(4));
        assert_eq!(jb.pop(), Some(Playout::Lost), "gap concealed");
        assert_eq!(jb.pop(), Some(Playout::Data(vec![2])));
    }

    #[test]
    fn drops_late_frames() {
        let mut jb = JitterBuffer::new(1, 8);
        jb.push(frame(5));
        assert_eq!(jb.pop(), Some(Playout::Data(vec![5])));
        // seq 4 arrives after we've already played past it -> dropped.
        jb.push(frame(4));
        assert!(jb.is_empty());
    }

    #[test]
    fn pcm_codec_round_trips() {
        let codec = PcmCodec;
        let samples = [0_i16, 1, -1, 32_767, -32_768, 1234];
        assert_eq!(codec.decode(&codec.encode(&samples)), samples);
    }

    #[test]
    fn f32_to_pcm16_scales_and_clamps() {
        use super::f32_to_pcm16;
        assert_eq!(f32_to_pcm16(&[0.0, 1.0, -1.0]), [0, 32_767, -32_767]);
        // Out-of-range input is clamped, not wrapped.
        assert_eq!(f32_to_pcm16(&[2.0, -2.0]), [32_767, -32_767]);
    }
}
