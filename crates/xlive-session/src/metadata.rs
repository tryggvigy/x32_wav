//! Decode and validate the little-endian fields in `SE_LOG.BIN`.

use std::io::{self, Read};

/// Validated session metadata decoded from an X-LIVE log.
///
/// Sample counts and marker positions are frames: one sample per channel.
/// The parser converts take lengths from interleaved 32-bit words into frames.
/// Take and marker counts come from their collection lengths.
#[derive(Debug)]
pub struct Metadata {
    /// Raw session identifier, also used in hexadecimal output folder names.
    pub(crate) session_id: u32,
    /// Number of interleaved channels, in the inclusive range 1 through 32.
    pub(crate) channel_count: u32,
    /// Nonzero sample rate in frames per second.
    pub(crate) sample_rate: u32,
    /// Recorder date code, retained without calendar conversion.
    date_code: u32,
    /// Declared session length in frames, excluding any excess take data.
    pub(crate) total_samples: u64,
    /// Nonzero frame count for each numbered take in recording order.
    pub(crate) take_samples: Vec<u64>,
    /// Zero-based frame positions in the marker order stored by the recorder.
    pub(crate) markers: Vec<u64>,
}

impl Metadata {
    /// Parse session metadata from the reader's current position.
    ///
    /// The layout contains seven 32-bit fields, 256 take-length slots, then
    /// the declared marker positions. All integers use little-endian byte order.
    /// Parsing stops after the last marker and leaves the session name unread.
    ///
    /// Validation requires 1–32 channels, a nonzero sample rate and session length,
    /// and 1–256 takes. Takes must contain complete, nonempty audio frames.
    /// Their combined length must cover the session, and markers cannot exceed it.
    /// Marker positions do not need to be sorted.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::InvalidData`] if a metadata value violates these
    /// rules. A truncated log returns [`io::ErrorKind::UnexpectedEof`]. Other
    /// reader errors pass through unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use xlive_session::Metadata;
    ///
    /// // One channel, two frames, one take, and no markers.
    /// let mut bytes = Vec::new();
    /// for word in [0x1234abcd_u32, 1, 48_000, 0, 1, 0, 2, 2] {
    ///     bytes.extend_from_slice(&word.to_le_bytes());
    /// }
    /// bytes.resize((7 + 256) * 4, 0);
    /// let metadata = Metadata::read(&mut bytes.as_slice())?;
    /// assert_eq!(metadata.session_id(), 0x1234abcd);
    /// assert_eq!(metadata.take_samples(), &[2]);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let session_id = read_u32(reader)?;
        let channel_count = read_u32(reader)?;
        let sample_rate = read_u32(reader)?;
        let date_code = read_u32(reader)?;
        let take_count = read_u32(reader)?;
        let marker_count = read_u32(reader)?;
        let total_samples = u64::from(read_u32(reader)?);
        if !(1..=32).contains(&channel_count)
            || sample_rate == 0
            || !(1..=256).contains(&take_count)
            || total_samples == 0
        {
            return Err(invalid_data(
                "invalid session channels, sample rate, takes, or length",
            ));
        }
        let mut take_samples = Vec::new();
        for _ in 0..take_count {
            let words = read_u32(reader)?;
            if words == 0 || !words.is_multiple_of(channel_count) {
                return Err(invalid_data(
                    "take length must contain complete audio frames",
                ));
            }
            take_samples.push(u64::from(words / channel_count));
        }
        for _ in take_count..256 {
            read_u32(reader)?;
        }
        if take_samples.iter().sum::<u64>() < total_samples {
            return Err(invalid_data(
                "takes are shorter than the declared session length",
            ));
        }
        let mut markers = Vec::new();
        for _ in 0..marker_count {
            let marker = u64::from(read_u32(reader)?);
            if marker > total_samples {
                return Err(invalid_data("marker is beyond the session length"));
            }
            markers.push(marker);
        }
        Ok(Self {
            session_id,
            channel_count,
            sample_rate,
            date_code,
            total_samples,
            take_samples,
            markers,
        })
    }

    /// Return the recorder's session identifier as an unsigned integer.
    pub fn session_id(&self) -> u32 {
        self.session_id
    }
    /// Return the channel count, in the inclusive range 1 through 32.
    pub fn channel_count(&self) -> u32 {
        self.channel_count
    }
    /// Return the nonzero sample rate in hertz.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    /// Return the raw recorder date code without calendar conversion.
    pub fn date_code(&self) -> u32 {
        self.date_code
    }
    /// Return the declared session length in frames, or samples per channel.
    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }
    /// Return take lengths in frames, in recording order.
    ///
    /// Element zero describes `00000001.WAV`. The sum can exceed the declared
    /// session length. Extraction excludes data beyond [`total_samples`](Self::total_samples).
    pub fn take_samples(&self) -> &[u64] {
        &self.take_samples
    }
    /// Return marker frame positions relative to the session start.
    ///
    /// Slice indices are the zero-based marker indices used by [`crate::Interval::Markers`].
    /// A marker can equal the session length and serve as an extraction endpoint.
    pub fn markers(&self) -> &[u64] {
        &self.markers
    }
}

/// Read exactly four bytes and decode one little-endian integer.
///
/// # Errors
///
/// Returns reader errors, including unexpected end of input.
fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Build an invalid-data error for a rejected log field.
fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
