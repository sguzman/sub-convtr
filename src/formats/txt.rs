use anyhow::{Result, anyhow};

use crate::{
    config::Config,
    formats::time::{format_vtt_timestamp, parse_time_range_arrow},
    model::{Cue, Transcript},
};

pub fn write_txt(t: &Transcript, cfg: &Config) -> String {
    let mode = cfg.formats.txt.mode.to_lowercase();
    let mut out = String::new();

    for cue in &t.cues {
        if mode == "text_only" {
            out.push_str(cue.text.trim());
            out.push('\n');
        } else {
            out.push_str(&format!(
                "[{} --> {}] ",
                format_vtt_timestamp(cue.start_ms + cfg.policy.timestamp_offset_ms),
                format_vtt_timestamp(cue.end_ms + cfg.policy.timestamp_offset_ms),
            ));
            out.push_str(cue.text.trim());
            out.push('\n');
        }
    }

    out
}

pub fn parse_txt(input: &str, cfg: &Config) -> Result<Transcript> {
    let mut cues: Vec<Cue> = Vec::new();
    let mut cursor_ms: i64 = 0;

    for (line_no, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix('[') {
            if let Some((range, text_part)) = rest.split_once(']') {
                let (start, end) = parse_time_range_arrow(range.trim())?;
                let text = text_part.trim().to_string();
                cues.push(Cue {
                    start_ms: start,
                    end_ms: end,
                    text,
                    speaker: None,
                });
                continue;
            }
        }

        if !cfg.policy.synthesize_timings {
            return Err(anyhow!(
                "TXT line {} has no timestamps and synthesize_timings=false",
                line_no + 1
            ));
        }

        let dur = synth_duration_ms(line, cfg);
        let start = cursor_ms;
        let end = cursor_ms + dur;

        cues.push(Cue {
            start_ms: start,
            end_ms: end,
            text: line.to_string(),
            speaker: None,
        });

        cursor_ms = end + cfg.policy.gap_ms;
    }

    Ok(Transcript::new(cues))
}

fn synth_duration_ms(text: &str, cfg: &Config) -> i64 {
    let cps = cfg.policy.chars_per_second.max(1.0);
    let raw = (text.chars().count() as f64 / cps * 1000.0).round() as i64;
    raw.clamp(cfg.policy.min_duration_ms, cfg.policy.max_duration_ms)
}
