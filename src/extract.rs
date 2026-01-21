use crate::helpers::*;
use std::fs::{File, OpenOptions};
use std::io::{Read, Result, Seek, SeekFrom, Write};
use std::time::Instant;

// TODO not working properly
pub fn name_session(name_str: &str) -> Result<()> {
    let mut log = if let Ok(file) = File::open("SE_LOG.BIN") {
        file
    } else {
        println!("log file not found!");
        return Ok(());
    };

    let mut log_copy = Vec::new();
    log.read_to_end(&mut log_copy)?;
    let mut log_copy_l = log_copy.clone();

    let name_p = 4 * 388;
    let name_len = name_str.len();

    // Copy the name into the log file, up to 19 characters
    for (i, &byte) in name_str.as_bytes().iter().enumerate().take(19) {
        log_copy_l[name_p + i] = byte;
    }
    // Pad the rest of the name with 0 if it's less than 19 characters
    for i in name_len..19 {
        log_copy_l[name_p + i] = 0;
    }
    // null terminate the string
    log_copy_l[name_p + 19] = 0;

    let mut log = OpenOptions::new().write(true).open("SE_LOG.BIN")?;
    log.write_all(&log_copy_l)?;
    println!("Session named successfully!");
    Ok(())
}

pub fn get_session_info() -> Result<()> {
    let log_data = read_log_file()?;
    if log_data.no_channels == 0 || log_data.sample_rate == 0 {
        return Ok(());
    }

    let total_length_bytes = log_data.total_length * 4;

    println!("session_str = {}", log_data.session_str);
    println!("no_channels = {}", log_data.no_channels);
    println!("sample_rate = {}", log_data.sample_rate);
    println!("no_takes = {}", log_data.no_takes);
    println!("no_markers = {}", log_data.no_markers);
    println!("Total audio length in bytes = {}", total_length_bytes);
    println!(
        "Total audio samples per channel = {}",
        log_data.total_length
    );
    println!(
        "Total audio time per channel = {}",
        log_data.total_length / log_data.sample_rate
    );

    for (i, &marker) in log_data
        .take_markers
        .iter()
        .enumerate()
        .take(log_data.no_markers as usize)
    {
        println!(
            "Marker {} at {} samples or {} seconds",
            i,
            marker,
            marker / log_data.sample_rate
        );
    }
    Ok(())
}

pub fn extract_session() -> Result<()> {
    let start = Instant::now();

    let log_data = read_log_file()?;
    if log_data.no_channels == 0 || log_data.sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;

    let folder_name = format!("session_{}", log_data.session_str);
    if let Err(e) = std::fs::create_dir(&folder_name) {
        eprintln!("Error creating folder '{}': {}", folder_name, e);
        return Err(e);
    }

    let mut waves = create_waves(
        &format!("session_{}", log_data.session_str),
        log_data.total_length,
        log_data.sample_rate,
        log_data.no_channels,
    )?;
    println!("Unpacking audio data, this may take a while :) \n");

    let mut take = Vec::new();
    for i in 0..log_data.no_takes {
        open_take(i as usize, &mut take, &log_data.take_size)?;

        take[i as usize].seek(SeekFrom::Start(32 * 1024))?;
        read_write_audio(
            &mut take[i as usize],
            log_data.take_size[i as usize],
            buf_size,
            log_data.no_channels,
            &mut waves,
        )?;
    }

    close_waves(waves)?;
    for t in take {
        t.sync_all()?;
    }

    let end = Instant::now();
    println!(
        "process completed in={}sec",
        end.duration_since(start).as_secs()
    );
    Ok(())
}

pub fn extract_channel(channel_no: u32) -> Result<()> {
    let start = Instant::now();

    let log_data = read_log_file()?;
    if log_data.no_channels == 0 || log_data.sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;

    let folder_name = format!("channel_{}_{}", channel_no, log_data.session_str);
    if let Err(e) = std::fs::create_dir(&folder_name) {
        eprintln!("Error creating folder '{}': {}", folder_name, e);
        return Err(e);
    }

    let mut wave = create_wave(
        &format!("channel_{}_{}", channel_no, log_data.session_str),
        log_data.total_length,
        log_data.sample_rate,
        channel_no,
    )?;
    println!("Unpacking audio data, this may take a while :) \n");

    let mut take = Vec::new();
    for i in 0..log_data.no_takes {
        open_take(i as usize, &mut take, &log_data.take_size)?;

        take[i as usize].seek(SeekFrom::Start(32 * 1024))?;
        read_write_audio_ch(
            &mut take[i as usize],
            log_data.take_size[i as usize],
            buf_size,
            log_data.no_channels,
            &mut wave,
            channel_no,
        )?;
    }

    wave.sync_all()?;
    for t in take {
        t.sync_all()?;
    }

    let end = Instant::now();
    println!(
        "process completed in={}sec",
        end.duration_since(start).as_secs()
    );
    Ok(())
}

pub fn extract_session_marker(start_marker: u32, stop_marker: u32) -> Result<()> {
    let start = Instant::now();

    let log_data = read_log_file()?;
    if log_data.no_channels == 0 || log_data.sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;
    let folder_name = format!(
        "session_marker_{}_{}_{}",
        start_marker, stop_marker, log_data.session_str
    );

    if let Err(e) = std::fs::create_dir(&folder_name) {
        eprintln!("Error creating folder '{}': {}", folder_name, e);
        return Err(e);
    }

    let mut waves = create_waves(
        &folder_name,
        log_data.total_length,
        log_data.sample_rate,
        log_data.no_channels,
    )?;
    println!("Unpacking audio data, this may take a while :) \n");

    let (start_take, end_take, s_time_x_ch, e_time_x_ch) = calc_limits_marker(
        start_marker.try_into().unwrap(),
        stop_marker.try_into().unwrap(),
        &log_data,
    );

    let mut take = Vec::new();
    for i in start_take..=end_take {
        open_take(i as usize, &mut take, &log_data.take_size)?;

        take[i as usize].seek(SeekFrom::Start(32 * 1024))?;
        let l_takesize = calc_take_len(
            i,
            start_take,
            &mut take,
            &log_data.take_size,
            end_take,
            s_time_x_ch,
            e_time_x_ch,
        )?;
        read_write_audio(
            &mut take[i as usize],
            l_takesize,
            buf_size,
            log_data.no_channels,
            &mut waves,
        )?;
    }

    close_waves(waves)?;
    for t in take {
        t.sync_all()?;
    }

    let end = Instant::now();
    println!(
        "process completed in={}sec",
        end.duration_since(start).as_secs()
    );
    Ok(())
}

pub fn extract_channel_marker(channel_no: u32, start_marker: u32, stop_marker: u32) -> Result<()> {
    let start = Instant::now();

    let log_data = read_log_file()?;
    if log_data.no_channels == 0 || log_data.sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;
    let folder_name = format!(
        "channel_marker_{}_{}_{}_{}",
        start_marker, stop_marker, channel_no, log_data.session_str
    );

    if let Err(e) = std::fs::create_dir(&folder_name) {
        eprintln!("Error creating folder '{}': {}", folder_name, e);
        return Err(e);
    }

    let mut wave = create_wave(
        &folder_name,
        log_data.total_length,
        log_data.sample_rate,
        channel_no,
    )?;
    println!("Unpacking audio data, this may take a while :) \n");

    let (start_take, end_take, s_time_x_ch, e_time_x_ch) = calc_limits_marker(
        start_marker.try_into().unwrap(),
        stop_marker.try_into().unwrap(),
        &log_data,
    );

    let mut take = Vec::new();
    for i in start_take..=end_take {
        open_take(i as usize, &mut take, &log_data.take_size)?;

        take[i as usize].seek(SeekFrom::Start(32 * 1024))?;
        let l_takesize = calc_take_len(
            i,
            start_take,
            &mut take,
            &log_data.take_size,
            end_take,
            s_time_x_ch,
            e_time_x_ch,
        )?;
        read_write_audio_ch(
            &mut take[i as usize],
            l_takesize,
            buf_size,
            log_data.no_channels,
            &mut wave,
            channel_no,
        )?;
    }

    wave.sync_all()?;
    for t in take {
        t.sync_all()?;
    }

    let end = Instant::now();
    println!(
        "process completed in={}sec",
        end.duration_since(start).as_secs()
    );
    Ok(())
}

pub fn extract_session_time(start_time: u32, stop_time: u32) -> Result<()> {
    let start = Instant::now();

    let log_data = read_log_file()?;
    if log_data.no_channels == 0 || log_data.sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;
    let folder_name = format!(
        "session_time_{}_{}_{}",
        start_time, stop_time, log_data.session_str
    );

    if let Err(e) = std::fs::create_dir(&folder_name) {
        eprintln!("Error creating folder '{}': {}", folder_name, e);
        return Err(e);
    }

    let mut waves = create_waves(
        &folder_name,
        log_data.total_length,
        log_data.sample_rate,
        log_data.no_channels,
    )?;
    println!("Unpacking audio data, this may take a while :) \n");

    let (start_take, end_take, s_time_x_ch, e_time_x_ch) =
        calc_limits_time(start_time, stop_time, &log_data);

    let mut take = Vec::new();
    for i in start_take..=end_take {
        open_take(i as usize, &mut take, &log_data.take_size)?;

        take[i as usize].seek(SeekFrom::Start(32 * 1024))?;
        let l_takesize = calc_take_len(
            i,
            start_take,
            &mut take,
            &log_data.take_size,
            end_take,
            s_time_x_ch,
            e_time_x_ch,
        )?;
        read_write_audio(
            &mut take[i as usize],
            l_takesize,
            buf_size,
            log_data.no_channels,
            &mut waves,
        )?;
    }

    close_waves(waves)?;
    for t in take {
        t.sync_all()?;
    }

    let end = Instant::now();
    println!(
        "process completed in={}sec",
        end.duration_since(start).as_secs()
    );
    Ok(())
}

pub fn extract_channel_time(channel_no: u32, start_time: u32, stop_time: u32) -> Result<()> {
    let start = Instant::now();

    let log_data = read_log_file()?;
    if log_data.no_channels == 0 || log_data.sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;
    let folder_name = format!(
        "channel_time_{}_{}_{}_{}",
        start_time, stop_time, channel_no, log_data.session_str
    );

    if let Err(e) = std::fs::create_dir(&folder_name) {
        eprintln!("Error creating folder '{}': {}", folder_name, e);
        return Err(e);
    }

    let mut wave = create_wave(
        &folder_name,
        log_data.total_length,
        log_data.sample_rate,
        channel_no,
    )?;
    println!("Unpacking audio data, this may take a while :) \n");

    let (start_take, end_take, s_time_x_ch, e_time_x_ch) =
        calc_limits_time(start_time, stop_time, &log_data);

    let mut take = Vec::new();
    for i in start_take..=end_take {
        open_take(i as usize, &mut take, &log_data.take_size)?;

        take[i as usize].seek(SeekFrom::Start(32 * 1024))?;
        let l_takesize = calc_take_len(
            i,
            start_take,
            &mut take,
            &log_data.take_size,
            end_take,
            s_time_x_ch,
            e_time_x_ch,
        )?;
        read_write_audio_ch(
            &mut take[i as usize],
            l_takesize,
            buf_size,
            log_data.no_channels,
            &mut wave,
            channel_no,
        )?;
    }

    wave.sync_all()?;
    for t in take {
        t.sync_all()?;
    }

    let end = Instant::now();
    println!(
        "process completed in={}sec",
        end.duration_since(start).as_secs()
    );
    Ok(())
}
