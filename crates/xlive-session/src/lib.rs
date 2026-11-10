//! Parse X-LIVE sessions and stream selected channels into mono WAV files.
//!
//! [`Session`] reads `SE_LOG.BIN` from a recording directory. Extraction reads
//! numbered take files such as `00000001.WAV`, skips their 32 KiB headers, and
//! converts interleaved 32-bit words into one 24-bit WAV file per selected channel.
//!
//! Channel numbers start at one. Marker indices start at zero. Sample positions
//! count audio frames: one sample for each channel at a single point in time.
//! All audio intervals include their start position and exclude their end position.
//!
//! Metadata parsing and range planning do not open audio files. Extraction opens
//! one take at a time and uses bounded buffers. This crate does not print messages.
//!
//! # Examples
//!
//! ```no_run
//! use xlive_session::{Channels, Interval, Session};
//!
//! let session = Session::open("recording")?;
//! assert!(session.metadata().channel_count() > 0);
//! session.extract(Channels::One(1), Interval::Full, "channel-one")?;
//! # Ok::<(), std::io::Error>(())
//! ```

mod audio;
mod extract;
mod metadata;
mod name;
mod plan;

use std::io;
use std::path::{Path, PathBuf};

pub use metadata::Metadata;
pub use plan::{Channels, Interval};

/// A recording directory and the validated metadata read when it was opened.
///
/// Opening a session closes the log file after parsing. Take files are opened
/// only during extraction. Metadata is a snapshot and does not refresh when
/// files on disk change. Keep the recording unchanged while extracting audio.
pub struct Session {
    /// Directory used to resolve the session log and numbered take filenames.
    root: PathBuf,
    /// Validated metadata snapshot shared by extraction and information queries.
    metadata: Metadata,
}

impl Session {
    /// Read and validate `SE_LOG.BIN` inside `root`.
    ///
    /// The supplied path is stored as given. Relative paths remain relative to
    /// the process working directory for later extraction and rename operations.
    /// This method does not check that the take files exist.
    ///
    /// # Errors
    ///
    /// Returns file access errors or the parsing errors from [`Metadata::read`].
    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        let mut log = std::fs::File::open(root.join("SE_LOG.BIN"))?;
        let metadata = Metadata::read(&mut log)?;
        Ok(Self { root, metadata })
    }

    /// Borrow the validated metadata snapshot without reading the log again.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

/// Build an invalid-input error for a rejected caller request.
fn invalid_input(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
