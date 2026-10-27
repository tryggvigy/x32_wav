//! Check binary metadata validation and isolated updates to the name field.

mod common;

use common::{log_bytes, Fixture};
use xlive_session::{Metadata, Session};

#[test]
/// Decode the session ID and convert interleaved take lengths into frame counts.
fn parses_little_endian_metadata_and_derives_counts() {
    let metadata = Metadata::read(&mut log_bytes().as_slice()).unwrap();
    assert_eq!(metadata.session_id(), 0x1234abcd);
    assert_eq!(metadata.channel_count(), 2);
    assert_eq!(metadata.sample_rate(), 2);
    assert_eq!(metadata.total_samples(), 6);
    assert_eq!(metadata.take_samples(), [3, 3]);
    assert_eq!(metadata.markers(), [1, 5]);
}

#[test]
/// Reject incomplete logs, invalid format values, and unaligned take lengths.
fn rejects_truncated_and_invalid_metadata() {
    let bytes = log_bytes();
    assert!(Metadata::read(&mut &bytes[..bytes.len() - 1]).is_err());
    for (word, value) in [(1, 0_u32), (1, 33), (2, 0), (4, 257), (7, 5)] {
        let mut invalid = bytes.clone();
        invalid[word * 4..word * 4 + 4].copy_from_slice(&value.to_le_bytes());
        assert!(Metadata::read(&mut invalid.as_slice()).is_err());
    }
}

#[test]
/// Preserve other log bytes and enforce the encoded name's byte limit and terminator.
fn naming_changes_only_the_fixed_name_field() {
    let fixture = Fixture::new();
    let path = fixture.0.join("SE_LOG.BIN");
    let mut original = log_bytes();
    original.resize(1600, 0xaa);
    std::fs::write(&path, &original).unwrap();
    let session = Session::open(&fixture.0).unwrap();
    session.rename("test").unwrap();
    let renamed = std::fs::read(&path).unwrap();
    assert_eq!(&renamed[..1552], &original[..1552]);
    assert_eq!(&renamed[1552..1556], b"test");
    assert_eq!(&renamed[1556..1572], &[0; 16]);
    assert_eq!(&renamed[1572..], &original[1572..]);
    assert!(session.rename("twenty characters!!!").is_err());
    assert!(session.rename("bad\0name").is_err());
    session.rename("é").unwrap();
    assert_eq!(&std::fs::read(&path).unwrap()[1552..1554], "é".as_bytes());
}
