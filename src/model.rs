use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub cues: Vec<Cue>,
    #[serde(default)]
    pub meta: Meta,
}

impl Transcript {
    pub fn new(cues: Vec<Cue>) -> Self {
        Self {
            cues,
            meta: Meta::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    pub fn duration_ms(&self) -> i64 {
        self.cues.last().map(|c| c.end_ms).unwrap_or(0).max(0)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Meta {
    pub source: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cue {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
    #[serde(default)]
    pub speaker: Option<String>,
}

impl Cue {
    pub fn duration_ms(&self) -> i64 {
        (self.end_ms - self.start_ms).max(0)
    }
}
