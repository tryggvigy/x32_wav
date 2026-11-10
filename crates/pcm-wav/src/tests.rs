//! Check WAV sizes, final padding, sample counts, and flush error propagation.

use super::*;

#[test]
/// Include odd-length padding in RIFF size, but exclude it from data size.
fn writes_header_samples_and_riff_padding() {
    let mut bytes = Vec::new();
    let mut writer = Mono24Writer::new(&mut bytes, 1, 48_000).unwrap();
    writer.write_samples(&[0x11, 0x22, 0x33]).unwrap();
    writer.finish().unwrap();
    assert_eq!(&bytes[..4], b"RIFF");
    assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 40);
    assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 3);
    assert_eq!(&bytes[44..], &[0x11, 0x22, 0x33, 0]);
}

#[test]
/// Reject oversized headers, incomplete samples, and missing declared audio.
fn rejects_unrepresentable_sizes_and_sample_lengths() {
    assert!(header(u64::MAX, 48_000).is_err());
    assert!(header(u32::MAX as u64 / 3, 48_000).is_err());
    assert!(header(1, 0).is_err());
    let mut writer = Mono24Writer::new(Vec::new(), 1, 48_000).unwrap();
    assert!(writer.write_samples(&[0; 2]).is_err());
    assert!(writer.write_samples(&[0; 6]).is_err());
    assert!(writer.finish().is_err());
}

#[test]
/// Require explicit finalization to report errors from the destination's flush.
fn reports_flush_failure() {
    /// Accept all writes and fail only when the caller flushes the destination.
    struct FailingFlush;
    impl Write for FailingFlush {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush failed"))
        }
    }
    assert!(Mono24Writer::new(FailingFlush, 0, 48_000)
        .unwrap()
        .finish()
        .is_err());
}
