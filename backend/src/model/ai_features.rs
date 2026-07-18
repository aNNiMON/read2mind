use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiFeature {
    Summary,
    Mindmap,
}

impl fmt::Display for AiFeature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Summary => "summary",
                Self::Mindmap => "mindmap",
            }
        )
    }
}

impl From<&str> for AiFeature {
    fn from(s: &str) -> Self {
        match s {
            "summary" => Self::Summary,
            "mindmap" => Self::Mindmap,
            _ => panic!("unknown AI feature: {s}"),
        }
    }
}
