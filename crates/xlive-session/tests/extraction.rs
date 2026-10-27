//! Check exported PCM bytes, WAV lengths, range boundaries, and failure behavior.

mod common;

use common::Fixture;
use xlive_session::{Channels, Interval, Session};

#[test]
/// Extract every selection form across take boundaries with exact samples and sizes.
fn extracts_exact_samples_and_headers_for_every_selection() {
    let fixture = Fixture::new();
    let session = Session::open(&fixture.0).unwrap();
    let cases = [
        (Interval::Full, 0, 6),
        (Interval::Seconds(1..3), 2, 6),
        (Interval::Markers(0..1), 1, 5),
        (Interval::Samples(3..6), 3, 6),
        (Interval::Samples(0..3), 0, 3),
    ];
    for (index, (interval, start, end)) in cases.into_iter().enumerate() {
        for selection in [Channels::All, Channels::One(2)] {
            let output = fixture.0.join(format!("out-{index}-{selection:?}"));
            session
                .extract(selection, interval.clone(), &output)
                .unwrap();
            let channels = match selection {
                Channels::All => vec![1, 2],
                Channels::One(channel) => vec![channel],
            };
            for channel in channels {
                let bytes = std::fs::read(output.join(format!("ch_{channel}.wav"))).unwrap();
                let expected: Vec<u8> = (start..end)
                    .flat_map(|frame| [frame, channel as u8 - 1, 0x7f])
                    .collect();
                assert_eq!(&bytes[44..44 + expected.len()], expected);
                assert_eq!(
                    u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize,
                    expected.len()
                );
                assert_eq!(bytes.len(), 44 + expected.len() + expected.len() % 2);
                if expected.len() % 2 == 1 {
                    assert_eq!(bytes.last(), Some(&0));
                }
            }
        }
    }
}

#[test]
/// Reject invalid channels and intervals before creating an output directory.
fn invalid_requests_do_not_create_output() {
    let fixture = Fixture::new();
    let session = Session::open(&fixture.0).unwrap();
    let output = fixture.0.join("invalid");
    for (channels, interval) in [
        (Channels::One(0), Interval::Full),
        (Channels::One(3), Interval::Full),
        (
            Channels::All,
            Interval::Samples(std::ops::Range { start: 5, end: 4 }),
        ),
        (Channels::All, Interval::Samples(3..3)),
        (Channels::All, Interval::Samples(0..7)),
        (Channels::All, Interval::Markers(0..2)),
        (Channels::All, Interval::Seconds(0..u64::MAX)),
    ] {
        assert!(session.extract(channels, interval, &output).is_err());
        assert!(!output.exists());
    }
}

#[test]
/// Preserve channel bytes across a full input buffer and the final short block.
fn streams_frame_aligned_blocks_and_a_final_partial_block() {
    let fixture = Fixture::new();
    let frames = (4 * 1024 * 1024 / 12) + 2_u32;
    let mut log = Vec::new();
    for word in [0x1234abcd, 3, 48_000, 0, 1, 0, frames, frames * 3] {
        log.extend_from_slice(&word.to_le_bytes());
    }
    log.resize((7 + 256) * 4, 0);
    std::fs::write(fixture.0.join("SE_LOG.BIN"), log).unwrap();
    let mut take = vec![0; 32 * 1024];
    for frame in 0..frames {
        for channel in 0..3 {
            take.extend_from_slice(&[0xee, frame as u8, channel, (frame >> 8) as u8]);
        }
    }
    std::fs::write(fixture.0.join("00000001.WAV"), take).unwrap();
    let session = Session::open(&fixture.0).unwrap();
    let output = fixture.0.join("streamed");
    session
        .extract(Channels::All, Interval::Full, &output)
        .unwrap();
    for channel in 0..3 {
        let bytes = std::fs::read(output.join(format!("ch_{}.wav", channel + 1))).unwrap();
        let expected: Vec<u8> = (0..frames)
            .flat_map(|frame| [frame as u8, channel, (frame >> 8) as u8])
            .collect();
        assert_eq!(&bytes[44..44 + expected.len()], expected);
    }
}

#[test]
/// Report truncated input and reject existing folders without overwriting their files.
fn truncated_audio_is_an_error_and_existing_output_is_preserved() {
    let fixture = Fixture::new();
    let session = Session::open(&fixture.0).unwrap();
    let output = fixture.0.join("existing");
    std::fs::create_dir(&output).unwrap();
    let existing = output.join("ch_1.wav");
    std::fs::write(&existing, b"keep this").unwrap();
    assert!(session
        .extract(Channels::All, Interval::Full, &output)
        .is_err());
    assert_eq!(std::fs::read(existing).unwrap(), b"keep this");

    std::fs::write(fixture.0.join("00000002.WAV"), [0; 32 * 1024]).unwrap();
    let error = session
        .extract(Channels::All, Interval::Full, fixture.0.join("truncated"))
        .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}
