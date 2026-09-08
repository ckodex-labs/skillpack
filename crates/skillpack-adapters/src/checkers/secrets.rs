//! Secret Scanner
//!
//! Shared regex + entropy-based secret detection helper.

use regex::Regex;
use std::sync::OnceLock;

pub struct SecretScanner;

#[derive(Debug, PartialEq)]
pub struct SecretHit {
    pub kind: &'static str,
    pub line: usize,
    pub matched: String,
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn scan(&self, content: &str) -> Vec<SecretHit> {
        static AWS: OnceLock<Regex> = OnceLock::new();
        static GH: OnceLock<Regex> = OnceLock::new();
        let aws = AWS.get_or_init(|| {
            Regex::new(r"AKIA[0-9A-Z]{16}").expect("hardcoded AWS key regex is valid")
        });
        let gh = GH.get_or_init(|| {
            Regex::new(r"gh[pousr]_[A-Za-z0-9]{36}").expect("hardcoded GitHub token regex is valid")
        });

        let mut hits = Vec::new();
        for (i, line) in content.lines().enumerate() {
            for m in aws.find_iter(line) {
                hits.push(SecretHit {
                    kind: "aws-access-key",
                    line: i + 1,
                    matched: m.as_str().into(),
                });
            }
            for m in gh.find_iter(line) {
                hits.push(SecretHit {
                    kind: "github-token",
                    line: i + 1,
                    matched: m.as_str().into(),
                });
            }
            for token in line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            {
                if token.len() >= 32 && shannon_entropy(token) > 4.0 {
                    hits.push(SecretHit {
                        kind: "high-entropy",
                        line: i + 1,
                        matched: token.into(),
                    });
                }
            }
        }
        hits
    }
}

fn shannon_entropy(s: &str) -> f64 {
    let len = s.len() as f64;
    if len == 0.0 {
        return 0.0;
    }
    let mut counts = std::collections::HashMap::new();
    for c in s.chars() {
        *counts.entry(c).or_insert(0u32) += 1;
    }
    counts
        .values()
        .map(|&n| {
            let p = n as f64 / len;
            -p * p.log2()
        })
        .sum()
}
