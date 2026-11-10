//! Update only the fixed session-name field in the existing log file.

use crate::{invalid_input, Session};
use std::fs::OpenOptions;
use std::io::{self, Seek, SeekFrom, Write};

impl Session {
    /// Write up to 19 UTF-8 bytes into the recorder's fixed name field.
    ///
    /// The limit counts encoded bytes, not Unicode characters. Empty names clear
    /// the field. The method zero-fills all unused bytes and the final terminator
    /// in the 20-byte field at byte offset 1552. Other log bytes remain unchanged.
    /// The recorder determines which characters it can display.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::InvalidInput`] if `name` exceeds 19 bytes or
    /// contains NUL. Returns [`io::ErrorKind::UnexpectedEof`] if the existing log
    /// lacks the complete name field. Also returns file access, seek, and write errors.
    /// A write error can leave a partially updated name field.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let session = xlive_session::Session::open("recording")?;
    /// session.rename("Rehearsal")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn rename(&self, name: &str) -> io::Result<()> {
        /// Byte offset of the recorder's fixed 20-byte name field.
        const NAME_OFFSET: u64 = 4 * 388;
        if name.len() > 19 || name.contains('\0') {
            return Err(invalid_input(
                "session name must be at most 19 bytes without NUL characters",
            ));
        }
        let mut field = [0; 20];
        field[..name.len()].copy_from_slice(name.as_bytes());
        let mut log = OpenOptions::new()
            .write(true)
            .open(self.root.join("SE_LOG.BIN"))?;
        if log.metadata()?.len() < NAME_OFFSET + field.len() as u64 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "session name field is missing",
            ));
        }
        log.seek(SeekFrom::Start(NAME_OFFSET))?;
        log.write_all(&field)
    }
}
