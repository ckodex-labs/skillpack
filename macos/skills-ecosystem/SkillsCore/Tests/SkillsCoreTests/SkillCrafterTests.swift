import Testing
import Foundation
@testable import SkillsCore

struct SkillCrafterTests {
    
    // Custom URLProtocol mock for network stubbing
    class MockURLProtocol: URLProtocol {
        static var requestHandler: ((URLRequest) throws -> (HTTPURLResponse, Data))?
        
        override class func canInit(with request: URLRequest) -> Bool {
            return true
        }
        
        override class func canonicalRequest(for request: URLRequest) -> URLRequest {
            return request
        }
        
        override func startLoading() {
            guard let handler = MockURLProtocol.requestHandler else { return }
            do {
                let (response, data) = try handler(request)
                client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
                client?.urlProtocol(self, didLoad: data)
                client?.urlProtocolDidFinishLoading(self)
            } catch {
                client?.urlProtocol(self, didFailWithError: error)
            }
        }
        
        override func stopLoading() {}
    }
    
    private func makeMockSession() -> URLSession {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [MockURLProtocol.self]
        return URLSession(configuration: configuration)
    }
    
    @Test func testCraftSkillSuccess() throws {
        setenv("DISABLE_APPLE_INTELLIGENCE", "1", 1)
        defer {
            unsetenv("DISABLE_APPLE_INTELLIGENCE")
        }
        
        let tempDir = TestFixtures.createTempDirectory()
        let targetDir = TestFixtures.createTempDirectory()
        defer {
            TestFixtures.removeTempDirectory(tempDir)
            TestFixtures.removeTempDirectory(targetDir)
        }
        
        // 1. Create a dummy file in targetDir so the enumerator finds it
        let dummyFileURL = targetDir.appendingPathComponent("App.swift")
        try! "import SwiftUI".write(to: dummyFileURL, atomically: true, encoding: .utf8)
        
        // 2. Set up LLM mock response
        let mockResponseJSON = """
        {
          "candidates": [
            {
              "content": {
                "parts": [
                  {
                    "text": "{\\"skillName\\": \\"swift-test-skill\\", \\"description\\": \\"A mocked swift test skill\\", \\"skillMarkdown\\": \\"# Mock Swift Skill\\\\n\\\\nContent goes here.\\"}"
                  }
                ]
              }
            }
          ]
        }
        """
        
        MockURLProtocol.requestHandler = { request in
            let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
            let data = mockResponseJSON.data(using: .utf8)!
            return (response, data)
        }
        
        // 3. Initialize SkillCrafter with mock session
        let crafter = SkillCrafter(
            skillsRoot: tempDir,
            apiKey: "mock-key",
            session: makeMockSession(),
            logger: { _ in }
        )
        
        let skillName = try crafter.craftSkill(from: targetDir.path)
        
        #expect(skillName == "swift-test-skill")
        
        // 4. Verify candidate files are generated
        let candidateSkillMdURL = tempDir.appendingPathComponent("candidates/swift-test-skill/SKILL.md")
        #expect(FileManager.default.fileExists(atPath: candidateSkillMdURL.path))
        
        let content = try String(contentsOf: candidateSkillMdURL, encoding: .utf8)
        #expect(content.contains("description: \"A mocked swift test skill\""))
        #expect(content.contains("# Mock Swift Skill"))
    }
    
    @Test func testCraftSkillIPGuardViolation() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        // Target dir matches Thales
        let targetDir = tempDir.appendingPathComponent("thales-project")
        try! FileManager.default.createDirectory(at: targetDir, withIntermediateDirectories: true, attributes: nil)
        
        let crafter = SkillCrafter(
            skillsRoot: tempDir,
            apiKey: "mock-key",
            logger: { _ in }
        )
        
        #expect(throws: IPGuardError.self) {
            try crafter.craftSkill(from: targetDir.path)
        }
    }
}
