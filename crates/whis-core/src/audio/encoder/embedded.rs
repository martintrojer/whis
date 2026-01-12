//! Embedded LAME encoder implementation.
//!
//! Uses the mp3lame-encoder crate which wraps the same LAME library (libmp3lame)
//! as FFmpeg's MP3 encoder, providing identical audio quality without requiring
//! FFmpeg as a runtime dependency.

use anyhow::{Context, Result};
use mp3lame_encoder::{Builder, FlushNoGap, InterleavedPcm, MonoPcm};

use super::AudioEncoder;

/// MP3 encoder using embedded LAME library.
///
/// Uses mp3lame-encoder crate for high-quality MP3 encoding without external dependencies.
pub struct EmbeddedEncoder {
    channels: u16,
}

impl EmbeddedEncoder {
    /// Create a new embedded LAME encoder.
    ///
    /// Always configured for mono (1 channel) output.
    pub fn new() -> Self {
        Self { channels: 1 }
    }

    /// Convert f32 samples to i16 PCM format.
    fn samples_to_i16(&self, samples: &[f32]) -> Vec<i16> {
        samples
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                (clamped * i16::MAX as f32) as i16
            })
            .collect()
    }

    /// Build and configure the LAME encoder.
    fn build_encoder(&self, sample_rate: u32) -> Result<mp3lame_encoder::Encoder> {
        let mut builder = Builder::new().context("Failed to create LAME builder")?;

        builder
            .set_num_channels(self.channels as u8)
            .map_err(|e| anyhow::anyhow!("Failed to set channels: {:?}", e))?;

        builder
            .set_sample_rate(sample_rate)
            .map_err(|e| anyhow::anyhow!("Failed to set sample rate: {:?}", e))?;

        builder
            .set_brate(mp3lame_encoder::Bitrate::Kbps128)
            .map_err(|e| anyhow::anyhow!("Failed to set bitrate: {:?}", e))?;

        builder
            .set_quality(mp3lame_encoder::Quality::Best)
            .map_err(|e| anyhow::anyhow!("Failed to set quality: {:?}", e))?;

        builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to initialize LAME encoder: {:?}", e))
    }

    /// Encode samples using the configured encoder.
    fn encode_and_flush(
        &self,
        encoder: &mut mp3lame_encoder::Encoder,
        i16_samples: &[i16],
    ) -> Result<Vec<u8>> {
        // Prepare output buffer
        let mut mp3_data = Vec::new();
        let max_size = mp3lame_encoder::max_required_buffer_size(i16_samples.len());
        mp3_data.reserve(max_size);

        // Encode based on channel count
        let encoded_size = if self.channels == 1 {
            let input = MonoPcm(i16_samples);
            encoder
                .encode(input, mp3_data.spare_capacity_mut())
                .map_err(|e| anyhow::anyhow!("Failed to encode MP3: {:?}", e))?
        } else {
            let input = InterleavedPcm(i16_samples);
            encoder
                .encode(input, mp3_data.spare_capacity_mut())
                .map_err(|e| anyhow::anyhow!("Failed to encode MP3: {:?}", e))?
        };

        // SAFETY: encoder.encode returns the number of bytes written to the buffer.
        // The mp3lame-encoder API requires MaybeUninit<u8> output and guarantees
        // that exactly encoded_size bytes are initialized on success.
        unsafe {
            mp3_data.set_len(encoded_size);
        }

        // Flush remaining data
        let flush_size = encoder
            .flush::<FlushNoGap>(mp3_data.spare_capacity_mut())
            .map_err(|e| anyhow::anyhow!("Failed to flush MP3 encoder: {:?}", e))?;

        // SAFETY: flush returns the number of additional bytes written.
        // The encoder guarantees flush_size bytes are initialized.
        unsafe {
            mp3_data.set_len(mp3_data.len() + flush_size);
        }

        Ok(mp3_data)
    }
}

impl Default for EmbeddedEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEncoder for EmbeddedEncoder {
    fn encode_samples(&self, samples: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
        // Convert f32 samples to i16
        let i16_samples = self.samples_to_i16(samples);

        // Build and configure encoder
        let mut encoder = self.build_encoder(sample_rate)?;

        // Encode and flush
        self.encode_and_flush(&mut encoder, &i16_samples)
    }
}
