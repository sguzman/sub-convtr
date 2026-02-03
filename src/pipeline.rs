use anyhow::{Context, Result, anyhow};
use std::{fs, path::Path};

use crate::{
    cli::{ConvertCmd, Format},
    config::Config,
    formats,
    model::{Cue, Transcript},
};

pub fn run_convert(cmd: ConvertCmd, cfg: &Config) -> Result<()> {
    let span = tracing::info_span!("convert", input = cmd.input.as_str(), to = ?cmd.to);
    let _g = span.enter();

    let input_format = cmd
        .from
        .unwrap_or_else(|| infer_format_from_path_or_dash(&cmd.input));
    tracing::info!(?input_format, "input format selected");

    let raw = read_input_to_string(&cmd.input)?;
    tracing::info!(bytes = raw.len(), "read input");

    let mut transcript = parse_any(&raw, input_format, cfg)
        .with_context(|| format!("failed parsing input as {:?}", input_format))?;

    apply_policies(&mut transcript, cfg);

    log_transcript_summary(&transcript, cfg);

    let rendered = render_any(&transcript, cmd.to, cfg)?;

    if cmd.stdout {
        print!("{rendered}");
        tracing::info!(mode = "stdout", "wrote output");
        return Ok(());
    }

    let out_path = derive_output_path(&cmd)?;
    write_output(&out_path, &rendered, cmd.overwrite)?;
    tracing::info!(path = out_path.as_str(), "wrote output file");

    Ok(())
}

fn infer_format_from_path_or_dash(input: &str) -> Format {
    if input == "-" {
        return Format::Txt;
    }
    let p = Path::new(input);
    match p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "srt" => Format::Srt,
        "vtt" => Format::Vtt,
        "txt" => Format::Txt,
        "tsv" => Format::Tsv,
        "json" => Format::Json,
        _ => Format::Txt,
    }
}

fn read_input_to_string(input: &str) -> Result<String> {
    if input == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        Ok(fs::read_to_string(input)?)
    }
}

fn parse_any(raw: &str, fmt: Format, cfg: &Config) -> Result<Transcript> {
    let trimmed = raw.trim_start();
    if fmt == Format::Txt && (trimmed.starts_with('{') || trimmed.starts_with('[')) {
        tracing::info!("stdin looks like JSON; attempting JSON parse");
        if let Ok(t) = formats::json::parse_json(raw) {
            return Ok(t);
        }
    }

    match fmt {
        Format::Srt | Format::Vtt => parse_srt_or_vtt_via_aspasia(raw, fmt),
        Format::Txt => formats::txt::parse_txt(raw, cfg),
        Format::Tsv => formats::tsv::parse_tsv(raw, cfg),
        Format::Json => formats::json::parse_json(raw),
    }
}

fn parse_srt_or_vtt_via_aspasia(raw: &str, fmt: Format) -> Result<Transcript> {
    if fmt == Format::Vtt {
        if let Ok(vtt) = raw.parse::<aspasia::WebVttSubtitle>() {
            tracing::info!("parsed as VTT via aspasia");
            let plain = aspasia::PlainSubtitle::from(&vtt);
            return Ok(plain_to_transcript(&plain));
        }

        if let Ok(srt) = raw.parse::<aspasia::SubRipSubtitle>() {
            tracing::info!("parsed as SRT via aspasia (fallback)");
            let plain = aspasia::PlainSubtitle::from(&srt);
            return Ok(plain_to_transcript(&plain));
        }
    } else {
        if let Ok(srt) = raw.parse::<aspasia::SubRipSubtitle>() {
            tracing::info!("parsed as SRT via aspasia");
            let plain = aspasia::PlainSubtitle::from(&srt);
            return Ok(plain_to_transcript(&plain));
        }

        if let Ok(vtt) = raw.parse::<aspasia::WebVttSubtitle>() {
            tracing::info!("parsed as VTT via aspasia (fallback)");
            let plain = aspasia::PlainSubtitle::from(&vtt);
            return Ok(plain_to_transcript(&plain));
        }
    }

    Err(anyhow!("failed to parse as SRT or VTT"))
}

fn plain_to_transcript(plain: &aspasia::PlainSubtitle) -> Transcript {
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

fn moment_to_ms(m: &aspasia::Moment) -> i64 {
    let h = m.hours();
    let min = m.minutes();
    let s = m.seconds();
    let ms = m.ms();
    (((h * 60 + min) * 60 + s) * 1000 + ms) as i64
}

fn apply_policies(t: &mut Transcript, cfg: &Config) {
    let span = tracing::info_span!("apply_policies");
    let _g = span.enter();

    if cfg.policy.trim_text || cfg.policy.normalize_whitespace {
        for c in &mut t.cues {
            if cfg.policy.trim_text {
                c.text = c.text.trim().to_string();
            }
            if cfg.policy.normalize_whitespace {
                c.text = normalize_ws(&c.text);
            }
        }
    }

    if cfg.policy.synthesize_timings {
        let mut cursor = 0i64;
        for c in &mut t.cues {
            if c.end_ms <= c.start_ms {
                let dur = synth_duration_ms(&c.text, cfg);
                c.start_ms = cursor;
                c.end_ms = cursor + dur;
                cursor = c.end_ms + cfg.policy.gap_ms;
            } else {
                cursor = c.end_ms + cfg.policy.gap_ms;
            }
        }
    }
}

fn synth_duration_ms(text: &str, cfg: &Config) -> i64 {
    let cps = cfg.policy.chars_per_second.max(1.0);
    let raw = (text.chars().count() as f64 / cps * 1000.0).round() as i64;
    raw.clamp(cfg.policy.min_duration_ms, cfg.policy.max_duration_ms)
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

fn log_transcript_summary(t: &Transcript, cfg: &Config) {
    tracing::info!(
        cues = t.cues.len(),
        duration_ms = t.duration_ms(),
        "transcript summary"
    );

    if tracing::enabled!(tracing::Level::DEBUG) {
        let n = cfg.logging.debug_cue_samples.min(t.cues.len());
        for (i, c) in t.cues.iter().take(n).enumerate() {
            tracing::debug!(
                idx = i,
                start_ms = c.start_ms,
                end_ms = c.end_ms,
                chars = c.text.chars().count(),
                "cue sample"
            );
        }
    }
}

fn render_any(t: &Transcript, fmt: Format, cfg: &Config) -> Result<String> {
    match fmt {
        Format::Srt => Ok(formats::srt::write_srt(t, cfg)),
        Format::Vtt => Ok(formats::vtt::write_vtt(t, cfg)),
        Format::Txt => Ok(formats::txt::write_txt(t, cfg)),
        Format::Tsv => formats::tsv::write_tsv(t, cfg),
        Format::Json => formats::json::write_json(
            t,
            cfg.formats.json.time_units.as_str(),
            cfg.formats.json.wrapped,
        ),
    }
}

fn derive_output_path(cmd: &ConvertCmd) -> Result<String> {
    if let Some(o) = &cmd.output {
        return Ok(o.clone());
    }

    if cmd.input == "-" {
        return Err(anyhow!(
            "output path required when input is stdin and --stdout is not set"
        ));
    }

    let p = Path::new(&cmd.input);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("bad input filename"))?;

    let parent = p.parent().unwrap_or_else(|| Path::new("."));
    let out = parent.join(format!("{stem}.{}", cmd.to.extension()));
    Ok(out.to_string_lossy().to_string())
}

fn write_output(path: &str, data: &str, overwrite: bool) -> Result<()> {
    if Path::new(path).exists() && !overwrite {
        return Err(anyhow!(
            "refusing to overwrite existing file (pass --overwrite): {path}"
        ));
    }
    fs::write(path, data)?;
    Ok(())
}
