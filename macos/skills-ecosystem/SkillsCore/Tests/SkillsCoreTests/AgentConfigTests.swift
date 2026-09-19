import Testing
import Foundation
@testable import SkillsCore

struct AgentConfigTests {
    
    @Test func testAgentRegistryCount() {
        // Current registry count is 12 agents (added augment)
        let agents = AgentRegistry.shared.agents
        #expect(agents.count == 12)
    }
    
    @Test func testAgentLookupCaseInsensitive() {
        let registry = AgentRegistry.shared
        
        let claudeLower = registry.agent(named: "claude")
        let claudeUpper = registry.agent(named: "CLAUDE")
        let claudeMixed = registry.agent(named: "cLaUdE")
        
        #expect(claudeLower != nil)
        #expect(claudeLower?.name == "claude")
        #expect(claudeUpper?.name == "claude")
        #expect(claudeMixed?.name == "claude")
    }
    
    @Test func testAgentLookupUnknownReturnsNil() {
        let registry = AgentRegistry.shared
        let unknown = registry.agent(named: "non-existent-agent-name")
        #expect(unknown == nil)
    }
    
    @Test func testCursorAgentIsIndexFile() {
        let registry = AgentRegistry.shared
        let cursor = registry.agent(named: "cursor")
        
        #expect(cursor != nil)
        #expect(cursor?.integrationType == .indexFile)
    }
    
    @Test func testOtherAgentsAreSymlinks() {
        let registry = AgentRegistry.shared
        for agent in registry.agents {
            if agent.name == "cursor" { continue }
            #expect(agent.integrationType == .symlink)
        }
    }
    
    @Test func testAugmentAgentExists() {
        let registry = AgentRegistry.shared
        let augment = registry.agent(named: "augment")
        
        #expect(augment != nil)
        #expect(augment?.name == "augment")
        #expect(augment?.integrationType == .symlink)
        #expect(augment?.envOverrideKey == "AUGMENT_SKILLS_DIR")
    }
}
