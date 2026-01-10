//! Progressive audio chunking during recording
//!
//! This module provides progressive chunking of audio streams during recording,
//! enabling transcription to begin before recording completes.
//!
//! ## Features
//! - Fixed duration chunking (90s default)
//! - VAD-aware chunking (chunks at silence near target duration)
//! - 2-second overlap between chunks for better accuracy
//!
//! ## Architecture
//! ```text
//! Audio Stream → ProgressiveChunker → Chunks with overlap
//!     ↓
//! Accumulate samples
//!     ↓
//! Detect boundary (90s or VAD silence)
//!     ↓
//! Create chunk with 2s overlap
//!     ↓
//! Send to transcription queue
//! ```

use std::collections::VecDeque;
use tokio::sync::mpsc;

use crate::resample::WHISPER_SAMPLE_RATE;

use super::vad::VadState;

/// Overlap duration in seconds (used for all providers)
const OVERLAP_SECS: usize = 2;

/// Overlap in samples at 16kHz
const OVERLAP_SAMPLES: usize = OVERLAP_SECS * WHISPER_SAMPLE_RATE as usize;

/// Audio chunk with metadata
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// Chunk index (0-based)
    pub index: usize,
    /// Audio samples (16kHz mono f32)
    pub samples: Vec<f32>,
    /// Whether this chunk has leading overlap from previous chunk
    pub has_leading_overlap: bool,
}

/// Configuration for progressive chunking
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Target chunk duration in seconds (default: 90)
    pub target_duration_secs: u64,
    /// Minimum chunk duration for VAD-aware mode (default: 60)
    pub min_duration_secs: u64,
    /// Maximum chunk duration to force chunking (default: 120)
    pub max_duration_secs: u64,
    /// Use VAD-aware chunking (chunk at silence near target)
    pub vad_aware: bool,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            target_duration_secs: 90,
            min_duration_secs: 60,
            max_duration_secs: 120,
            vad_aware: true,
        }
    }
}

/// Buffer for accumulating audio samples with overlap management
struct ChunkBuffer {
    /// Current chunk being accumulated
    current_chunk: Vec<f32>,
    /// Rolling buffer of last 2 seconds for overlap
    overlap_buffer: VecDeque<f32>,
    /// Current chunk index
    chunk_index: usize,
}

impl ChunkBuffer {
    fn new() -> Self {
        Self {
            current_chunk: Vec::new(),
            overlap_buffer: VecDeque::with_capacity(OVERLAP_SAMPLES + 1024),
            chunk_index: 0,
        }
    }

    /// Add samples to buffer
    fn add_samples(&mut self, samples: &[f32]) {
        // Add to current chunk
        self.current_chunk.extend(samples);

        // Add to overlap buffer and keep only last 2 seconds
        self.overlap_buffer.extend(samples);
        while self.overlap_buffer.len() > OVERLAP_SAMPLES {
            self.overlap_buffer.pop_front();
        }
    }

    /// Get current duration in seconds
    fn duration_secs(&self) -> u64 {
        (self.current_chunk.len() as f32 / WHISPER_SAMPLE_RATE as f32) as u64
    }

    /// Create a chunk and prepare buffer for next chunk
    fn create_chunk(&mut self) -> AudioChunk {
        let chunk = AudioChunk {
            index: self.chunk_index,
            samples: std::mem::take(&mut self.current_chunk),
            has_leading_overlap: self.chunk_index > 0,
        };

        // Prepend overlap to next chunk (for continuity)
        self.current_chunk.extend(self.overlap_buffer.iter());

        self.chunk_index += 1;
        chunk
    }

    /// Create final chunk (no overlap needed for next)
    fn create_final_chunk(&mut self) -> Option<AudioChunk> {
        if self.current_chunk.is_empty() {
            return None;
        }

        Some(AudioChunk {
            index: self.chunk_index,
            samples: std::mem::take(&mut self.current_chunk),
            has_leading_overlap: self.chunk_index > 0,
        })
    }
}

/// Progressive audio chunker
///
/// Consumes streaming audio and produces chunks based on:
/// - Fixed duration (if VAD disabled)
/// - VAD-aware silence detection (if VAD enabled)
pub struct ProgressiveChunker {
    config: ChunkerConfig,
    buffer: ChunkBuffer,
    chunk_tx: mpsc::UnboundedSender<AudioChunk>,
}

impl ProgressiveChunker {
    /// Create a new progressive chunker
    pub fn new(config: ChunkerConfig, chunk_tx: mpsc::UnboundedSender<AudioChunk>) -> Self {
        Self {
            config,
            buffer: ChunkBuffer::new(),
            chunk_tx,
        }
    }

    /// Check if we should create a chunk
    ///
    /// Decision logic:
    /// - If VAD disabled: Chunk at exactly target_duration_secs
    /// - If VAD enabled:
    ///   - Chunk if duration >= min AND VAD is in silence
    ///   - Force chunk if duration >= max (regardless of VAD)
    fn should_chunk(&self, vad_state: Option<VadState>) -> bool {
        let duration = self.buffer.duration_secs();

        if let Some(state) = vad_state
            && self.config.vad_aware
        {
            // VAD-aware: Look for silence near target
            if duration >= self.config.min_duration_secs && state.is_silence() {
                // Found natural pause after minimum duration
                return true;
            }
            if duration >= self.config.max_duration_secs {
                // Force chunk at maximum duration
                return true;
            }
            return false;
        }

        // Fixed duration: Chunk at target
        duration >= self.config.target_duration_secs
    }

    /// Consume audio stream and produce chunks
    ///
    /// Reads from audio_rx, accumulates samples, and sends chunks
    /// to chunk_tx when boundaries are detected.
    pub async fn consume_stream(
        &mut self,
        mut audio_rx: mpsc::UnboundedReceiver<Vec<f32>>,
        mut vad_state_rx: Option<mpsc::UnboundedReceiver<VadState>>,
    ) -> Result<(), String> {
        let mut current_vad_state: Option<VadState> = None;

        loop {
            tokio::select! {
                // Receive audio samples
                Some(samples) = audio_rx.recv() => {
                    self.buffer.add_samples(&samples);

                    // Check if we should chunk
                    if self.should_chunk(current_vad_state) {
                        let chunk = self.buffer.create_chunk();
                        crate::verbose!(
                            "Created chunk {} ({:.1}s)",
                            chunk.index,
                            chunk.samples.len() as f32 / WHISPER_SAMPLE_RATE as f32
                        );
                        self.chunk_tx.send(chunk).map_err(|e| e.to_string())?;
                    }
                }

                // Receive VAD state updates (if enabled)
                Some(state) = async {
                    match &mut vad_state_rx {
                        Some(rx) => rx.recv().await,
                        None => None,
                    }
                } => {
                    current_vad_state = Some(state);
                }

                // Audio stream closed - send final chunk
                else => {
                    if let Some(final_chunk) = self.buffer.create_final_chunk() {
                        crate::verbose!(
                            "Created final chunk {} ({:.1}s)",
                            final_chunk.index,
                            final_chunk.samples.len() as f32 / WHISPER_SAMPLE_RATE as f32
                        );
                        self.chunk_tx.send(final_chunk).map_err(|e| e.to_string())?;
                    }
                    break;
                }
            }
        }

        Ok(())
    }
}
