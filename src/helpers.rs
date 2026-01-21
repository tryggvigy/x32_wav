use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom, Write};

pub struct LogData {
    pub session_str: String,
    pub session_no: u32,
    pub no_channels: u32,
    pub sample_rate: u32,
    pub date_code: u32,
    pub no_takes: u32,
    pub no_markers: u32,
    pub total_length: u32,
    pub take_size: Vec<u32>,
    pub take_markers: Vec<u32>,
}

pub fn read_log_file() -> Result<LogData> {
    let mut log = File::open("SE_LOG.BIN")?;
    let mut buffer = [0; 4];

    let read_u32 = |log: &mut File, buffer: &mut [u8; 4]| -> Result<u32> {
        log.read_exact(buffer)?;
        Ok(u32::from_str_radix(
            &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
            16,
        )
        .unwrap())
    };

    let session_str = hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>());
    let session_no = read_u32(&mut log, &mut buffer)?;
    let no_channels = read_u32(&mut log, &mut buffer)?;
    let sample_rate = read_u32(&mut log, &mut buffer)?;
    let date_code = read_u32(&mut log, &mut buffer)?;
    let no_takes = read_u32(&mut log, &mut buffer)?;
    let no_markers = read_u32(&mut log, &mut buffer)?;
    let total_length = read_u32(&mut log, &mut buffer)?;

    let take_size: Vec<u32> = (0..no_takes)
        .map(|_| read_u32(&mut log, &mut buffer))
        .collect::<Result<_>>()?;
    for _ in 0..(256 - no_takes) {
        log.read_exact(&mut buffer)?;
    }
    let take_markers: Vec<u32> = (0..no_markers)
        .map(|_| read_u32(&mut log, &mut buffer))
        .collect::<Result<_>>()?;

    Ok(LogData {
        session_str,
        session_no,
        no_channels,
        sample_rate,
        date_code,
        no_takes,
        no_markers,
        total_length,
        take_size,
        take_markers,
    })
}

pub fn create_waves(
    folder: &str,
    no_samples: u32,
    sample_rate: u32,
    no_waves: u32,
) -> Result<Vec<File>> {
    let bytes_datachunk = no_samples * 3;
    let mut channels = Vec::new();

    for i in 0..no_waves {
        let path = format!("{}/ch_{}.wav", folder, i + 1);
        let mut file = File::create(&path)?;
        file.write_all(b"RIFF")?;
        file.write_all(&u32::to_le_bytes(bytes_datachunk + 36))?;
        file.write_all(b"WAVEfmt ")?;
        file.write_all(&u32::to_le_bytes(16))?;
        file.write_all(&u16::to_le_bytes(1))?;
        file.write_all(&u16::to_le_bytes(1))?;
        file.write_all(&u32::to_le_bytes(sample_rate))?;
        file.write_all(&u32::to_le_bytes(sample_rate * 3))?;
        file.write_all(&u16::to_le_bytes(3))?;
        file.write_all(&u16::to_le_bytes(24))?;
        file.write_all(b"data")?;
        file.write_all(&u32::to_le_bytes(bytes_datachunk))?;
        channels.push(file);
    }

    Ok(channels)
}

pub fn close_waves(waves: Vec<File>) -> Result<()> {
    for wave in waves {
        wave.sync_all()?;
    }
    Ok(())
}

pub fn create_wave(
    folder: &str,
    no_samples: u32,
    sample_rate: u32,
    ch_number: u32,
) -> Result<File> {
    let bytes_datachunk = no_samples * 3;
    let path = format!("{}/ch_{}.wav", folder, ch_number);
    let mut file = File::create(&path)?;
    file.write_all(b"RIFF")?;
    file.write_all(&u32::to_le_bytes(bytes_datachunk + 36))?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&u32::to_le_bytes(16))?;
    file.write_all(&u16::to_le_bytes(1))?;
    file.write_all(&u16::to_le_bytes(1))?;
    file.write_all(&u32::to_le_bytes(sample_rate))?;
    file.write_all(&u32::to_le_bytes(sample_rate * 3))?;
    file.write_all(&u16::to_le_bytes(3))?;
    file.write_all(&u16::to_le_bytes(24))?;
    file.write_all(b"data")?;
    file.write_all(&u32::to_le_bytes(bytes_datachunk))?;
    Ok(file)
}

pub fn read_write_audio(
    take: &mut File,
    takesize: u32,
    bufsize: usize,
    no_channels: u32,
    waves_to: &mut Vec<File>,
) -> Result<()> {
    let mut read_buf = vec![0; bufsize];
    for _ in 0..(takesize * 4 / bufsize as u32) {
        take.read_exact(&mut read_buf)?;
        for j in 0..no_channels {
            let mut ch_buffer = Vec::new();
            for k in (0..bufsize).step_by((no_channels * 4) as usize) {
                let idx_read_buf = k + j as usize * 4;
                ch_buffer.extend_from_slice(&read_buf[idx_read_buf + 1..idx_read_buf + 4]);
            }
            waves_to[j as usize].write_all(&ch_buffer)?;
        }
    }

    let buf_size_rest = (takesize * 4 % bufsize as u32) as usize;
    read_buf.resize(buf_size_rest, 0);
    take.read_exact(&mut read_buf)?;
    for j in 0..no_channels {
        let mut ch_buffer = Vec::new();
        for k in (0..buf_size_rest).step_by((no_channels * 4) as usize) {
            let idx_read_buf = k + j as usize * 4;
            ch_buffer.extend_from_slice(&read_buf[idx_read_buf + 1..idx_read_buf + 4]);
        }
        waves_to[j as usize].write_all(&ch_buffer)?;
    }

    Ok(())
}

pub fn read_write_audio_ch(
    take: &mut File,
    takesize: u32,
    bufsize: usize,
    no_channels: u32,
    wave_to: &mut File,
    channel_no: u32,
) -> Result<()> {
    let mut read_buf = vec![0; bufsize];
    for _ in 0..(takesize * 4 / bufsize as u32) {
        take.read_exact(&mut read_buf)?;
        let mut ch_buffer = Vec::new();
        for k in (0..bufsize).step_by((no_channels * 4) as usize) {
            let idx_read_buf = k + (channel_no - 1) as usize * 4;
            ch_buffer.extend_from_slice(&read_buf[idx_read_buf + 1..idx_read_buf + 4]);
        }
        wave_to.write_all(&ch_buffer)?;
    }

    let buf_size_rest = (takesize * 4 % bufsize as u32) as usize;
    read_buf.resize(buf_size_rest, 0);
    take.read_exact(&mut read_buf)?;
    let mut ch_buffer = Vec::new();
    for k in (0..buf_size_rest).step_by((no_channels * 4) as usize) {
        let idx_read_buf = k + (channel_no - 1) as usize * 4;
        ch_buffer.extend_from_slice(&read_buf[idx_read_buf + 1..idx_read_buf + 4]);
    }
    wave_to.write_all(&ch_buffer)?;

    Ok(())
}

pub fn calc_take_len(
    i: usize,
    start_take: usize,
    take: &mut Vec<File>,
    take_size: &[u32],
    end_take: usize,
    s_time_x_ch: u32,
    e_time_x_ch: u32,
) -> Result<u32> {
    let l_takesize = if i == start_take {
        take[i].seek(SeekFrom::Current((s_time_x_ch * 4) as i64))?;
        if i == end_take {
            e_time_x_ch - s_time_x_ch
        } else {
            take_size[i] - s_time_x_ch
        }
    } else if i == end_take {
        e_time_x_ch
    } else {
        take_size[i]
    };

    Ok(l_takesize)
}

pub fn open_take(i: usize, take: &mut Vec<File>, take_size: &[u32]) -> Result<()> {
    let filename = format!("{:08}.WAV", i + 1);
    let file = File::open(&filename)?;
    println!("reading take {} \n", i + 1);
    println!("take length {} \n", take_size[i]);
    take.push(file);

    Ok(())
}

pub fn calc_limits_time(
    start_time: u32,
    stop_time: u32,
    log_data: &LogData,
) -> (usize, usize, u32, u32) {
    let s_time_x_ch = start_time * log_data.sample_rate * log_data.no_channels;
    let mut time_compare = 0;
    let start_take = (0..log_data.no_takes)
        .find(|&i| {
            time_compare += log_data.take_size[i as usize];
            s_time_x_ch < time_compare
        })
        .unwrap_or(0) as usize;

    let e_time_x_ch = stop_time * log_data.sample_rate * log_data.no_channels;
    time_compare = 0;
    let end_take = (0..log_data.no_takes)
        .find(|&i| {
            time_compare += log_data.take_size[i as usize];
            e_time_x_ch < time_compare
        })
        .unwrap_or(0) as usize;

    (start_take, end_take, s_time_x_ch, e_time_x_ch)
}

pub fn calc_limits_marker(
    start_marker: usize,
    stop_marker: usize,
    log_data: &LogData,
) -> (usize, usize, u32, u32) {
    let s_time_x_ch = log_data.take_markers[start_marker] * log_data.no_channels;
    let mut time_compare = 0;
    let start_take = (0..log_data.no_takes)
        .find(|&i| {
            time_compare += log_data.take_size[i as usize];
            s_time_x_ch < time_compare
        })
        .unwrap_or(0) as usize;

    let e_time_x_ch = log_data.take_markers[stop_marker] * log_data.no_channels;
    time_compare = 0;
    let end_take = (0..log_data.no_takes)
        .find(|&i| {
            time_compare += log_data.take_size[i as usize];
            e_time_x_ch < time_compare
        })
        .unwrap_or(0) as usize;

    (start_take, end_take, s_time_x_ch, e_time_x_ch)
}
