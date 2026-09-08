//! Signer implementations for EvidenceEnvelope.

use skillpack_domain::envelope::{EvidenceEnvelope, Signature};
use skillpack_domain::ports::{Signer, SignerError};

/// Noop signer for testing — attaches a synthetic signature.
#[derive(Default)]
pub struct NoopSigner;

impl Signer for NoopSigner {
    fn sign(&self, mut envelope: EvidenceEnvelope) -> Result<EvidenceEnvelope, SignerError> {
        envelope.signatures.push(Signature {
            algo: "noop".into(),
            keyid: "noop-test-key".into(),
            sig: "noop-not-cryptographically-valid".into(),
            created_at: None,
        });
        Ok(envelope)
    }
}

/// Cosign-backed signer (keyless or key-file).
pub enum CosignMode {
    Keyless,
    KeyFile(std::path::PathBuf),
}

pub struct CosignSigner {
    mode: CosignMode,
}

impl CosignSigner {
    pub fn new_keyless() -> Self {
        Self {
            mode: CosignMode::Keyless,
        }
    }

    pub fn from_key_file(path: std::path::PathBuf) -> Self {
        Self {
            mode: CosignMode::KeyFile(path),
        }
    }
}

impl Signer for CosignSigner {
    fn sign(&self, mut envelope: EvidenceEnvelope) -> Result<EvidenceEnvelope, SignerError> {
        let payload = serde_json::to_vec(&envelope.statement)
            .map_err(|e| SignerError::SigningFailed(e.to_string()))?;
        let (algo, keyid, sig) = match &self.mode {
            CosignMode::Keyless => sign_keyless(&payload)?,
            CosignMode::KeyFile(p) => sign_with_key(p, &payload)?,
        };
        envelope.signatures.push(Signature {
            algo,
            keyid,
            sig,
            created_at: None,
        });
        Ok(envelope)
    }
}

fn sign_keyless(payload: &[u8]) -> Result<(String, String, String), SignerError> {
    let token_str = std::env::var("SIGSTORE_ID_TOKEN")
        .map_err(|_| SignerError::OidcUnavailable("SIGSTORE_ID_TOKEN not set".into()))?;

    let context = sigstore::bundle::sign::SigningContext::production()
        .map_err(|e| SignerError::SigningFailed(format!("Sigstore context failed: {e}")))?;

    let token = sigstore::oauth::IdentityToken::try_from(token_str.as_str())
        .map_err(|e| SignerError::OidcUnavailable(format!("Invalid identity token: {e}")))?;

    let email = token.unverified_claims().email.clone();

    let signer = context
        .blocking_signer(token)
        .map_err(|e| SignerError::SigningFailed(format!("Signer creation failed: {e}")))?;

    let artifact = signer
        .sign(std::io::Cursor::new(payload))
        .map_err(|e| SignerError::SigningFailed(format!("Signing failed: {e}")))?;

    let bundle = artifact.to_bundle();
    let bundle_json = serde_json::to_string(&bundle)
        .map_err(|e| SignerError::SigningFailed(format!("Bundle serialization failed: {e}")))?;

    Ok(("sigstore".into(), email, bundle_json))
}

fn sign_with_key(
    _path: &std::path::Path,
    _payload: &[u8],
) -> Result<(String, String, String), SignerError> {
    Err(SignerError::KeyAccessDenied(
        "key file signing not yet implemented".into(),
    ))
}
