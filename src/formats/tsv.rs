use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};

use crate::{
    config::Config,
    formats::time::{format_vtt_timestamp, parse_time_to_ms},
    model::{Cue, Transcript},
};

pub fn write_tsv(t: &Transcript, cfg: &Config) -> Result<String> {
    let mut wtr = WriterBuilder::new().delimiter(b'\t').from_writer(vec![]);

    let cols = cfg.formats.tsv.columns.clone();
    wtr.write_record(&cols)?;

    for cue in &t.cues {
        let mut row: Vec<String> = Vec::with_capacity(cols.len());
        for c in &cols {
            row.push(value_for_column(c, cue, cfg));
        }
        wtr.write_record(&row)?;
    }

    let data = wtr.into_inner().map_err(|e| anyhow!(e.to_string()))?;
    Ok(String::from_utf8(data)?)
}

fn value_for_column(col: &str, cue: &Cue, cfg: &Config) -> String {
    match col {
        "start" => fmt_time(cue.start_ms, cfg),
        "end" => fmt_time(cue.end_ms, cfg),
        "text" => cue.text.clone(),
        "speaker" => cue.speaker.clone().unwrap_or_default(),
        _ => "".to_string(),
    }
}

fn fmt_time(ms: i64, cfg: &Config) -> String {
    match cfg.formats.tsv.time_units.as_str() {
        "seconds" => format!("{:.3}", (ms as f64) / 1000.0),
        "timestamp" => format_vtt_timestamp(ms),
        _ => ms.to_string(),
    }
}

pub fn parse_tsv(input: &str, cfg: &Config) -> Result<Transcript> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(input.as_bytes());

    let headers = rdr.headers()?.clone();
    let start_ix = headers
        .iter()
        .position(|h| h == "start")
        .ok_or_else(|| anyhow!("missing 'start' column"))?;
    let end_ix = headers
        .iter()
        .position(|h| h == "end")
        .ok_or_else(|| anyhow!("missing 'end' column"))?;
    let text_ix = headers
        .iter()
        .position(|h| h == "text")
        .ok_or_else(|| anyhow!("missing 'text' column"))?;
    let speaker_ix = headers.iter().position(|h| h == "speaker");

    let mut cues = Vec::new();

    for rec in rdr.records() {
        let rec = rec?;
        let start_s = rec.get(start_ix).unwrap_or("").trim();
        let end_s = rec.get(end_ix).unwrap_or("").trim();
        let text = rec.get(text_ix).unwrap_or("").to_string();

        let start_ms = parse_tsv_time(start_s, cfg)?;
        let end_ms = parse_tsv_time(end_s, cfg)?;

        let speaker = speaker_ix
            .and_then(|ix| rec.get(ix))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        cues.push(Cue {
            start_ms,
            end_ms,
            text,
            speaker,
        });
    }

    Ok(Transcript::new(cues))
}

fn parse_tsv_time(s: &str, cfg: &Config) -> Result<i64> {
    match cfg.formats.tsv.time_units.as_str() {
        "seconds" => {
            let v: f64 = s.parse()?;
            Ok((v * 1000.0).round() as i64)
        }
        "timestamp" => parse_time_to_ms(s),
        _ => parse_time_to_ms(s),
    }
}
