//! SVG Grade Badge Reporter
//!
//! Presentation space only: reads the domain-computed grade and score from an
//! [`Assessment`] and emits a shields.io-style flat badge as SVG markup. No
//! grading or scoring logic lives here.
//!
//! Determinism: output bytes depend only on assessment content (no timestamps,
//! no RNG), so committed badges diff cleanly in CI.

use skillpack_domain::SkillIdentity;
use skillpack_domain::{Assessment, Grade, ReportError, ReportGenerator};

/// Re-usable dispatch table: grade variant -> badge fill color.
fn grade_color(grade: &Grade) -> &'static str {
    match grade {
        Grade::SPlus | Grade::S => "#4c1",
        Grade::A => "#97ca00",
        Grade::B => "#dfb317",
        Grade::C => "#fe7d37",
        Grade::D => "#e05d44",
        Grade::F => "#cb2431",
    }
}

/// XML-escape the five markup-significant characters.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Left cell label: `skillpack: <name>`, truncated so the plate stays sane.
fn label_for(identity: &SkillIdentity) -> String {
    let raw = format!("skillpack: {}", identity.name);
    if raw.chars().count() > 24 {
        format!("{}…", raw.chars().take(23).collect::<String>())
    } else {
        raw
    }
}

/// Approximate rendered width of Verdana 11px text used by shields-style
/// badges: ~7px per glyph plus 10px cell padding per side.
fn cell_width(text: &str) -> f64 {
    let glyphs = text.chars().count() as f64;
    // ~7px/glyph at 11px Verdana + 20px padding, plus one extra glyph-width
    // per ~8 characters so longer labels keep real headroom.
    glyphs * 7.0 + 20.0 + (glyphs / 8.0).floor() * 7.0
}

/// One colored cell of the badge with centered white text.
fn cell(x: f64, width: f64, fill: &str, text: &str) -> String {
    let cx = x + width / 2.0;
    let text = esc(text);
    format!(
        r##"<rect x="{x}" width="{width}" height="20" fill="{fill}"/><text x="{cx}" y="14" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="11" fill="#fff" fill-opacity=".95">{text}</text>"##
    )
}

pub struct BadgeReporter;

impl ReportGenerator for BadgeReporter {
    fn generate(&self, assessment: &Assessment) -> Result<String, ReportError> {
        let grade = assessment.grade();
        // Domain provides the grade; presentation provides display text.
        // cell() escapes; do not pre-escape or name fragments get doubled.
        let grade_text = grade.as_str().to_owned();
        let score_text = format!("{:.1}/150", assessment.total_score().value());
        let label = label_for(&assessment.skill);
        let fill = grade_color(&grade);

        let label_w = cell_width(&label);
        let grade_w = cell_width(&grade_text);
        let score_w = cell_width(&score_text);
        let total_w = label_w + grade_w + score_w;

        let svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="20" role="img" aria-label="SkillPack grade: {grade_text}"><g shape-rendering="crispEdges">{}{}{}</g>{}</svg>"##,
            cell(0.0, label_w, "#555", &label),
            cell(label_w, grade_w, fill, &grade_text),
            cell(label_w + grade_w, score_w, fill, &score_text),
            // Verify-suffix omitted: the score cell already carries the number.
            ""
        );

        Ok(svg)
    }

    fn format(&self) -> &'static str {
        "badge"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skillpack_domain::SkillIdentity;

    fn sample(name: &str) -> Assessment {
        Assessment::new(SkillIdentity::new(name, "0.1.0", "/tmp/skill"))
    }

    #[test]
    fn badge_is_valid_xml_and_carries_grade() {
        let svg = BadgeReporter.generate(&sample("demo")).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.trim_end().ends_with("</svg>"));
        assert!(svg.contains("skillpack: demo"));
        assert!(svg.contains("/150"));
    }

    #[test]
    fn badge_escapes_markup_in_skill_names_once() {
        let svg = BadgeReporter
            .generate(&sample("<script>alert(1)</script>"))
            .unwrap();
        assert!(!svg.contains("<script>"), "raw markup must not survive");
        assert!(svg.contains("&lt;script&gt;"));
        assert!(!svg.contains("&amp;lt;"), "double-escape regression");
    }

    #[test]
    fn badge_is_deterministic() {
        let a = BadgeReporter.generate(&sample("demo")).unwrap();
        let b = BadgeReporter.generate(&sample("demo")).unwrap();
        assert_eq!(a, b, "identical assessments must produce identical bytes");
    }

    #[test]
    fn grade_color_covers_every_variant() {
        for grade in [
            Grade::SPlus,
            Grade::S,
            Grade::A,
            Grade::B,
            Grade::C,
            Grade::D,
            Grade::F,
        ] {
            assert!(grade_color(&grade).starts_with('#'));
        }
    }
}
