//! Markdown Reporter

use skillpack_domain::{Assessment, ReportError, ReportGenerator};

pub struct MarkdownReporter;

impl ReportGenerator for MarkdownReporter {
    fn generate(&self, assessment: &Assessment) -> Result<String, ReportError> {
        let mut md = String::new();

        md.push_str(&format!(
            "# SkillPack Report: {}\n\n",
            assessment.skill.name
        ));
        md.push_str(&format!("**Version:** {}  \n", assessment.skill.version));
        md.push_str(&format!("**Grade:** {}  \n", assessment.grade()));
        md.push_str(&format!(
            "**Score:** {:.0}/150\n\n",
            assessment.total_score().value()
        ));

        md.push_str("## Dimensions\n\n");
        md.push_str("| Dimension | Score |\n");
        md.push_str("|-----------|-------|\n");
        for (id, score) in &assessment.dimension_scores {
            md.push_str(&format!("| {} | {:.0}/100 |\n", id.name(), score.value()));
        }

        if !assessment.issues.is_empty() {
            md.push_str("\n## Issues\n\n");
            for issue in &assessment.issues {
                md.push_str(&format!(
                    "- **{:?}** [{}]: {}\n",
                    issue.severity,
                    issue.dimension.name(),
                    issue.message
                ));
            }
        }

        Ok(md)
    }

    fn format(&self) -> &'static str {
        "markdown"
    }
}
