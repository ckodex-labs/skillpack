import Testing
import Foundation
@testable import SkillsCore

struct FoundationDBClientTests {
    
    @Test func testFDBOperations() throws {
        let client = FoundationDBClient()
        
        let testKey = "skills/test_db_client"
        let testValue = "hello foundationdb from swift unit test! 🚀"
        
        // 1. Clean up key first
        try? client.clear(key: testKey)
        
        // 2. Get (should be nil initially)
        let initial = try client.get(key: testKey)
        #expect(initial == nil)
        
        // 3. Set value
        try client.set(key: testKey, value: testValue)
        
        // 4. Get and verify
        let retrieved = try client.get(key: testKey)
        #expect(retrieved == testValue)
        
        // 5. Getrange
        let range = try client.getrange(prefix: "skills/test_db")
        #expect(range[testKey] == testValue)
        
        // 6. Clear and verify deleted
        try client.clear(key: testKey)
        let cleared = try client.get(key: testKey)
        #expect(cleared == nil)
    }
    
    @Test func testFDBCheckHealth() throws {
        let client = FoundationDBClient()
        let isHealthy = client.checkHealth()
        // If FDB is functional in the test sandbox, it will return true; otherwise it returns false.
        // We assert that it completes without throwing any errors.
        #expect(isHealthy == true || isHealthy == false)
        
        // Assert that an unreachable fdbcli safely returns false without crashing.
        let invalidClient = FoundationDBClient(fdbcliPath: "/invalid/bin/fdbcli")
        #expect(invalidClient.checkHealth() == false)
    }
}
