//! File-based evidence sink.

use skillpack_domain::envelope::EvidenceEnvelope;
use skillpack_domain::ports::EvidenceSink;
use std::path::PathBuf;

pub struct FileEvidenceSink {
    out_dir: PathBuf,
}

impl FileEvidenceSink {
    pub fn new(out_dir: impl Into<PathBuf>) -> Self {
        Self {
            out_dir: out_dir.into(),
        }
    }
}

impl EvidenceSink for FileEvidenceSink {
    fn write(
        &self,
        envelope: &EvidenceEnvelope,
    ) -> Result<PathBuf, skillpack_domain::ports::SinkError> {
        std::fs::create_dir_all(&self.out_dir)
            .map_err(|e| skillpack_domain::ports::SinkError::WriteError(e.to_string()))?;
        let path = self.out_dir.join("skillpack-assessment.json");
        let json = serde_json::to_string_pretty(envelope)
            .map_err(|e| skillpack_domain::ports::SinkError::WriteError(e.to_string()))?;
        std::fs::write(&path, json)
            .map_err(|e| skillpack_domain::ports::SinkError::WriteError(e.to_string()))?;
        Ok(path)
    }
}
