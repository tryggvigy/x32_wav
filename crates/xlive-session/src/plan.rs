//! Resolve user selections into take-local spans without accessing files.

use crate::{invalid_input, Metadata};
use std::io;
use std::ops::Range;

/// Select all recorded channels or one recorder channel.
///
/// [`crate::Session::extract`] checks the selection against the session metadata.
#[derive(Clone, Copy, Debug)]
pub enum Channels {
    /// Extract every channel in recorder order.
    All,
    /// Extract one channel by its one-based recorder number.
    ///
    /// The number must be between one and the session channel count, inclusive.
    One(u32),
}

/// Select a session interval by time, markers, or sample positions.
///
/// Audio intervals include the start sample and exclude the end sample.
/// [`crate::Session::extract`] rejects empty, reversed, or out-of-session intervals.
#[derive(Clone, Debug)]
pub enum Interval {
    /// Extract the declared session length, excluding any excess take data.
    Full,
    /// Extract between whole-second offsets relative to the session start.
    ///
    /// Both endpoints are multiplied by the session sample rate.
    Seconds(Range<u64>),
    /// Extract audio between two zero-based marker indices.
    ///
    /// Both indices must exist. The range stores endpoint indices, not a slice
    /// of markers. For `0..1`, audio starts at marker zero and stops immediately
    /// before the sample at marker one. Ordering depends on marker positions.
    Markers(Range<usize>),
    /// Extract between frame positions relative to the session start.
    ///
    /// A frame contains one sample per channel, independent of the channel count.
    Samples(Range<u64>),
}

/// The nonempty overlap between a requested interval and one take file.
pub(crate) struct TakeSpan {
    /// Zero-based index of the take in the metadata.
    pub index: usize,
    /// Frame offset from the start of this take's audio, excluding its header.
    pub offset_samples: u64,
    /// Number of frames to read from this take.
    pub sample_count: u64,
}

/// A validated extraction request with channels and take-local frame spans.
pub(crate) struct Plan {
    /// One-based recorder channel numbers in output order.
    pub channels: Vec<u32>,
    /// Total output frames per channel, used to size each WAV header.
    pub sample_count: u64,
    /// Nonempty spans in recording order whose lengths sum to `sample_count`.
    pub takes: Vec<TakeSpan>,
}

impl Metadata {
    /// Resolve a selection into channel numbers and take-local frame spans.
    ///
    /// This calculation performs no I/O. A range ending at a take boundary does
    /// not include an empty span for the next take.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::InvalidInput`] for an invalid channel or marker,
    /// time conversion overflow, or an empty, reversed, or out-of-session range.
    pub(crate) fn plan(&self, channels: Channels, interval: Interval) -> io::Result<Plan> {
        let channels = match channels {
            Channels::All => (1..=self.channel_count).collect(),
            Channels::One(channel) if (1..=self.channel_count).contains(&channel) => vec![channel],
            Channels::One(_) => return Err(invalid_input("channel is outside the session")),
        };
        let range = match interval {
            Interval::Full => 0..self.total_samples,
            Interval::Samples(range) => range,
            Interval::Seconds(range) => {
                let to_samples = |seconds: u64| {
                    seconds
                        .checked_mul(u64::from(self.sample_rate))
                        .ok_or_else(|| invalid_input("time is too large"))
                };
                to_samples(range.start)?..to_samples(range.end)?
            }
            Interval::Markers(range) => {
                let marker = |index| {
                    self.markers
                        .get(index)
                        .copied()
                        .ok_or_else(|| invalid_input("marker is outside the session"))
                };
                marker(range.start)?..marker(range.end)?
            }
        };
        if range.start >= range.end || range.end > self.total_samples {
            return Err(invalid_input(
                "sample range must be nonempty and inside the session",
            ));
        }
        let mut takes = Vec::new();
        let mut take_start = 0;
        for (index, &samples) in self.take_samples.iter().enumerate() {
            let take_end = take_start + samples;
            let start = range.start.max(take_start);
            let end = range.end.min(take_end);
            if start < end {
                takes.push(TakeSpan {
                    index,
                    offset_samples: start - take_start,
                    sample_count: end - start,
                });
            }
            take_start = take_end;
        }
        Ok(Plan {
            channels,
            sample_count: range.end - range.start,
            takes,
        })
    }
}
