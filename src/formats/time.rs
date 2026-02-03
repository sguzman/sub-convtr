use anyhow::{Result, anyhow};

pub fn format_srt_timestamp(ms: i64) -> String {
    format_timestamp(ms, ',', true)
}

pub fn format_vtt_timestamp(ms: i64) -> String {
    format_timestamp(ms, '.', false)
}

fn format_timestamp(ms_in: i64, ms_sep: char, _force_hours: bool) -> String {
    let ms = ms_in.max(0);

    let total_seconds = ms / 1000;
    let milli = (ms % 1000) as i64;

    let sec = (total_seconds % 60) as i64;
    let total_minutes = total_seconds / 60;
    let min = (total_minutes % 60) as i64;
    let hour = (total_minutes / 60) as i64;

    format!("{hour:02}:{min:02}:{sec:02}{ms_sep}{milli:03}")
}

pub fn parse_time_to_ms(s: &str) -> Result<i64> {
    let t = s.trim();

    if let Ok(v) = t.parse::<i64>() {
        return Ok(v);
    }

    if let Ok(v) = t.parse::<f64>() {
        let ms = (v * 1000.0).round() as i64;
        return Ok(ms);
    }

    let (hms, milli) = if let Some((a, b)) = t.split_once(',') {
        (a, Some(b))
    } else if let Some((a, b)) = t.split_once('.') {
        (a, Some(b))
    } else {
        (t, None)
    };

    let parts: Vec<&str> = hms.split(':').collect();
    if parts.len() != 3 {
        return Err(anyhow!("unrecognized timestamp: '{t}'"));
    }

    let h: i64 = parts[0].parse().map_err(|_| anyhow!("bad hours: '{t}'"))?;
    let m: i64 = parts[1]
        .parse()
        .map_err(|_| anyhow!("bad minutes: '{t}'"))?;
    let s2: i64 = parts[2]
        .parse()
        .map_err(|_| anyhow!("bad seconds: '{t}'"))?;

    let mut ms = ((h * 60 + m) * 60 + s2) * 1000;

    if let Some(frac) = milli {
        let mut frac_s = frac.trim().to_string();
        if frac_s.len() > 3 {
            frac_s.truncate(3);
        }
        while frac_s.len() < 3 {
            frac_s.push('0');
        }
        let milli: i64 = frac_s
            .parse()
            .map_err(|_| anyhow!("bad milliseconds: '{t}'"))?;
        ms += milli;
    }

    Ok(ms)
}

pub fn parse_time_range_arrow(line: &str) -> Result<(i64, i64)> {
    let (a, b) = line
        .split_once("-->")
        .ok_or_else(|| anyhow!("missing '-->' in time range: '{line}'"))?;
    let start = parse_time_to_ms(a.trim())?;
    let end = parse_time_to_ms(b.trim())?;
    Ok((start, end))
}
