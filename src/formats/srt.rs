use crate::{config::Config, formats::time::format_srt_timestamp, model::Transcript};
use textwrap::wrap;

pub fn write_srt(t: &Transcript, cfg: &Config) -> String {
    let mut out = String::new();

    for (i, cue) in t.cues.iter().enumerate() {
        out.push_str(&(i + 1).to_string());
        out.push('\n');

        out.push_str(&format!(
            "{} --> {}\n",
            format_srt_timestamp(cue.start_ms + cfg.policy.timestamp_offset_ms),
            format_srt_timestamp(cue.end_ms + cfg.policy.timestamp_offset_ms)
        ));

        let text = cue_text_for_export(cue.text.as_str(), cfg);
        for line in wrap(&text, cfg.formats.srt.wrap_width) {
            out.push_str(&line);
            out.push('\n');
        }

        out.push('\n');
    }

    out
}

fn cue_text_for_export(text: &str, cfg: &Config) -> String {
    let mut s = text.to_string();
    if cfg.policy.trim_text {
        s = s.trim().to_string();
    }
    if cfg.policy.normalize_whitespace {
        s = normalize_ws(&s);
    }
    s
}

fn normalize_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        let is_ws = ch.is_whitespace();
        if is_ws {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}
