//! ARC-06 spectrum / FFT pipeline (Phase 2).
//!
//! Computes a magnitude spectrum (FFT bins, in dB) from a block of samples
//! (FR-SPEC-01), using a pure-Rust FFT (`rustfft`) so the aarch64 cross-build
//! stays free of a C toolchain. The sample source is abstracted behind
//! [`SampleSource`]: this build ships a synthetic generator (used until the
//! Phase-3 audio capture provides real samples) so the whole spectrum path can
//! be exercised without audio hardware.

// DSP: sample counts and rates are small integers that convert exactly to f32
// (well under 2^24); the "precision loss" cast lints don't apply meaningfully.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]

use std::f32::consts::PI;
use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

/// A block-based FFT magnitude analyser.
pub struct SpectrumAnalyzer {
    size: usize,
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
}

impl SpectrumAnalyzer {
    /// Create an analyser for `size`-sample blocks. `size` should be a power of
    /// two for efficiency.
    #[must_use]
    pub fn new(size: usize) -> Self {
        let fft = FftPlanner::new().plan_fft_forward(size);
        let window = hann_window(size);
        Self { size, fft, window }
    }

    /// The number of output bins (`size / 2`, the non-redundant half).
    #[must_use]
    pub fn bin_count(&self) -> usize {
        self.size / 2
    }

    /// Compute the magnitude spectrum in dB for one block of samples.
    ///
    /// `samples` is windowed (Hann), zero-padded or truncated to the FFT size,
    /// transformed, and reduced to `size / 2` bins of `20·log10(|X|/N)`.
    #[must_use]
    pub fn analyze(&self, samples: &[f32]) -> Vec<f32> {
        let mut buffer = vec![Complex::new(0.0_f32, 0.0); self.size];
        for (slot, (sample, window)) in buffer.iter_mut().zip(samples.iter().zip(&self.window)) {
            slot.re = sample * window;
        }
        self.fft.process(&mut buffer);

        let norm = self.size as f32;
        buffer
            .iter()
            .take(self.size / 2)
            .map(|c| {
                let magnitude = c.norm() / norm;
                20.0 * (magnitude + 1e-9).log10()
            })
            .collect()
    }
}

/// A source of real-valued samples for the spectrum pipeline. The Phase-3 audio
/// capture implements this; [`SyntheticSource`] stands in until then.
pub trait SampleSource: Send + Sync {
    /// Produce the next `size` samples (normalised to roughly [-1, 1]).
    fn next_block(&self, size: usize) -> Vec<f32>;
}

/// A deterministic synthetic source: a tone at `tone_hz` plus a low-level
/// second harmonic, so the spectrum path produces a recognisable, moving-free
/// signal without audio hardware.
pub struct SyntheticSource {
    sample_rate: u32,
    tone_hz: f32,
}

impl SyntheticSource {
    #[must_use]
    pub fn new(sample_rate: u32, tone_hz: f32) -> Self {
        Self {
            sample_rate,
            tone_hz,
        }
    }
}

impl SampleSource for SyntheticSource {
    fn next_block(&self, size: usize) -> Vec<f32> {
        let step = 2.0 * PI * self.tone_hz / self.sample_rate as f32;
        (0..size)
            .map(|n| {
                let phase = step * n as f32;
                phase.sin() + 0.25 * (2.0 * phase).sin()
            })
            .collect()
    }
}

fn hann_window(size: usize) -> Vec<f32> {
    if size <= 1 {
        return vec![1.0; size];
    }
    let denom = (size - 1) as f32;
    (0..size)
        .map(|n| 0.5 - 0.5 * (2.0 * PI * n as f32 / denom).cos())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{SampleSource, SpectrumAnalyzer, SyntheticSource};
    use std::f32::consts::PI;

    fn tone(size: usize, bin: usize) -> Vec<f32> {
        // A pure sinusoid whose frequency lands exactly on `bin`.
        (0..size)
            .map(|n| (2.0 * PI * bin as f32 * n as f32 / size as f32).sin())
            .collect()
    }

    #[test]
    fn bin_count_is_half_the_fft_size() {
        assert_eq!(SpectrumAnalyzer::new(1024).bin_count(), 512);
    }

    #[test]
    fn a_tone_peaks_in_its_bin() {
        // FR-SPEC-01: the FFT must localise a tone to the correct bin.
        let size = 1024;
        let target = 64;
        let analyzer = SpectrumAnalyzer::new(size);
        let spectrum = analyzer.analyze(&tone(size, target));

        let peak = spectrum
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(index, _)| index)
            .unwrap();
        // Windowing spreads energy slightly; the peak must be at (or adjacent to)
        // the target bin.
        assert!(
            (peak as i64 - target as i64).abs() <= 1,
            "peak at {peak}, want {target}"
        );
    }

    #[test]
    fn synthetic_source_yields_requested_length() {
        let source = SyntheticSource::new(48_000, 1_500.0);
        assert_eq!(source.next_block(1024).len(), 1024);
    }

    #[test]
    fn synthetic_tone_is_detectable() {
        let size = 1024;
        let sample_rate = 48_000;
        let tone_hz = sample_rate as f32 * 64.0 / size as f32; // lands on bin 64
        let analyzer = SpectrumAnalyzer::new(size);
        let source = SyntheticSource::new(sample_rate, tone_hz);
        let spectrum = analyzer.analyze(&source.next_block(size));
        let peak = spectrum
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(index, _)| index)
            .unwrap();
        assert!((peak as i64 - 64).abs() <= 1);
    }
}
