//! Subresource Integrity (SRI) hash types
//!
//! Pure compute over byte slices — no IO.

use data_encoding::BASE64;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// SRI integrity hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sri {
    algo: &'static str,
    b64: String,
}

impl Sri {
    pub fn sha256(data: &[u8]) -> Self {
        let h = Sha256::digest(data);
        Self {
            algo: "sha256",
            b64: BASE64.encode(&h),
        }
    }

    pub fn sha384(data: &[u8]) -> Self {
        let h = Sha384::digest(data);
        Self {
            algo: "sha384",
            b64: BASE64.encode(&h),
        }
    }

    pub fn sha512(data: &[u8]) -> Self {
        let h = Sha512::digest(data);
        Self {
            algo: "sha512",
            b64: BASE64.encode(&h),
        }
    }

    pub fn value(&self) -> String {
        format!("{}-{}", self.algo, self.b64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_known_vector() {
        let sri = Sri::sha256(b"hello world");
        assert_eq!(
            sri.value(),
            "sha256-uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek="
        );
    }

    #[test]
    fn sha384_produces_prefix() {
        let sri = Sri::sha384(b"hello world");
        assert!(sri.value().starts_with("sha384-"));
    }

    #[test]
    fn sha512_produces_prefix() {
        let sri = Sri::sha512(b"hello world");
        assert!(sri.value().starts_with("sha512-"));
    }
}
