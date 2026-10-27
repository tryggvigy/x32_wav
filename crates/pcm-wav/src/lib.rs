//! Mono, 24-bit PCM WAV output with checked RIFF sizes.
//!
//! Each sample consists of three little-endian bytes. Callers provide the sample
//! count before writing audio. [`Mono24Writer`] checks that the final data length
//! matches this count and adds RIFF padding when the data length is odd.
//!
//! This crate writes classic RIFF/WAVE files. It does not support RF64 or data
//! lengths that exceed the 32-bit RIFF size fields.

use std::io::{self, Write};

/// Build a 44-byte header for mono, 24-bit PCM audio.
///
/// `sample_count` is the number of mono samples, and `sample_rate` is in hertz.
/// Zero samples are allowed. The RIFF length includes a final padding byte for
/// odd data lengths, but the data chunk length excludes that byte.
/// The caller must write the audio and padding after this header.
///
/// # Errors
///
/// Returns [`io::ErrorKind::InvalidInput`] if the data length, RIFF length, or
/// byte rate exceeds its 32-bit field, or if `sample_rate` is zero.
///
/// # Examples
///
/// ```
/// let header = pcm_wav::header(48_000, 48_000)?;
/// assert_eq!(&header[..4], b"RIFF");
/// assert_eq!(u32::from_le_bytes(header[40..44].try_into()?), 144_000);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn header(sample_count: u64, sample_rate: u32) -> io::Result<[u8; 44]> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidInput, "audio exceeds WAV size limits");
    let data_bytes = sample_count
        .checked_mul(3)
        .and_then(|bytes| u32::try_from(bytes).ok())
        .ok_or_else(invalid)?;
    let padding = data_bytes % 2;
    let riff_bytes = data_bytes.checked_add(36 + padding).ok_or_else(invalid)?;
    let byte_rate = sample_rate
        .checked_mul(3)
        .filter(|rate| *rate > 0)
        .ok_or_else(invalid)?;
    let mut bytes = [0; 44];
    bytes[..4].copy_from_slice(b"RIFF");
    bytes[4..8].copy_from_slice(&riff_bytes.to_le_bytes());
    bytes[8..16].copy_from_slice(b"WAVEfmt ");
    bytes[16..20].copy_from_slice(&16_u32.to_le_bytes());
    bytes[20..22].copy_from_slice(&1_u16.to_le_bytes());
    bytes[22..24].copy_from_slice(&1_u16.to_le_bytes());
    bytes[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    bytes[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    bytes[32..34].copy_from_slice(&3_u16.to_le_bytes());
    bytes[34..36].copy_from_slice(&24_u16.to_le_bytes());
    bytes[36..40].copy_from_slice(b"data");
    bytes[40..44].copy_from_slice(&data_bytes.to_le_bytes());
    Ok(bytes)
}

/// Write a fixed number of mono, 24-bit PCM samples to an owned destination.
///
/// Audio bytes must already contain little-endian, signed 24-bit PCM samples.
/// Call [`finish`](Self::finish) to check the final sample count, add padding,
/// and report flush errors. Dropping this writer does not finish the WAV file.
///
/// # Examples
///
/// ```
/// use pcm_wav::Mono24Writer;
///
/// let mut bytes = Vec::new();
/// let mut writer = Mono24Writer::new(&mut bytes, 1, 48_000)?;
/// writer.write_samples(&[0x11, 0x22, 0x33])?;
/// writer.finish()?;
/// assert_eq!(&bytes[44..], &[0x11, 0x22, 0x33, 0]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct Mono24Writer<W: Write> {
    /// Destination for the header, audio data, and optional padding byte.
    writer: W,
    /// Number of declared audio bytes still required before finalization.
    remaining: u64,
    /// Whether the data chunk requires one trailing zero byte for alignment.
    padding: bool,
}

impl<W: Write> Mono24Writer<W> {
    /// Validate the format and immediately write the WAV header to `writer`.
    ///
    /// `sample_count` fixes the total number of samples accepted by this writer.
    /// `sample_rate` is in hertz. The destination can be a file or a memory buffer.
    ///
    /// # Errors
    ///
    /// Returns the validation errors from [`header`] or an underlying write error.
    pub fn new(mut writer: W, sample_count: u64, sample_rate: u32) -> io::Result<Self> {
        writer.write_all(&header(sample_count, sample_rate)?)?;
        Ok(Self {
            writer,
            remaining: sample_count * 3,
            padding: sample_count % 2 == 1,
        })
    }

    /// Append complete samples without exceeding the declared sample count.
    ///
    /// `bytes` contains packed three-byte samples. An empty slice is allowed.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::InvalidInput`] if the length is not a multiple
    /// of three or exceeds the remaining data length. Also returns write errors.
    /// An I/O error can leave partial data in the destination. Discard the writer
    /// after such an error.
    pub fn write_samples(&mut self, bytes: &[u8]) -> io::Result<()> {
        if !bytes.len().is_multiple_of(3) || bytes.len() as u64 > self.remaining {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid PCM sample length",
            ));
        }
        self.writer.write_all(bytes)?;
        self.remaining -= bytes.len() as u64;
        Ok(())
    }

    /// Consume the writer, add any RIFF padding, and flush the destination.
    ///
    /// A successful flush does not guarantee persistence to physical storage.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] if fewer samples were written
    /// than declared. Also returns errors from padding writes or flushing.
    pub fn finish(mut self) -> io::Result<()> {
        if self.remaining != 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "missing PCM samples",
            ));
        }
        if self.padding {
            self.writer.write_all(&[0])?;
        }
        self.writer.flush()
    }
}

#[cfg(test)]
mod tests;
