import Testing
import Foundation
@testable import SkillsCore

/// T-10 (§3.6 failure modes): Documents and verifies graceful fallback when
/// FoundationDB / RustFS is unavailable. All tests use the LOCAL backend so
/// they run without infrastructure; the distributed-backend path is exercised
/// by the FoundationDBClientTests / RustFSClientTests suites against stubs.
@Suite(.serialized) struct StorageBackendFallbackTests {

    // MARK: - LocalSkillStorageProvider fallback behaviour

    @Test func testLocalProvider_getManifest_returnsNilWhenBundleJsonAbsent() throws {
        let root = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(root) }
        _ = TestFixtures.createSkillDirectory(in: root, name: "no-bundle", skillMdContent: "---\ndescription: \"No bundle\"\n---")

        let provider = LocalSkillStorageProvider(skillsRoot: root)
        let manifest = try provider.getManifest(named: "no-bundle")
        #expect(manifest == nil)
    }

    @Test func testLocalProvider_getManifest_returnsContentWhenPresent() throws {
        let root = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(root) }
        let skillDir = TestFixtures.createSkillDirectory(in: root, name: "has-bundle", skillMdContent: "---\ndescription: \"Has bundle\"\n---")
        let sentinel = "{\"manifest_version\":1,\"name\":\"has-bundle\"}"
        try sentinel.write(to: skillDir.appendingPathComponent("bundle.json"), atomically: true, encoding: .utf8)

        let provider = LocalSkillStorageProvider(skillsRoot: root)
        let manifest = try provider.getManifest(named: "has-bundle")
        #expect(manifest == sentinel)
    }

    @Test func testLocalProvider_getEvidence_returnsEmptyWhenDirAbsent() throws {
        let root = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(root) }
        _ = TestFixtures.createSkillDirectory(in: root, name: "no-evidence", skillMdContent: "---\ndescription: \"No evidence\"\n---")

        let provider = LocalSkillStorageProvider(skillsRoot: root)
        let records = try provider.getEvidence(named: "no-evidence", predicateType: nil)
        #expect(records.isEmpty)
    }

    @Test func testLocalProvider_getEvidence_filtersOnPredicateType() throws {
        let root = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(root) }
        let skillDir = TestFixtures.createSkillDirectory(in: root, name: "ev-skill", skillMdContent: "---\ndescription: \"Evidence\"\n---")
        let evidenceDir = skillDir.appendingPathComponent(".evidence")
        try FileManager.default.createDirectory(at: evidenceDir, withIntermediateDirectories: true)

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601

        let r1 = SkillEvidenceRecord(skillName: "ev-skill", predicateType: "urn:skill:static-analysis:v1", payload: "{}", recordedAt: Date())
        let r2 = SkillEvidenceRecord(skillName: "ev-skill", predicateType: "cyclonedx.org/bom", payload: "{}", recordedAt: Date())
        try encoder.encode(r1).write(to: evidenceDir.appendingPathComponent("r1.json"))
        try encoder.encode(r2).write(to: evidenceDir.appendingPathComponent("r2.json"))

        let provider = LocalSkillStorageProvider(skillsRoot: root)
        let filtered = try provider.getEvidence(named: "ev-skill", predicateType: "urn:skill:static-analysis:v1")
        #expect(filtered.count == 1)
        #expect(filtered[0].predicateType == "urn:skill:static-analysis:v1")

        let all = try provider.getEvidence(named: "ev-skill", predicateType: nil)
        #expect(all.count == 2)
    }

    @Test func testLocalProvider_listVersions_returnsEmptyWhenDirAbsent() throws {
        let root = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(root) }
        _ = TestFixtures.createSkillDirectory(in: root, name: "no-versions", skillMdContent: "---\ndescription: \"No versions\"\n---")

        let provider = LocalSkillStorageProvider(skillsRoot: root)
        let versions = try provider.listVersions(named: "no-versions")
        #expect(versions.isEmpty)
    }

    @Test func testLocalProvider_listVersions_returnsSortedDescending() throws {
        let root = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(root) }
        let skillDir = TestFixtures.createSkillDirectory(in: root, name: "versioned", skillMdContent: "---\ndescription: \"V\"\n---")
        let versionsDir = skillDir.appendingPathComponent(".versions")
        try FileManager.default.createDirectory(at: versionsDir, withIntermediateDirectories: true)
        for v in ["1.0.0", "1.1.0", "2.0.0"] {
            try "".write(to: versionsDir.appendingPathComponent("\(v).md"), atomically: true, encoding: .utf8)
        }

        let provider = LocalSkillStorageProvider(skillsRoot: root)
        let versions = try provider.listVersions(named: "versioned")
        #expect(versions == ["2.0.0", "1.1.0", "1.0.0"])
    }

    // MARK: - FoundationDBClient / RustFSClient unreachable path

    @Test func testFDBClient_checkHealth_returnsFalseWhenBinaryMissing() {
        let client = FoundationDBClient(fdbcliPath: "/tmp/nonexistent-fdbcli-\(UUID().uuidString)")
        #expect(client.checkHealth() == false)
    }

    @Test func testRustFSClient_checkHealth_returnsFalseWhenUnreachable() {
        let client = RustFSClient(endpointURL: "http://127.0.0.1:1", session: .shared)
        #expect(client.checkHealth() == false)
    }

    // MARK: - SyncManager falls back to local when STORAGE_BACKEND=local (default)

    @Test func testSyncManager_localBackend_survivesWithoutFDB() throws {
        let root = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(root) }
        _ = TestFixtures.createSkillDirectory(in: root, name: "fallback-skill", skillMdContent: "---\ndescription: \"Fallback\"\n---")

        // Ensure no STORAGE_BACKEND override leaks distributed path
        unsetenv("STORAGE_BACKEND")

        var logs: [String] = []
        let manager = SyncManager(skillsRootPath: root.path) { logs.append($0) }

        #expect(throws: Never.self) {
            try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: nil))
        }

        // bundle.json should exist (T-05)
        let bundleJson = root.appendingPathComponent("bundle.json")
        #expect(FileManager.default.fileExists(atPath: bundleJson.path))

        // manifest.yaml should exist
        let manifest = root.appendingPathComponent("manifest.yaml")
        #expect(FileManager.default.fileExists(atPath: manifest.path))
    }
}
