use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub fn read_log_file() -> io::Result<(
    String,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    Vec<u32>,
    Vec<u32>,
)> {
    let mut log = File::open("se_log.bin")?;
    let mut buffer = [0; 4];

    log.read_exact(&mut buffer)?;
    let session_str = hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>());
    let session_no = u32::from_str_radix(&session_str, 16).unwrap();

    log.read_exact(&mut buffer)?;
    let no_channels = u32::from_str_radix(
        &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
        16,
    )
    .unwrap();

    log.read_exact(&mut buffer)?;
    let sample_rate = u32::from_str_radix(
        &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
        16,
    )
    .unwrap();

    log.read_exact(&mut buffer)?;
    let date_code = u32::from_str_radix(
        &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
        16,
    )
    .unwrap();

    log.read_exact(&mut buffer)?;
    let no_takes = u32::from_str_radix(
        &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
        16,
    )
    .unwrap();

    log.read_exact(&mut buffer)?;
    let no_markers = u32::from_str_radix(
        &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
        16,
    )
    .unwrap();

    log.read_exact(&mut buffer)?;
    let total_length = u32::from_str_radix(
        &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
        16,
    )
    .unwrap();

    let mut take_size = Vec::new();
    for _ in 0..no_takes {
        log.read_exact(&mut buffer)?;
        take_size.push(
            u32::from_str_radix(
                &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
                16,
            )
            .unwrap(),
        );
    }

    for _ in 0..(256 - no_takes) {
        log.read_exact(&mut buffer)?;
    }

    let mut take_markers = Vec::new();
    for _ in 0..no_markers {
        log.read_exact(&mut buffer)?;
        take_markers.push(
            u32::from_str_radix(
                &hex::encode(buffer.iter().rev().cloned().collect::<Vec<u8>>()),
                16,
            )
            .unwrap(),
        );
    }

    Ok((
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
    ))
}

pub fn create_waves(
    folder: &str,
    no_samples: u32,
    sample_rate: u32,
    no_waves: u32,
) -> io::Result<Vec<File>> {
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

pub fn close_waves(waves: Vec<File>) -> io::Result<()> {
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
) -> io::Result<File> {
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
) -> io::Result<()> {
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
) -> io::Result<()> {
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
    take_size: &Vec<u32>,
    end_take: usize,
    s_time_x_ch: u32,
    e_time_x_ch: u32,
) -> io::Result<u32> {
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

pub fn open_take(i: usize, take: &mut Vec<File>, take_size: &Vec<u32>) -> io::Result<()> {
    let filename = if i + 1 < 10 {
        format!("0000000{}.wav", i + 1)
    } else if i + 1 < 100 {
        format!("000000{}.wav", i + 1)
    } else {
        format!("00000{}.wav", i + 1)
    };

    let file = File::open(&filename)?;
    println!("reading take {} \n", i + 1);
    println!("take length {} \n", take_size[i]);
    take.push(file);

    Ok(())
}

pub fn calc_limits_time(
    start_time: u32,
    stop_time: u32,
    no_takes: u32,
    take_size: &Vec<u32>,
    sample_rate: u32,
    no_channels: u32,
) -> (usize, usize, u32, u32) {
    let s_time_x_ch = start_time * sample_rate * no_channels;
    let mut time_compare = 0;
    let start_take = (0..no_takes)
        .find(|&i| {
            time_compare += take_size[i as usize];
            s_time_x_ch < time_compare
        })
        .unwrap_or(0) as usize;

    let e_time_x_ch = stop_time * sample_rate * no_channels;
    time_compare = 0;
    let end_take = (0..no_takes)
        .find(|&i| {
            time_compare += take_size[i as usize];
            e_time_x_ch < time_compare
        })
        .unwrap_or(0) as usize;

    (start_take, end_take, s_time_x_ch, e_time_x_ch)
}

pub fn calc_limits_marker(
    start_marker: usize,
    stop_marker: usize,
    no_takes: u32,
    take_size: &Vec<u32>,
    take_markers: &Vec<u32>,
    no_channels: u32,
) -> (usize, usize, u32, u32) {
    let s_time_x_ch = take_markers[start_marker] * no_channels;
    let mut time_compare = 0;
    let start_take = (0..no_takes)
        .find(|&i| {
            time_compare += take_size[i as usize];
            s_time_x_ch < time_compare
        })
        .unwrap_or(0) as usize;

    let e_time_x_ch = take_markers[stop_marker] * no_channels;
    time_compare = 0;
    let end_take = (0..no_takes)
        .find(|&i| {
            time_compare += take_size[i as usize];
            e_time_x_ch < time_compare
        })
        .unwrap_or(0) as usize;

    (start_take, end_take, s_time_x_ch, e_time_x_ch)
}

pub fn create_header(
    folder: &str,
    no_samples: u32,
    no_channels: u32,
    sample_rate: u32,
    session_no_str: &str,
) -> io::Result<File> {
    let size = no_samples * 4 * no_channels;
    let path = format!("{}/session{}.wav", folder, session_no_str);
    let mut file = File::create(&path)?;
    file.write_all(b"RIFF")?;
    file.write_all(&u32::to_le_bytes(size + 44 + 460))?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&u32::to_le_bytes(16))?;
    file.write_all(&u16::to_le_bytes(1))?;
    file.write_all(&u16::to_le_bytes(no_channels as u16))?;
    file.write_all(&u32::to_le_bytes(sample_rate))?;
    file.write_all(&u32::to_le_bytes(sample_rate * no_channels * 4))?;
    file.write_all(&u16::to_le_bytes((no_channels * 4) as u16))?;
    file.write_all(&u16::to_le_bytes(32))?;
    file.write_all(b"JUNK")?;
    file.write_all(&u32::to_le_bytes(460))?;
    for _ in 0..460 {
        file.write_all(b" ")?;
    }
    file.write_all(b"data")?;
    file.write_all(&u32::to_le_bytes(size))?;
    Ok(file)
}
