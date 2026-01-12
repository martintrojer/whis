//! Audio encoding module providing MP3 encoding via embedded LAME encoder.

#[cfg(feature = "embedded-encoder")]
mod embedded;

use anyhow::Result;

/// Trait for encoding raw audio samples to compressed formats.
pub trait AudioEncoder: Send + Sync {
    /// Encode raw f32 PCM samples to MP3.
    ///
    /// # Parameters
    /// - `samples`: Raw audio samples (f32 PCM, expected to be 16kHz mono)
    /// - `sample_rate`: Sample rate of the input audio
    ///
    /// # Returns
    /// Encoded MP3 data as bytes
    fn encode_samples(&self, samples: &[f32], sample_rate: u32) -> Result<Vec<u8>>;
}

/// Create the audio encoder using embedded LAME library.
///
/// Uses mp3lame-encoder crate which wraps the same LAME library as FFmpeg's libmp3lame,
/// so audio quality is identical while eliminating the FFmpeg runtime dependency.
pub fn create_encoder() -> Box<dyn AudioEncoder> {
    #[cfg(feature = "embedded-encoder")]
    {
        Box::new(embedded::EmbeddedEncoder::new())
    }

    #[cfg(not(feature = "embedded-encoder"))]
    {
        panic!("No audio encoder available. Enable the 'embedded-encoder' feature.");
    }
}
