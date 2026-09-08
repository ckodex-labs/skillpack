//! Score Entity
//!
//! Scoring model with base + bonus points.

use serde::{Deserialize, Serialize};

/// Score value (0-100 for dimension, 0-150 for total)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Score {
    value: f64,
    is_stub: bool,
}

impl Score {
    pub const MAX_DIMENSION: f64 = 100.0;
    pub const MAX_BASE: f64 = 100.0;
    pub const MAX_BONUS: f64 = 50.0;
    pub const MAX_TOTAL: f64 = 150.0;

    pub fn new(value: f64) -> Self {
        Self {
            value: value.clamp(0.0, Self::MAX_TOTAL),
            is_stub: false,
        }
    }

    pub fn dimension(value: f64) -> Self {
        Self {
            value: value.clamp(0.0, Self::MAX_DIMENSION),
            is_stub: false,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn as_percentage(&self) -> f64 {
        (self.value / Self::MAX_DIMENSION) * 100.0
    }

    pub fn with_stub_marker(mut self, is_stub: bool) -> Self {
        self.is_stub = is_stub;
        self
    }

    pub fn is_stub(&self) -> bool {
        self.is_stub
    }
}

/// Grade derived from total score
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    F,
    D,
    C,
    B,
    A,
    S,
    SPlus,
}

impl Grade {
    pub fn from_score(score: f64) -> Self {
        match score as u32 {
            120.. => Self::SPlus,
            100..=119 => Self::S,
            90..=99 => Self::A,
            80..=89 => Self::B,
            70..=79 => Self::C,
            60..=69 => Self::D,
            _ => Self::F,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SPlus => "S+",
            Self::S => "S",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        }
    }

    pub fn meets_minimum(&self, minimum: &Grade) -> bool {
        self >= minimum
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Bonus point categories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BonusPoints {
    pub slsa_level_3: bool,        // +15
    pub sigstore_signing: bool,    // +10
    pub dagger_pipeline: bool,     // +10
    pub stride_threat_model: bool, // +10
    pub mcp_server: bool,          // +5
}

impl BonusPoints {
    pub fn total(&self) -> u32 {
        let mut total = 0;
        if self.slsa_level_3 {
            total += 15;
        }
        if self.sigstore_signing {
            total += 10;
        }
        if self.dagger_pipeline {
            total += 10;
        }
        if self.stride_threat_model {
            total += 10;
        }
        if self.mcp_server {
            total += 5;
        }
        total
    }
}
