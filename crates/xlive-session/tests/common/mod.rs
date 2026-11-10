//! Synthetic session fixtures shared by metadata and extraction tests.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Build a two-channel log with six frames, two takes, and markers at frames 1 and 5.
///
/// The bytes end after the markers and do not include a session-name field.
pub fn log_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    // Two channels, six frames across two takes, markers at frames 1 and 5.
    for value in [0x1234abcd_u32, 2, 2, 0, 2, 2, 6, 6, 6] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.resize((7 + 256) * 4, 0);
    for marker in [1_u32, 5] {
        bytes.extend_from_slice(&marker.to_le_bytes());
    }
    bytes
}

/// An owned temporary session directory, removed when the fixture is dropped.
pub struct Fixture(
    /// Directory containing the log, synthetic takes, and any test outputs.
    pub std::path::PathBuf,
);

impl Fixture {
    /// Create a log and two takes whose sample bytes identify their frame and channel.
    ///
    /// Each take starts with a 32 KiB placeholder header and contains three frames.
    ///
    /// # Panics
    ///
    /// Panics if the temporary directory or fixture files cannot be created.
    pub fn new() -> Self {
        /// Distinguish concurrently created fixtures within the same test process.
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "x32-wav-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("SE_LOG.BIN"), log_bytes()).unwrap();
        for take in 0..2 {
            let mut bytes = vec![0; 32 * 1024];
            for frame in take * 3..take * 3 + 3 {
                for channel in 0..2 {
                    bytes.extend_from_slice(&[0xee, frame, channel, 0x7f]);
                }
            }
            std::fs::write(path.join(format!("{:08}.WAV", take + 1)), bytes).unwrap();
        }
        Self(path)
    }
}

impl Drop for Fixture {
    /// Remove the fixture directory, including partial outputs from error tests.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
