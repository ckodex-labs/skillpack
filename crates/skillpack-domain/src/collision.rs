//! Description Collision Detection
//!
//! Skill descriptions are routers: an agent picks which skill to load by
//! matching a task against them. Two skills with near-identical descriptions
//! compete for the same triggers, so one silently shadows the other.
//!
//! This module scores pairwise description similarity with a stemmed TF-IDF
//! cosine — a deterministic, dependency-free approximation of lexical
//! routing. Constants follow the agent-skills catalog validation harness:
//! warn at cosine >= 0.5, error at >= 0.75, name tokens weighted 2x,
//! IDF = ln(1 + n/(1+df)).

use std::collections::{HashMap, HashSet};

/// Similarity above which two descriptions likely compete for triggers.
pub const COLLISION_WARN: f64 = 0.5;
/// Similarity above which routing between the two skills is effectively random.
pub const COLLISION_ERROR: f64 = 0.75;

/// Weight multiplier for tokens drawn from the skill name.
const NAME_TOKEN_WEIGHT: f64 = 2.0;

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "any", "are", "as", "at", "be", "before", "by", "for", "from", "in", "into",
    "is", "it", "its", "my", "need", "needs", "of", "on", "or", "our", "so", "that", "the", "them",
    "this", "to", "use", "want", "we", "when", "with", "you", "your", "help", "me", "i",
];

/// One skill's routing surface: its name plus frontmatter description.
#[derive(Debug, Clone)]
pub struct RoutingSurface {
    pub name: String,
    pub description: String,
}

/// A pair of skills whose descriptions collide.
#[derive(Debug, Clone, PartialEq)]
pub struct Collision {
    pub a: String,
    pub b: String,
    pub similarity: f64,
    pub severity: CollisionSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionSeverity {
    Warn,
    Error,
}

/// Pairwise collision scan over a skill catalog. O(n²) on descriptions —
/// fine for catalogs in the thousands.
pub fn find_collisions(surfaces: &[RoutingSurface]) -> Vec<Collision> {
    let vectors: Vec<HashMap<String, f64>> = {
        let tfs: Vec<HashMap<String, f64>> = surfaces
            .iter()
            .map(|s| term_frequencies(&s.name, &s.description))
            .collect();
        let idf = inverse_document_frequencies(&tfs);
        tfs.iter()
            .map(|tf| {
                tf.iter()
                    .map(|(t, f)| (t.clone(), f * idf.get(t).copied().unwrap_or(0.0)))
                    .collect()
            })
            .collect()
    };

    let mut out = Vec::new();
    for i in 0..surfaces.len() {
        for j in (i + 1)..surfaces.len() {
            let sim = cosine(&vectors[i], &vectors[j]);
            if sim >= COLLISION_WARN {
                out.push(Collision {
                    a: surfaces[i].name.clone(),
                    b: surfaces[j].name.clone(),
                    similarity: sim,
                    severity: if sim >= COLLISION_ERROR {
                        CollisionSeverity::Error
                    } else {
                        CollisionSeverity::Warn
                    },
                });
            }
        }
    }
    out.sort_by(|x, y| y.similarity.total_cmp(&x.similarity));
    out
}

/// One skill's rank against a task prompt.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutingHit {
    pub name: String,
    pub score: f64,
    pub rank: usize,
}

/// Rank the catalog against a task prompt by TF-IDF cosine similarity — a
/// deterministic approximation of how an agent routes a task to a skill via
/// its description. Returns the top `top_k` skills, highest score first.
///
/// This is the Tier-2 routing check: a skill whose triggers are well-written
/// should surface at rank 1 for the tasks it is meant to handle; if it does
/// not, its description under-routes and needs a sharper "Use when …" trigger.
pub fn rank_by_query(surfaces: &[RoutingSurface], query: &str, top_k: usize) -> Vec<RoutingHit> {
    if surfaces.is_empty() {
        return Vec::new();
    }
    let tfs: Vec<HashMap<String, f64>> = surfaces
        .iter()
        .map(|s| term_frequencies(&s.name, &s.description))
        .collect();
    let idf = inverse_document_frequencies(&tfs);
    let weight = |tf: &HashMap<String, f64>| -> HashMap<String, f64> {
        tf.iter()
            .map(|(t, f)| (t.clone(), f * idf.get(t).copied().unwrap_or(0.0)))
            .collect()
    };

    // Query is scored against the corpus IDF; no name, so weight 1 per token.
    let mut query_tf: HashMap<String, f64> = HashMap::new();
    for tok in tokenize(query) {
        *query_tf.entry(tok).or_default() += 1.0;
    }
    let query_vec = weight(&query_tf);

    let mut hits: Vec<(String, f64)> = surfaces
        .iter()
        .zip(tfs.iter())
        .map(|(s, tf)| (s.name.clone(), cosine(&weight(tf), &query_vec)))
        .collect();
    hits.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    hits.into_iter()
        .take(top_k.max(1))
        .enumerate()
        .map(|(i, (name, score))| RoutingHit {
            name,
            score,
            rank: i + 1,
        })
        .collect()
}

fn term_frequencies(name: &str, description: &str) -> HashMap<String, f64> {
    let mut tf: HashMap<String, f64> = HashMap::new();
    for tok in tokenize(name) {
        *tf.entry(tok).or_default() += NAME_TOKEN_WEIGHT;
    }
    for tok in tokenize(description) {
        *tf.entry(tok).or_default() += 1.0;
    }
    tf
}

fn inverse_document_frequencies(tfs: &[HashMap<String, f64>]) -> HashMap<String, f64> {
    let n = tfs.len() as f64;
    let mut df: HashMap<&str, f64> = HashMap::new();
    for tf in tfs {
        let unique: HashSet<&str> = tf.keys().map(String::as_str).collect();
        for t in unique {
            *df.entry(t).or_default() += 1.0;
        }
    }
    df.into_iter()
        .map(|(t, d)| (t.to_string(), (1.0 + n / (1.0 + d)).ln()))
        .collect()
}

fn cosine(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let dot: f64 = a.iter().filter_map(|(t, v)| b.get(t).map(|w| v * w)).sum();
    let na: f64 = a.values().map(|v| v * v).sum::<f64>().sqrt();
    let nb: f64 = b.values().map(|v| v * v).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() > 1 && !STOPWORDS.contains(t))
        .map(stem)
        .filter(|t| !t.is_empty())
        .collect()
}

/// Light suffix stemmer: ally/ing/ed/es/al, then plural -s, trailing -e,
/// doubled-consonant collapse, and trailing y -> i.
fn stem(token: &str) -> String {
    let mut t = token.to_string();
    for suffix in ["ally", "ing", "ed", "es", "al"] {
        if t.len() > suffix.len() + 2 {
            if let Some(stripped) = t.strip_suffix(suffix) {
                t = stripped.to_string();
                break;
            }
        }
    }
    if t.len() > 3 {
        if let Some(stripped) = t.strip_suffix('s') {
            t = stripped.to_string();
        }
    }
    if t.len() > 3 {
        if let Some(stripped) = t.strip_suffix('e') {
            t = stripped.to_string();
        }
    }
    let bytes = t.as_bytes();
    if bytes.len() > 3 && bytes[bytes.len() - 1] == bytes[bytes.len() - 2] {
        t.pop();
    }
    if t.len() > 3 && t.ends_with('y') {
        t.pop();
        t.push('i');
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(name: &str, desc: &str) -> RoutingSurface {
        RoutingSurface {
            name: name.into(),
            description: desc.into(),
        }
    }

    #[test]
    fn identical_descriptions_collide_at_error_level() {
        let hits = find_collisions(&[
            s(
                "format-csv",
                "Convert CSV files into JSON records with schema inference.",
            ),
            s(
                "csv-format",
                "Convert CSV files into JSON records with schema inference.",
            ),
        ]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].severity, CollisionSeverity::Error);
        assert!(hits[0].similarity > 0.9);
    }

    #[test]
    fn unrelated_descriptions_do_not_collide() {
        let hits = find_collisions(&[
            s(
                "format-csv",
                "Convert CSV files into JSON records with schema inference.",
            ),
            s(
                "deploy-k8s",
                "Roll out container workloads to Kubernetes clusters with canary gates.",
            ),
        ]);
        assert!(hits.is_empty());
    }

    #[test]
    fn near_duplicates_warn() {
        let hits = find_collisions(&[
            s(
                "azure-storage",
                "Manage Azure storage accounts, blobs, and access keys for cloud workloads.",
            ),
            s(
                "azure-storage-admin",
                "Manage Azure storage accounts, containers, and access policies for workloads.",
            ),
        ]);
        assert_eq!(hits.len(), 1, "expected one collision, got {:?}", hits);
        assert!(hits[0].similarity >= COLLISION_WARN);
    }

    #[test]
    fn stemmer_normalizes_variants() {
        assert_eq!(stem("formatting"), stem("formats"));
        assert_eq!(stem("deployed"), stem("deploys"));
    }

    #[test]
    fn rank_by_query_surfaces_the_right_skill() {
        let fleet = vec![
            s(
                "csv-to-json",
                "Convert CSV files into JSON records with schema inference. Use when transforming tabular exports.",
            ),
            s(
                "deploy-k8s",
                "Roll out container workloads to Kubernetes clusters with canary gates.",
            ),
            s(
                "pdf-extract",
                "Extract text and tables from PDF documents into structured output.",
            ),
        ];
        let hits = rank_by_query(&fleet, "I need to turn a CSV export into JSON", 3);
        assert_eq!(hits[0].name, "csv-to-json");
        assert_eq!(hits[0].rank, 1);
        assert!(hits[0].score > 0.0);
    }

    #[test]
    fn rank_by_query_respects_top_k_and_empty() {
        let fleet = vec![
            s("a", "Manage cloud storage buckets and access keys."),
            s("b", "Roll out Kubernetes workloads with canary gates."),
        ];
        assert_eq!(rank_by_query(&fleet, "kubernetes rollout", 1).len(), 1);
        assert!(rank_by_query(&[], "anything", 3).is_empty());
    }
}
