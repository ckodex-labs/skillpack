//! ASC Pattern Validator
//!
//! Validates CNSB ASC string entries against the OIS-ASC pattern.

use regex::Regex;
use std::sync::OnceLock;

pub struct AscPattern;

impl AscPattern {
    pub fn is_valid(s: &str) -> bool {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| {
            Regex::new(r"^OIS-ASC-(0[0-9]{2}|0[1-7][0-9]|08[0-5])$")
                .expect("hardcoded ASC regex is valid")
        });
        re.is_match(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ois_asc_pattern_validates_examples() {
        assert!(AscPattern::is_valid("OIS-ASC-001"));
        assert!(AscPattern::is_valid("OIS-ASC-085"));
    }

    #[test]
    fn ois_asc_rejects_malformed() {
        assert!(!AscPattern::is_valid("ASC-001"));
        assert!(!AscPattern::is_valid("OIS-ASC-"));
        assert!(!AscPattern::is_valid(""));
        assert!(!AscPattern::is_valid("OIS-ASC-9999"));
    }
}
