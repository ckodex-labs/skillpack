import Foundation

/// T-11 (RFC-001 §3.4): `intent_to_invoke(skill, message)` predicate.
///
/// Guards the L2 → L3 tier promotion transition.  Returns `true` when a
/// user message contains direct, specific evidence that the user wants to
/// invoke the given skill in the current turn.
///
/// Rules are evaluated in priority order; first match wins.  This
/// implementation covers the lexical path (rules 1–4 + 5) which is
/// correct and conservative without embedding infrastructure.
public struct IntentToInvoke {

    // MARK: - Configuration (RFC-001 §5)

    /// L2 → L3 promotion threshold (τ_full_load).
    public static let tauFullLoad: Double = 0.65

    // MARK: - Public API

    /// Evaluate `intent_to_invoke(skill, message, score)`.
    ///
    /// - Parameters:
    ///   - skill: The `Skill` being evaluated at L2.
    ///   - message: The current user message string.
    ///   - score: Pre-computed composite score ∈ [0,1] for this skill.
    ///   - implicitInvocation: Whether the skill allows implicit invocation
    ///     (corresponds to `runtime.implicitInvocation` in `skill.json`).
    ///     Defaults to `true` for v1 manifests without an ontology block.
    ///   - triggers: Ontology trigger keywords from `skill.json` ontology block.
    ///   - subjects: Ontology subject keywords from `skill.json` ontology block.
    /// - Returns: `true` if the skill should be promoted to L3.
    public static func evaluate(
        skill: Skill,
        message: String,
        score: Double,
        implicitInvocation: Bool = true,
        triggers: [String] = [],
        subjects: [String] = []
    ) -> Bool {
        let msg = message.lowercased()

        // Rule 1: non-implicit skill requires explicit mention
        if !implicitInvocation && !explicitMention(skill: skill, message: msg, triggers: triggers) {
            return false
        }

        // Rule 2: explicit mention — sufficient regardless of score
        if explicitMention(skill: skill, message: msg, triggers: triggers) {
            return true
        }

        // Rule 3: ontology trigger keyword present AND score qualifies
        if !triggers.isEmpty && keywordMatch(terms: triggers, message: msg) && score >= tauFullLoad {
            return true
        }

        // Rule 4: subject keyword with elevated threshold (+0.10)
        if !subjects.isEmpty && keywordMatch(terms: subjects, message: msg) && score >= tauFullLoad + 0.10 {
            return true
        }

        // Rule 5: implicit skill, score alone sufficient
        if implicitInvocation && score >= tauFullLoad {
            return true
        }

        // Rule 6: default — do not promote
        return false
    }

    // MARK: - Sub-predicates (RFC-001 §3.4)

    /// Returns true if any of skill.name, metadata.name, or any trigger
    /// appears verbatim (case-insensitive) in `message`.
    public static func explicitMention(skill: Skill, message: String, triggers: [String] = []) -> Bool {
        let msg = message.lowercased()
        let names = [skill.name] + triggers
        return names.contains(where: { term in
            !term.isEmpty && msg.contains(term.lowercased())
        })
    }

    /// Returns true if any term in `terms` appears in `message` at a word
    /// boundary (case-insensitive).
    public static func keywordMatch(terms: [String], message: String) -> Bool {
        let msg = message.lowercased()
        for term in terms {
            let t = term.lowercased()
            guard !t.isEmpty else { continue }
            // Word-boundary check: surrounded by non-alphanumeric or at string edges
            if wordBoundaryContains(haystack: msg, needle: t) {
                return true
            }
        }
        return false
    }

    // MARK: - Internal helpers

    private static func wordBoundaryContains(haystack: String, needle: String) -> Bool {
        guard let range = haystack.range(of: needle) else { return false }

        let beforeIdx = range.lowerBound
        let afterIdx = range.upperBound

        let beforeOk = beforeIdx == haystack.startIndex
            || !haystack[haystack.index(before: beforeIdx)].isLetter
        let afterOk = afterIdx == haystack.endIndex
            || !haystack[afterIdx].isLetter

        return beforeOk && afterOk
    }
}
