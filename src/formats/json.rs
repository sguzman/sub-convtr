use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::{Cue, Transcript};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedJson {
    pub schema: String,
    pub version: u32,
    pub cues: Vec<JsonCue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonCue {
    pub start: Value,
    pub end: Value,
    pub text: String,
    #[serde(default)]
    pub speaker: Option<String>,
}

pub fn write_json(t: &Transcript, cfg_time_units: &str, wrapped: bool) -> Result<String> {
    if wrapped {
        let w = WrappedJson {
            schema: "subxform.transcript".to_string(),
            version: 1,
            cues: t
                .cues
                .iter()
                .map(|c| JsonCue {
                    start: encode_time(c.start_ms, cfg_time_units),
                    end: encode_time(c.end_ms, cfg_time_units),
                    text: c.text.clone(),
                    speaker: c.speaker.clone(),
                })
                .collect(),
        };
        Ok(serde_json::to_string_pretty(&w)?)
    } else {
        let cues: Vec<JsonCue> = t
            .cues
            .iter()
            .map(|c| JsonCue {
                start: encode_time(c.start_ms, cfg_time_units),
                end: encode_time(c.end_ms, cfg_time_units),
                text: c.text.clone(),
                speaker: c.speaker.clone(),
            })
            .collect();
        Ok(serde_json::to_string_pretty(&cues)?)
    }
}

fn encode_time(ms: i64, units: &str) -> Value {
    match units {
        "ms" => Value::from(ms),
        _ => Value::from((ms as f64) / 1000.0),
    }
}

pub fn parse_json(input: &str) -> Result<Transcript> {
    let v: Value = serde_json::from_str(input)?;

    if let Some(cues) = v.get("cues") {
        return parse_json_cues_array(cues);
    }

    if let Some(segs) = v.get("segments") {
        return parse_json_segments_array(segs);
    }

    if v.is_array() {
        return parse_json_cues_array(&v);
    }

    Err(anyhow!("unrecognized JSON transcript shape"))
}

fn parse_json_cues_array(v: &Value) -> Result<Transcript> {
    let arr = v
        .as_array()
        .ok_or_else(|| anyhow!("cues must be an array"))?;
    let mut cues: Vec<Cue> = Vec::with_capacity(arr.len());

    for item in arr {
        let obj = item
            .as_object()
            .ok_or_else(|| anyhow!("cue must be an object"))?;

        let start_ms =
            decode_time_to_ms(obj.get("start").ok_or_else(|| anyhow!("missing start"))?)?;
        let end_ms = decode_time_to_ms(obj.get("end").ok_or_else(|| anyhow!("missing end"))?)?;
        let text = obj
            .get("text")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("missing text"))?
            .to_string();

        let speaker = obj
            .get("speaker")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        cues.push(Cue {
            start_ms,
            end_ms,
            text,
            speaker,
        });
    }

    Ok(Transcript::new(cues))
}

fn parse_json_segments_array(v: &Value) -> Result<Transcript> {
    let arr = v
        .as_array()
        .ok_or_else(|| anyhow!("segments must be an array"))?;
    let mut cues: Vec<Cue> = Vec::with_capacity(arr.len());

    for item in arr {
        let obj = item
            .as_object()
            .ok_or_else(|| anyhow!("segment must be an object"))?;
        let start_ms =
            decode_time_to_ms(obj.get("start").ok_or_else(|| anyhow!("missing start"))?)?;
        let end_ms = decode_time_to_ms(obj.get("end").ok_or_else(|| anyhow!("missing end"))?)?;
        let text = obj
            .get("text")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();

        cues.push(Cue {
            start_ms,
            end_ms,
            text,
            speaker: None,
        });
    }

    Ok(Transcript::new(cues))
}

fn decode_time_to_ms(v: &Value) -> Result<i64> {
    match v {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i)
            } else if let Some(f) = n.as_f64() {
                Ok((f * 1000.0).round() as i64)
            } else {
                Err(anyhow!("bad numeric time"))
            }
        }
        Value::String(s) => {
            if let Ok(i) = s.trim().parse::<i64>() {
                return Ok(i);
            }
            if let Ok(f) = s.trim().parse::<f64>() {
                return Ok((f * 1000.0).round() as i64);
            }
            crate::formats::time::parse_time_to_ms(s)
        }
        _ => Err(anyhow!("unsupported time type")),
    }
}
