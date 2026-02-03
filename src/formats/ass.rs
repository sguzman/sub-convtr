use crate::{config::Config, model::Transcript};
use anyhow::{Result, anyhow};
use aspasia::{AssSubtitle, PlainSubtitle};

pub fn write_ass(t: &Transcript, cfg: &Config) -> String {
    let ass_cfg = &cfg.formats.ass;
    let mut out = String::new();

    out.push_str("[Script Info]\n");
    out.push_str("ScriptType: v4.00+\n");
    out.push_str(&format!("PlayResX: {}\n", ass_cfg.play_res_x));
    out.push_str(&format!("PlayResY: {}\n\n", ass_cfg.play_res_y));

    out.push_str("[V4+ Styles]\n");
    out.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    out.push_str(&format_style(ass_cfg));
    out.push_str("\n[Events]\n");
    out.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    for cue in &t.cues {
        let start = format_ass_timestamp(cue.start_ms + cfg.policy.timestamp_offset_ms);
        let end = format_ass_timestamp(cue.end_ms + cfg.policy.timestamp_offset_ms);
        let text = escape_ass_text(&cue_text_for_export(cue.text.as_str(), cfg));

        out.push_str(&format!(
            "Dialogue: {layer},{start},{end},{style},,{margin_l},{margin_r},{margin_v},,{text}\n",
            layer = ass_cfg.event_layer,
            start = start,
            end = end,
            style = ass_cfg.style_name,
            margin_l = pad_margin(ass_cfg.margin_l),
            margin_r = pad_margin(ass_cfg.margin_r),
            margin_v = pad_margin(ass_cfg.margin_v),
            text = text
        ));
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
        if ch.is_whitespace() {
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

fn format_style(cfg: &crate::config::AssCfg) -> String {
    format!(
        "Style: {name},{font},{size:.1},{primary},{secondary},{outline},{back},{bold},{italic},{underline},{strike},{scale_x},{scale_y},{spacing},{angle},{border_style},{outline_width},{shadow},{alignment},{margin_l},{margin_r},{margin_v},{encoding}\n",
        name = cfg.style_name,
        font = cfg.font_name,
        size = cfg.font_size,
        primary = cfg.primary_color,
        secondary = cfg.secondary_color,
        outline = cfg.outline_color,
        back = cfg.back_color,
        bold = if cfg.bold { -1 } else { 0 },
        italic = if cfg.italic { -1 } else { 0 },
        underline = if cfg.underline { -1 } else { 0 },
        strike = if cfg.strike_out { -1 } else { 0 },
        scale_x = cfg.scale_x,
        scale_y = cfg.scale_y,
        spacing = cfg.spacing,
        angle = cfg.angle,
        border_style = cfg.border_style,
        outline_width = cfg.outline,
        shadow = cfg.shadow,
        alignment = cfg.alignment,
        margin_l = pad_margin(cfg.margin_l),
        margin_r = pad_margin(cfg.margin_r),
        margin_v = pad_margin(cfg.margin_v),
        encoding = cfg.encoding,
    )
}

fn pad_margin(value: i32) -> String {
    let v = value.clamp(0, 9999);
    format!("{:0>4}", v)
}

fn format_ass_timestamp(ms_in: i64) -> String {
    let ms = ms_in.max(0);
    let total_centis = (ms / 10).max(0);
    let centis = total_centis % 100;
    let total_seconds = (total_centis / 100) as i64;
    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;
    format!("{hours}:{minutes:02}:{seconds:02}.{centis:02}")
}

fn escape_ass_text(text: &str) -> String {
    let cleaned = text.replace('\r', "");
    let with_newlines = cleaned.replace('\n', "\\N");
    with_newlines.replace('\\', "\\\\")
}

pub fn parse_ass(input: &str) -> Result<Transcript> {
    if let Ok(ass) = input.parse::<AssSubtitle>() {
        tracing::info!("parsed as ASS via aspasia");
        let plain = PlainSubtitle::from(&ass);
        return Ok(plain_to_transcript(&plain));
    }

    Err(anyhow!("failed to parse as ASS"))
}

fn plain_to_transcript(plain: &PlainSubtitle) -> Transcript {
    use crate::model::Cue;

    let cues: Vec<Cue> = plain
        .events()
        .iter()
        .map(|e| Cue {
            start_ms: moment_to_ms(&e.start),
            end_ms: moment_to_ms(&e.end),
            text: e.text.clone(),
            speaker: None,
        })
        .collect();

    Transcript::new(cues)
}

fn moment_to_ms(m: &aspasia::timing::Moment) -> i64 {
    let h = m.hours();
    let min = m.minutes();
    let s = m.seconds();
    let ms = m.ms();
    (((h * 60 + min) * 60 + s) * 1000 + ms) as i64
}
