//! Secret Scanner
//!
//! Shared regex + entropy-based secret detection helper.
//!
//! Regexes are compiled once per scanner instance; construction is fallible
//! so no `expect`/`unwrap` is needed anywhere in production paths.

use regex::Regex;

/// One detected secret-shaped token.
#[derive(Debug, PartialEq)]
pub struct SecretHit {
    pub kind: &'static str,
    pub line: usize,
    pub matched: String,
}

/// Regex + entropy secret scanner.
///
/// Cheap to build (two small regexes); build one per checker run.
pub struct SecretScanner {
    aws: Regex,
    github: Regex,
}

impl SecretScanner {
    /// Build a scanner, compiling the detection regexes.
    ///
    /// # Errors
    ///
    /// Returns an error when a built-in pattern fails to compile, which can
    /// only happen if the hardcoded patterns above are edited incorrectly.
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            aws: Regex::new(r"AKIA[0-9A-Z]{16}")?,
            github: Regex::new(r"gh[pousr]_[A-Za-z0-9]{36}")?,
        })
    }

    /// Scan `content` line-by-line for known secret shapes and
    /// high-entropy tokens.
    pub fn scan(&self, content: &str) -> Vec<SecretHit> {
        let mut hits = Vec::new();
        for (i, line) in content.lines().enumerate() {
            for m in self.aws.find_iter(line) {
                hits.push(SecretHit {
                    kind: "aws-access-key",
                    line: i + 1,
                    matched: m.as_str().into(),
                });
            }
            for m in self.github.find_iter(line) {
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
