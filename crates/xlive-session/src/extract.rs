//! Execute validated extraction plans and finalize the mono WAV outputs.

use crate::audio::{copy_audio, Output};
use crate::{Channels, Interval, Session};
use pcm_wav::Mono24Writer;
use std::fs::File;
use std::io::{self, BufWriter, Seek, SeekFrom};
use std::path::Path;

impl Session {
    /// Extract selected audio into a new directory of mono, 24-bit WAV files.
    ///
    /// `channels` uses one-based recorder numbers. `interval` includes its start
    /// sample and excludes its end sample. Each file is named `ch_N.wav`, where
    /// `N` is the original channel number. Header lengths match the selected audio.
    ///
    /// `output` is used as given, relative to the process working directory if
    /// it is not absolute. Its parent must exist, and `output` itself must not exist.
    /// Selection and WAV-size checks run before directory creation.
    ///
    /// Extraction opens one take at a time and uses bounded, frame-aligned
    /// buffers. Successful completion includes final WAV padding and output
    /// flushing, but does not guarantee persistence to physical storage.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::InvalidInput`] for invalid selections, time
    /// overflow, or WAV sizes that exceed their 32-bit fields. Also returns file
    /// creation, read, seek, write, and flush errors. Truncated audio returns
    /// [`io::ErrorKind::UnexpectedEof`].
    ///
    /// Errors after directory creation can leave incomplete files. This method
    /// does not remove them. Existing output directories are rejected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xlive_session::{Channels, Interval, Session};
    ///
    /// let session = Session::open("recording")?;
    /// session.extract(Channels::All, Interval::Seconds(10..30), "excerpt")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn extract(
        &self,
        channels: Channels,
        interval: Interval,
        output: impl AsRef<Path>,
    ) -> io::Result<()> {
        let plan = self.metadata.plan(channels, interval)?;
        pcm_wav::header(plan.sample_count, self.metadata.sample_rate)?;
        let output = output.as_ref();
        std::fs::create_dir(output)?;
        let mut outputs = Vec::new();
        for channel in plan.channels {
            let file = File::create(output.join(format!("ch_{channel}.wav")))?;
            outputs.push(Output {
                channel_index: (channel - 1) as usize,
                writer: Mono24Writer::new(
                    BufWriter::new(file),
                    plan.sample_count,
                    self.metadata.sample_rate,
                )?,
                buffer: Vec::new(),
            });
        }
        for span in plan.takes {
            let mut take = File::open(self.root.join(format!("{:08}.WAV", span.index + 1)))?;
            let offset_bytes = span.offset_samples * u64::from(self.metadata.channel_count) * 4;
            take.seek(SeekFrom::Start(32 * 1024 + offset_bytes))?;
            copy_audio(
                &mut take,
                span.sample_count,
                self.metadata.channel_count,
                &mut outputs,
            )?;
        }
        for output in outputs {
            output.writer.finish()?;
        }
        Ok(())
    }
}
