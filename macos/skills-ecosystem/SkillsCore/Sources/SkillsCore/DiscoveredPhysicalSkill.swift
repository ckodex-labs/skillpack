import Foundation

public struct DiscoveredPhysicalSkill {
    public let agentName: String
    public let skillName: String
    public let path: String
    public let sizeBytes: Int
    
    public init(agentName: String, skillName: String, path: String, sizeBytes: Int) {
        self.agentName = agentName
        self.skillName = skillName
        self.path = path
        self.sizeBytes = sizeBytes
    }
}
