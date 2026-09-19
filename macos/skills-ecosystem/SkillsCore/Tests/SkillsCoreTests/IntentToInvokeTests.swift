import Testing
import Foundation
@testable import SkillsCore

/// T-11 (RFC-001 §3.4): Unit tests for IntentToInvoke predicate.
@Suite struct IntentToInvokeTests {

    private func makeSkill(name: String) -> Skill {
        let root = TestFixtures.createTempDirectory()
        let skillDir = TestFixtures.createSkillDirectory(
            in: root, name: name,
            skillMdContent: "---\ndescription: \"Test skill \(name)\"\n---"
        )
        return Skill(url: skillDir)
    }

    // MARK: - Rule 1: non-implicit skill without explicit mention → false

    @Test func testRule1_nonImplicit_noExplicitMention_returnsFalse() {
        let skill = makeSkill(name: "oscal-skill")
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "help me with compliance documents",
            score: 0.80, implicitInvocation: false, triggers: ["OSCAL"]
        )
        #expect(result == false)
    }

    // MARK: - Rule 2: explicit mention → true regardless of score

    @Test func testRule2_explicitMention_belowScoreThreshold_returnsTrue() {
        let skill = makeSkill(name: "oscal-skill")
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "use oscal-skill to validate this",
            score: 0.30, implicitInvocation: false, triggers: []
        )
        #expect(result == true)
    }

    @Test func testRule2_triggerKeywordExplicitMention_returnsTrue() {
        let skill = makeSkill(name: "oscal-skill")
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "I need to work with OSCAL profiles",
            score: 0.40, implicitInvocation: false, triggers: ["OSCAL"]
        )
        #expect(result == true)
    }

    // MARK: - Rule 3: trigger keyword + score qualifies → true

    @Test func testRule3_triggerMatch_aboveThreshold_returnsTrue() {
        let skill = makeSkill(name: "ckodex-oscal")
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "validate my SSP document",
            score: 0.70, implicitInvocation: true,
            triggers: ["SSP", "POA&M", "FedRAMP"]
        )
        #expect(result == true)
    }

    @Test func testRule3_triggerMatch_belowThreshold_fallsThrough() {
        let skill = makeSkill(name: "ckodex-oscal")
        // score < τ_full_load; no explicit mention; should NOT promote via rule 3
        // but rule 5 (implicit) won't fire either since score < 0.65
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "validate my SSP document",
            score: 0.50, implicitInvocation: false,
            triggers: ["SSP"]
        )
        #expect(result == false)
    }

    // MARK: - Rule 4: subject keyword + elevated threshold → true

    @Test func testRule4_subjectMatch_elevatedThreshold_returnsTrue() {
        let skill = makeSkill(name: "nist-skill")
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "I need help with compliance",
            score: 0.76, implicitInvocation: true,
            triggers: [], subjects: ["compliance", "nist"]
        )
        #expect(result == true)
    }

    @Test func testRule4_subjectMatch_justBelowElevatedThreshold_usesRule5() {
        let skill = makeSkill(name: "nist-skill")
        // score = 0.72 < 0.65+0.10=0.75 → rule 4 fails
        // but rule 5: implicitInvocation=true AND score >= 0.65 → true
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "I need help with compliance",
            score: 0.72, implicitInvocation: true,
            triggers: [], subjects: ["compliance"]
        )
        #expect(result == true)
    }

    // MARK: - Rule 5: implicit + score sufficient → true

    @Test func testRule5_implicit_scoreSufficient_returnsTrue() {
        let skill = makeSkill(name: "generic-skill")
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "do something",
            score: 0.65, implicitInvocation: true,
            triggers: [], subjects: []
        )
        #expect(result == true)
    }

    @Test func testRule5_implicit_scoreBelowThreshold_returnsFalse() {
        let skill = makeSkill(name: "generic-skill")
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "do something",
            score: 0.60, implicitInvocation: true
        )
        #expect(result == false)
    }

    // MARK: - Rule 6: default → false

    @Test func testRule6_default_returnsFalse() {
        let skill = makeSkill(name: "rare-skill")
        let result = IntentToInvoke.evaluate(
            skill: skill, message: "completely unrelated message",
            score: 0.10, implicitInvocation: false,
            triggers: ["rare-domain-term"]
        )
        #expect(result == false)
    }

    // MARK: - Word-boundary matching

    @Test func testKeywordMatch_wordBoundary_doesNotMatchSubstring() {
        // "OSCAL" should not match inside "prOSCALe"
        let result = IntentToInvoke.keywordMatch(terms: ["OSCAL"], message: "prOSCALe the document")
        #expect(result == false)
    }

    @Test func testKeywordMatch_wordBoundary_matchesStandaloneWord() {
        let result = IntentToInvoke.keywordMatch(terms: ["OSCAL"], message: "please validate this OSCAL file")
        #expect(result == true)
    }

    @Test func testKeywordMatch_caseInsensitive() {
        let result = IntentToInvoke.keywordMatch(terms: ["FedRAMP"], message: "fedramp compliance is required")
        #expect(result == true)
    }
}
