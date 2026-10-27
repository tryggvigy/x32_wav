//! Format validated session metadata for the command-line information command.

use xlive_session::Metadata;

/// Print the session identifier, format, lengths, and marker positions to stdout.
///
/// Sample counts are frames, and displayed seconds use integer division.
/// The legacy byte-length field reports four bytes per sample for one channel,
/// rather than the combined length of all channels.
///
/// # Panics
///
/// Panics if writing to standard output fails, as with [`println!`].
pub fn print(metadata: &Metadata) {
    println!("session_str = {:08x}", metadata.session_id());
    println!("no_channels = {}", metadata.channel_count());
    println!("sample_rate = {}", metadata.sample_rate());
    println!("no_takes = {}", metadata.take_samples().len());
    println!("no_markers = {}", metadata.markers().len());
    println!(
        "Total audio length in bytes = {}",
        metadata.total_samples() * 4
    );
    println!(
        "Total audio samples per channel = {}",
        metadata.total_samples()
    );
    println!(
        "Total audio time per channel = {}",
        metadata.total_samples() / u64::from(metadata.sample_rate())
    );
    for (index, marker) in metadata.markers().iter().enumerate() {
        println!(
            "Marker {} at {} samples or {} seconds",
            index,
            marker,
            marker / u64::from(metadata.sample_rate())
        );
    }
}
