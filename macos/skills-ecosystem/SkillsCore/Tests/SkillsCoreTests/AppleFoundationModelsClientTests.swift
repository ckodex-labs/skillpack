import Testing
import Foundation
@testable import SkillsCore

struct AppleFoundationModelsClientTests {
    
    @Test func testAvailabilityAndGeneration() throws {
        let client = AppleFoundationModelsClient()
        
        if client.isAvailable {
            // Verify content generation doesn't crash on available systems
            let prompt = "Respond with 'hello' only."
            let response = try client.generateContent(prompt: prompt)
            #expect(!response.isEmpty)
        } else {
            // Verify that calling on unavailable system throws the correct error
            withKnownIssue("Expected behavior on machines where Apple Intelligence is not enabled or supported.") {
                let prompt = "Respond with 'hello' only."
                #expect(throws: Error.self) {
                    try client.generateContent(prompt: prompt)
                }
            }
        }
    }
}
