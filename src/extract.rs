use crate::helpers::*;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, Write};
use std::time::Instant;

pub fn name_session(name_str: &str) -> io::Result<()> {
    let mut log = match File::open("se_log.bin") {
        Ok(file) => file,
        Err(_) => {
            println!("log file not found!");
            return Ok(());
        }
    };

    let mut log_copy = Vec::new();
    log.read_to_end(&mut log_copy)?;
    let mut log_copy_l = log_copy.clone();

    let name_p = 4 * 388;
    let name_len = name_str.len();

    for i in 0..19 {
        if i < name_len {
            log_copy_l[name_p + i] = name_str.as_bytes()[i];
        } else {
            log_copy_l[name_p + i] = 0;
        }
    }

    log_copy_l[name_p + 19] = 0;

    let mut log = OpenOptions::new().write(true).open("se_log.bin")?;
    log.write_all(&log_copy_l)?;
    println!("Session named successfully!");
    Ok(())
}

pub fn get_session_info() -> io::Result<()> {
    let (
        _session_str,
        _session_no,
        no_channels,
        sample_rate,
        _date_code,
        no_takes,
        no_markers,
        total_length,
        _take_size,
        take_markers,
    ) = read_log_file()?;
    if no_channels == 0 || sample_rate == 0 {
        return Ok(());
    }

    let total_length_bytes = total_length * 4;

    println!("no_channels = {}", no_channels);
    println!("sample_rate = {}", sample_rate);
    println!("no_takes = {}", no_takes);
    println!("no_markers = {}", no_markers);
    println!("Total audio length in bytes = {}", total_length_bytes);
    println!("Total audio samples per channel = {}", total_length);
    println!(
        "Total audio time per channel = {}",
        total_length / sample_rate
    );

    for i in 0..no_markers {
        println!(
            "Marker {} at {} samples or {} seconds",
            i,
            take_markers[i as usize],
            take_markers[i as usize] / sample_rate
        );
    }
    Ok(())
}

pub fn extract_session() -> io::Result<()> {
    let start = Instant::now();

    let (
        session_str,
        _session_no,
        no_channels,
        sample_rate,
        _date_code,
        no_takes,
        _no_markers,
        total_length,
        take_size,
        _take_markers,
    ) = read_log_file()?;
    if no_channels == 0 || sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;

    match std::fs::create_dir(format!("session_{}", session_str)) {
        Ok(_) => (),
        Err(_) => {
            println!("please remove existing folder");
            return Ok(());
        }
    }

    let mut waves = create_waves(
        &format!("session_{}", session_str),
        total_length,
        sample_rate,
        no_channels,
    )?;
    println!("Unpacking audio data, this may take a while :) \n");

    let mut take = Vec::new();
    for i in 0..no_takes {
        open_take(i as usize, &mut take, &take_size)?;

        take[i as usize].seek(io::SeekFrom::Start(32 * 1024))?;
        read_write_audio(
            &mut take[i as usize],
            take_size[i as usize],
            buf_size,
            no_channels,
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

pub fn extract_channel(channel_no: u32) -> io::Result<()> {
    let start = Instant::now();

    let (
        session_str,
        _session_no,
        no_channels,
        sample_rate,
        _date_code,
        no_takes,
        _no_markers,
        total_length,
        take_size,
        _take_markers,
    ) = read_log_file()?;
    if no_channels == 0 || sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;

    match std::fs::create_dir(format!("channel_{}_{}", channel_no, session_str)) {
        Ok(_) => (),
        Err(_) => {
            println!("please remove existing folder");
            return Ok(());
        }
    }

    let mut wave = create_wave(
        &format!("channel_{}_{}", channel_no, session_str),
        total_length,
        sample_rate,
        channel_no,
    )?;
    println!("Unpacking audio data, this may take a while :) \n");

    let mut take = Vec::new();
    for i in 0..no_takes {
        open_take(i as usize, &mut take, &take_size)?;

        take[i as usize].seek(io::SeekFrom::Start(32 * 1024))?;
        read_write_audio_ch(
            &mut take[i as usize],
            take_size[i as usize],
            buf_size,
            no_channels,
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

pub fn extract_session_marker(start_marker: u32, stop_marker: u32) -> io::Result<()> {
    let start = Instant::now();

    let (
        session_str,
        _session_no,
        no_channels,
        sample_rate,
        _date_code,
        no_takes,
        _no_markers,
        total_length,
        take_size,
        take_markers,
    ) = read_log_file()?;
    if no_channels == 0 || sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;
    let folder_name = format!(
        "session_marker_{}_{}_{}",
        start_marker, stop_marker, session_str
    );

    match std::fs::create_dir(&folder_name) {
        Ok(_) => (),
        Err(_) => {
            println!("please remove existing folder");
            return Ok(());
        }
    }

    let mut waves = create_waves(&folder_name, total_length, sample_rate, no_channels)?;
    println!("Unpacking audio data, this may take a while :) \n");

    let (start_take, end_take, s_time_x_ch, e_time_x_ch) = calc_limits_marker(
        start_marker.try_into().unwrap(),
        stop_marker.try_into().unwrap(),
        no_takes,
        &take_size,
        &take_markers,
        no_channels,
    );

    let mut take = Vec::new();
    for i in start_take..=end_take {
        open_take(i as usize, &mut take, &take_size)?;

        take[i as usize].seek(io::SeekFrom::Start(32 * 1024))?;
        let l_takesize = calc_take_len(
            i,
            start_take,
            &mut take, // Change this line to pass a mutable reference
            &take_size,
            end_take,
            s_time_x_ch,
            e_time_x_ch,
        )?;
        read_write_audio(
            &mut take[i as usize],
            l_takesize,
            buf_size,
            no_channels,
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

pub fn extract_channel_marker(
    channel_no: u32,
    start_marker: u32,
    stop_marker: u32,
) -> io::Result<()> {
    let start = Instant::now();

    let (
        session_str,
        _session_no,
        no_channels,
        sample_rate,
        _date_code,
        no_takes,
        _no_markers,
        total_length,
        take_size,
        take_markers,
    ) = read_log_file()?;
    if no_channels == 0 || sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;
    let folder_name = format!(
        "channel_marker_{}_{}_{}_{}",
        start_marker, stop_marker, channel_no, session_str
    );

    match std::fs::create_dir(&folder_name) {
        Ok(_) => (),
        Err(_) => {
            println!("please remove existing folder");
            return Ok(());
        }
    }

    let mut wave = create_wave(&folder_name, total_length, sample_rate, channel_no)?;
    println!("Unpacking audio data, this may take a while :) \n");

    let (start_take, end_take, s_time_x_ch, e_time_x_ch) = calc_limits_marker(
        start_marker.try_into().unwrap(),
        stop_marker.try_into().unwrap(),
        no_takes,
        &take_size,
        &take_markers,
        no_channels,
    );

    let mut take = Vec::new();
    for i in start_take..=end_take {
        open_take(i as usize, &mut take, &take_size)?;

        take[i as usize].seek(io::SeekFrom::Start(32 * 1024))?;
        let l_takesize = calc_take_len(
            i,
            start_take,
            &mut take, // Change this line to pass a mutable reference
            &take_size,
            end_take,
            s_time_x_ch,
            e_time_x_ch,
        )?;
        read_write_audio_ch(
            &mut take[i as usize],
            l_takesize,
            buf_size,
            no_channels,
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

pub fn extract_session_time(start_time: u32, stop_time: u32) -> io::Result<()> {
    let start = Instant::now();

    let (
        session_str,
        _session_no,
        no_channels,
        sample_rate,
        _date_code,
        no_takes,
        _no_markers,
        total_length,
        take_size,
        _take_markers,
    ) = read_log_file()?;
    if no_channels == 0 || sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;
    let folder_name = format!("session_time_{}_{}_{}", start_time, stop_time, session_str);

    match std::fs::create_dir(&folder_name) {
        Ok(_) => (),
        Err(_) => {
            println!("please remove existing folder");
            return Ok(());
        }
    }

    let mut waves = create_waves(&folder_name, total_length, sample_rate, no_channels)?;
    println!("Unpacking audio data, this may take a while :) \n");

    let (start_take, end_take, s_time_x_ch, e_time_x_ch) = calc_limits_time(
        start_time,
        stop_time,
        no_takes,
        &take_size,
        sample_rate,
        no_channels,
    );

    let mut take = Vec::new();
    for i in start_take..=end_take {
        open_take(i as usize, &mut take, &take_size)?;

        take[i as usize].seek(io::SeekFrom::Start(32 * 1024))?;
        let l_takesize = calc_take_len(
            i,
            start_take,
            &mut take, // Change this line to pass a mutable reference
            &take_size,
            end_take,
            s_time_x_ch,
            e_time_x_ch,
        )?;
        read_write_audio(
            &mut take[i as usize],
            l_takesize,
            buf_size,
            no_channels,
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

pub fn extract_channel_time(channel_no: u32, start_time: u32, stop_time: u32) -> io::Result<()> {
    let start = Instant::now();

    let (
        session_str,
        _session_no,
        no_channels,
        sample_rate,
        _date_code,
        no_takes,
        _no_markers,
        total_length,
        take_size,
        _take_markers,
    ) = read_log_file()?;
    if no_channels == 0 || sample_rate == 0 {
        return Ok(());
    }

    let buf_size = 1024 * 1024 * 4;
    let folder_name = format!(
        "channel_time_{}_{}_{}_{}",
        start_time, stop_time, channel_no, session_str
    );

    match std::fs::create_dir(&folder_name) {
        Ok(_) => (),
        Err(_) => {
            println!("please remove existing folder");
            return Ok(());
        }
    }

    let mut wave = create_wave(&folder_name, total_length, sample_rate, channel_no)?;
    println!("Unpacking audio data, this may take a while :) \n");

    let (start_take, end_take, s_time_x_ch, e_time_x_ch) = calc_limits_time(
        start_time,
        stop_time,
        no_takes,
        &take_size,
        sample_rate,
        no_channels,
    );

    let mut take = Vec::new();
    for i in start_take..=end_take {
        open_take(i as usize, &mut take, &take_size)?;

        take[i as usize].seek(io::SeekFrom::Start(32 * 1024))?;
        let l_takesize = calc_take_len(
            i,
            start_take,
            &mut take, // Change this line to pass a mutable reference
            &take_size,
            end_take,
            s_time_x_ch,
            e_time_x_ch,
        )?;
        read_write_audio_ch(
            &mut take[i as usize],
            l_takesize,
            buf_size,
            no_channels,
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
