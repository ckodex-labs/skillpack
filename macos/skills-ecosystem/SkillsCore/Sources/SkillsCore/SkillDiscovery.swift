import Foundation

public struct SkillDiscovery {
    private let fileManager = FileManager.default
    
    public init() {}
    
    public func discoverSkills(in skillsRoot: URL) throws -> [Skill] {
        let resourceKeys: [URLResourceKey] = [.isDirectoryKey]
        let contents = try fileManager.contentsOfDirectory(at: skillsRoot, includingPropertiesForKeys: resourceKeys, options: [.skipsHiddenFiles])
        
        var skills: [Skill] = []
        for dirURL in contents {
            var isDir: ObjCBool = false
            if fileManager.fileExists(atPath: dirURL.path, isDirectory: &isDir), isDir.boolValue {
                let skillMdURL = dirURL.appendingPathComponent("SKILL.md")
                if fileManager.fileExists(atPath: skillMdURL.path) {
                    skills.append(Skill(url: dirURL))
                }
            }
        }
        return skills.sorted { $0.name < $1.name }
    }
    
    public func scanForPhysicalSkills() throws -> [DiscoveredPhysicalSkill] {
        var results: [DiscoveredPhysicalSkill] = []
        
        for agent in AgentRegistry.shared.agents {
            guard agent.integrationType == .symlink else { continue }
            
            guard let url = agent.resolvedURL else { continue }
            var isDir: ObjCBool = false
            guard fileManager.fileExists(atPath: url.path, isDirectory: &isDir), isDir.boolValue else {
                continue
            }
            
            let contents = try fileManager.contentsOfDirectory(at: url, includingPropertiesForKeys: [.isDirectoryKey], options: [.skipsHiddenFiles])
            for item in contents {
                var isItemDir: ObjCBool = false
                guard fileManager.fileExists(atPath: item.path, isDirectory: &isItemDir), isItemDir.boolValue else {
                    continue
                }
                
                // Read symlink status
                if let attrs = try? fileManager.attributesOfItem(atPath: item.path),
                   attrs[.type] as? FileAttributeType == .typeSymbolicLink {
                    continue
                }
                
                // Calculate size using Skill helper
                let size = Skill.calculateSize(of: item)
                
                results.append(DiscoveredPhysicalSkill(
                    agentName: agent.name,
                    skillName: item.lastPathComponent,
                    path: item.path,
                    sizeBytes: size
                ))
            }
        }
        
        return results.sorted { 
            if $0.agentName == $1.agentName {
                return $0.skillName < $1.skillName
            }
            return $0.agentName < $1.agentName
        }
    }
}
