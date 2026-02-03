use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub logging: Logging,
    pub policy: Policy,
    pub formats: Formats,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            logging: Logging::default(),
            policy: Policy::default(),
            formats: Formats::default(),
        }
    }
}

impl Config {
    pub fn load(path_opt: Option<&Path>) -> Result<Self> {
        let default_path = Path::new("config.toml");
        let path = if let Some(p) = path_opt {
            Some(p)
        } else if default_path.exists() {
            Some(default_path)
        } else {
            None
        };

        let mut cfg = Config::default();

        if let Some(path) = path {
            let raw = fs::read_to_string(path)
                .with_context(|| format!("failed reading config file: {}", path.display()))?;
            let parsed: Config = toml::from_str(&raw)
                .with_context(|| format!("failed parsing TOML config: {}", path.display()))?;
            cfg = parsed;
        }

        Ok(cfg)
    }

    pub fn to_toml_pretty(&self) -> Result<String> {
        let s = toml::to_string_pretty(self).context("failed serializing config as TOML")?;
        Ok(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub level: String,
    pub format: String,
    pub debug_cue_samples: usize,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            debug_cue_samples: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub synthesize_timings: bool,
    pub chars_per_second: f64,
    pub min_duration_ms: i64,
    pub max_duration_ms: i64,
    pub gap_ms: i64,
    pub normalize_whitespace: bool,
    pub trim_text: bool,
    pub timestamp_offset_ms: i64,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            synthesize_timings: true,
            chars_per_second: 18.0,
            min_duration_ms: 600,
            max_duration_ms: 8_000,
            gap_ms: 120,
            normalize_whitespace: true,
            trim_text: true,
            timestamp_offset_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Formats {
    pub srt: SrtCfg,
    pub vtt: VttCfg,
    pub txt: TxtCfg,
    pub tsv: TsvCfg,
    pub json: JsonCfg,
}

impl Default for Formats {
    fn default() -> Self {
        Self {
            srt: SrtCfg::default(),
            vtt: VttCfg::default(),
            txt: TxtCfg::default(),
            tsv: TsvCfg::default(),
            json: JsonCfg::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrtCfg {
    pub wrap_width: usize,
    pub max_lines: usize,
}

impl Default for SrtCfg {
    fn default() -> Self {
        Self {
            wrap_width: 42,
            max_lines: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VttCfg {
    pub wrap_width: usize,
}

impl Default for VttCfg {
    fn default() -> Self {
        Self { wrap_width: 60 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxtCfg {
    pub mode: String,
}

impl Default for TxtCfg {
    fn default() -> Self {
        Self {
            mode: "timestamp_range".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsvCfg {
    pub time_units: String,
    pub columns: Vec<String>,
}

impl Default for TsvCfg {
    fn default() -> Self {
        Self {
            time_units: "ms".to_string(),
            columns: vec![
                "start".to_string(),
                "end".to_string(),
                "text".to_string(),
                "speaker".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonCfg {
    pub time_units: String,
    pub wrapped: bool,
}

impl Default for JsonCfg {
    fn default() -> Self {
        Self {
            time_units: "seconds".to_string(),
            wrapped: true,
        }
    }
}

pub fn init_tracing(logging: &Logging, cli_override_level: Option<&str>) -> Result<()> {
    use tracing_subscriber::{EnvFilter, fmt};

    let level = cli_override_level.unwrap_or(logging.level.as_str());
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let is_json = logging.format.to_lowercase() == "json";

    if is_json {
        fmt()
            .with_env_filter(filter)
            .event_format(fmt::format().json())
            .with_target(true)
            .init();
    } else {
        fmt()
            .with_env_filter(filter)
            .with_target(true)
            .pretty()
            .init();
    }

    tracing::info!(
        level = level,
        format = logging.format.as_str(),
        "logging initialized"
    );

    Ok(())
}
