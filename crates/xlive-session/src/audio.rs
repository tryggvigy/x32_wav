//! Convert interleaved X-LIVE words into packed PCM bytes in bounded blocks.

use pcm_wav::Mono24Writer;
use std::fs::File;
use std::io::{self, BufWriter, Read};

/// One selected channel's destination and reusable conversion buffer.
pub(crate) struct Output {
    /// Zero-based position of this channel's word within each input frame.
    pub channel_index: usize,
    /// WAV destination shared across all selected take spans.
    pub writer: Mono24Writer<BufWriter<File>>,
    /// Packed 24-bit sample bytes for the current block, reused between blocks.
    pub buffer: Vec<u8>,
}

/// Copy `sample_count` frames from the source's current audio position.
///
/// Each frame contains `channel_count` interleaved, four-byte little-endian words.
/// The upper three bytes of each selected word become one output PCM sample.
/// The input buffer holds at most 4 MiB and ends on a complete frame boundary.
/// Output buffers retain their allocations for later blocks and take spans.
///
/// The caller must supply validated metadata and outputs: 1–32 channels,
/// in-range channel indices, and enough declared samples in each WAV writer.
/// The source must already point past the take header and any skipped frames.
///
/// # Errors
///
/// Returns source read errors, including unexpected end of audio, or errors
/// from [`Mono24Writer::write_samples`]. Output can be incomplete after an error.
///
/// # Panics
///
/// Panics if the internal channel count is zero or an output index exceeds
/// the available channels. The extraction planner prevents these cases.
pub(crate) fn copy_audio(
    source: &mut impl Read,
    sample_count: u64,
    channel_count: u32,
    outputs: &mut [Output],
) -> io::Result<()> {
    let frame_bytes = channel_count as usize * 4;
    let frames_per_block = (4 * 1024 * 1024) / frame_bytes;
    let mut buffer = vec![0; frames_per_block * frame_bytes];
    let mut remaining = sample_count;
    while remaining > 0 {
        let frames = remaining.min(frames_per_block as u64) as usize;
        let block = &mut buffer[..frames * frame_bytes];
        source.read_exact(block)?;
        for output in outputs.iter_mut() {
            output.buffer.clear();
            let offset = output.channel_index * 4;
            for frame in block.chunks_exact(frame_bytes) {
                // X-LIVE stores 24-bit audio in the upper three bytes of each 32-bit word.
                output
                    .buffer
                    .extend_from_slice(&frame[offset + 1..offset + 4]);
            }
            output.writer.write_samples(&output.buffer)?;
        }
        remaining -= frames as u64;
    }
    Ok(())
}
