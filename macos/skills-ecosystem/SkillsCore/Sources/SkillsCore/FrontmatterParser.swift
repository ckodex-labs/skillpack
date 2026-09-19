import Foundation

// MARK: - Sub-types (from skill.v1.1.schema.json)

public struct SkillCapabilityTool: Codable, Equatable {
    public var name: String
    public var scope: String?
    public var risk: String?
    public init(name: String, scope: String? = nil, risk: String? = nil) {
        self.name = name; self.scope = scope; self.risk = risk
    }
}

public struct SkillCapabilities: Codable, Equatable {
    public var requiresNetwork: Bool
    public var requiresFilesystem: Bool
    public var requiresGpu: Bool
    public var os: [String]
    public var arch: [String]
    public var tools: [SkillCapabilityTool]
    public init(requiresNetwork: Bool = false, requiresFilesystem: Bool = true,
                requiresGpu: Bool = false, os: [String] = [], arch: [String] = [],
                tools: [SkillCapabilityTool] = []) {
        self.requiresNetwork = requiresNetwork; self.requiresFilesystem = requiresFilesystem
        self.requiresGpu = requiresGpu; self.os = os; self.arch = arch; self.tools = tools
    }
}

public struct SkillAIPackCapabilities: Codable, Equatable {
    public var read: Bool
    public var write: Bool
    public var network: Bool
    public var filesystem: Bool
    public var execution: Bool
    public init(read: Bool = true, write: Bool = false, network: Bool = false,
                filesystem: Bool = true, execution: Bool = false) {
        self.read = read; self.write = write; self.network = network
        self.filesystem = filesystem; self.execution = execution
    }
}

public struct SkillOntology: Codable, Equatable {
    public var subjects: [String]
    public var produces: [String]
    public var consumes: [String]
    public var related: [String]
    public var exclusiveOf: [String]
    public var triggers: [String]
    public var galMinimum: Int?
    public var proofTypes: [String]
    public init(subjects: [String] = [], produces: [String] = [], consumes: [String] = [],
                related: [String] = [], exclusiveOf: [String] = [], triggers: [String] = [],
                galMinimum: Int? = nil, proofTypes: [String] = []) {
        self.subjects = subjects; self.produces = produces; self.consumes = consumes
        self.related = related; self.exclusiveOf = exclusiveOf; self.triggers = triggers
        self.galMinimum = galMinimum; self.proofTypes = proofTypes
    }
}

public struct SkillRuntime: Codable, Equatable {
    public var implicitInvocation: Bool
    public var minTier: String?
    public var maxTier: String?
    public var l4xAllowed: Bool
    public init(implicitInvocation: Bool = true, minTier: String? = nil,
                maxTier: String? = nil, l4xAllowed: Bool = false) {
        self.implicitInvocation = implicitInvocation; self.minTier = minTier
        self.maxTier = maxTier; self.l4xAllowed = l4xAllowed
    }
}

public struct SkillGovernance: Codable, Equatable {
    public var galMinimum: Int?
    public var proofRequired: Bool?
    public var proofTypes: [String]
    public var enforcementPoints: [String]
    public var emergencyProtocols: [String]
    public init(galMinimum: Int? = nil, proofRequired: Bool? = nil, proofTypes: [String] = [],
                enforcementPoints: [String] = [], emergencyProtocols: [String] = []) {
        self.galMinimum = galMinimum; self.proofRequired = proofRequired
        self.proofTypes = proofTypes; self.enforcementPoints = enforcementPoints
        self.emergencyProtocols = emergencyProtocols
    }
}

public struct SkillPolicy: Codable, Equatable {
    public var sandbox: String
    public var userConfirmation: String
    public var pii: Bool
    public var secrets: Bool
    public var allowedOutputs: [String]
    public init(sandbox: String = "recommended", userConfirmation: String = "dangerous-only",
                pii: Bool = false, secrets: Bool = true, allowedOutputs: [String] = []) {
        self.sandbox = sandbox; self.userConfirmation = userConfirmation
        self.pii = pii; self.secrets = secrets; self.allowedOutputs = allowedOutputs
    }
}

public struct SkillSupplyChain: Codable, Equatable {
    public var repo: String?
    public var path: String?
    public var revision: String?
    public var digest: String?
    public var slsaLevel: Int?
    public init(repo: String? = nil, path: String? = nil, revision: String? = nil,
                digest: String? = nil, slsaLevel: Int? = nil) {
        self.repo = repo; self.path = path; self.revision = revision
        self.digest = digest; self.slsaLevel = slsaLevel
    }
}

// MARK: - ParsedFrontmatter

public struct ParsedFrontmatter {
    // v1 core (skill.v1.schema.json)
    public var name: String
    public var version: String
    public var description: String
    public var license: String
    public var authors: [String]
    public var keywords: [String]
    public var labels: [String: String]
    public var annotations: [String: String]
    public var compatibility: String
    public var allowedTools: [String]
    public var capabilities: SkillCapabilities
    public var aipackCapabilities: SkillAIPackCapabilities
    public var governance: SkillGovernance
    public var policy: SkillPolicy
    public var supplyChain: SkillSupplyChain

    // v1.1 additions (skill.v1.1.schema.json)
    public var synopsis: String
    public var synopsisHash: String
    public var tier: String          // deployment tier: experimental|beta|stable|deprecated|retired
    public var ontology: SkillOntology
    public var runtime: SkillRuntime

    // Legacy / asm-compat fields (kept for SKILL.md frontmatter back-compat)
    public var creator: String       // alias → authors[0]
    public var effort: String
    public var asc: [String]
    public var lifecycle: String     // draft|active|deprecated|retired
    public var ontologyRef: String   // legacy single-ref before v1.1 ontology block

    public static var empty: ParsedFrontmatter {
        ParsedFrontmatter(
            name: "", version: "0.0.0", description: "", license: "",
            authors: [], keywords: [], labels: [:], annotations: [:],
            compatibility: "", allowedTools: [],
            capabilities: SkillCapabilities(),
            aipackCapabilities: SkillAIPackCapabilities(),
            governance: SkillGovernance(),
            policy: SkillPolicy(),
            supplyChain: SkillSupplyChain(),
            synopsis: "", synopsisHash: "", tier: "",
            ontology: SkillOntology(), runtime: SkillRuntime(),
            creator: "", effort: "", asc: [], lifecycle: "unknown", ontologyRef: ""
        )
    }
}

public struct FrontmatterParser {

    public init() {}

    public func parse(url: URL) -> ParsedFrontmatter {
        guard let content = try? String(contentsOf: url, encoding: .utf8) else {
            return .empty
        }
        return parse(content: content)
    }

    public func parse(content: String) -> ParsedFrontmatter {
        var result = ParsedFrontmatter.empty
        let lines = content.components(separatedBy: .newlines)
        var inFrontmatter = false
        var frontmatterClosed = false
        var multilineKey: String? = nil
        var multilineItems: [String] = []

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)

            if trimmed == "---" {
                if !inFrontmatter && !frontmatterClosed {
                    inFrontmatter = true
                    continue
                } else if inFrontmatter {
                    flushMultiline(key: &multilineKey, items: &multilineItems, result: &result)
                    frontmatterClosed = true
                    inFrontmatter = false
                    continue
                }
            }

            guard inFrontmatter else { continue }

            // Multiline array item: "  - value"
            if trimmed.hasPrefix("- "), let mk = multilineKey {
                let val = String(trimmed.dropFirst(2)).trimmingCharacters(in: .whitespaces)
                    .trimmingCharacters(in: CharacterSet(charactersIn: "\"'"))
                multilineItems.append(val)
                _ = mk
                continue
            }

            // If we hit a new key while collecting multiline, flush first
            if multilineKey != nil && trimmed.contains(":") && !trimmed.hasPrefix("-") {
                flushMultiline(key: &multilineKey, items: &multilineItems, result: &result)
            }

            if let (key, val) = splitKeyValue(trimmed) {
                if val.isEmpty {
                    // Start of block array
                    multilineKey = key
                    multilineItems = []
                } else {
                    assign(key: key, value: val, result: &result)
                }
            }
        }

        // Flush any dangling multiline
        flushMultiline(key: &multilineKey, items: &multilineItems, result: &result)

        return result
    }

    // MARK: - Helpers

    private func splitKeyValue(_ line: String) -> (String, String)? {
        guard let colonIdx = line.firstIndex(of: ":") else { return nil }
        let key = String(line[line.startIndex..<colonIdx]).trimmingCharacters(in: .whitespaces)
        let afterColon = String(line[line.index(after: colonIdx)...]).trimmingCharacters(in: .whitespaces)
        let cleaned = afterColon.trimmingCharacters(in: CharacterSet(charactersIn: "\"'"))
        return (key, cleaned)
    }

    private func parseInlineArray(_ raw: String) -> [String] {
        raw.trimmingCharacters(in: CharacterSet(charactersIn: "[]"))
            .components(separatedBy: ",")
            .map { $0.trimmingCharacters(in: .whitespaces).trimmingCharacters(in: CharacterSet(charactersIn: "\"'")) }
            .filter { !$0.isEmpty }
    }

    private func assign(key: String, value: String, result: inout ParsedFrontmatter) {
        switch key.lowercased() {
        // --- v1 core ---
        case "name":
            result.name = value
        case "version":
            result.version = value.isEmpty ? "0.0.0" : value
        case "description":
            result.description = String(value.prefix(1024))
        case "creator", "author":
            result.creator = value
            if result.authors.isEmpty { result.authors = [value] }
        case "license":
            result.license = value
        case "compatibility":
            result.compatibility = value
        case "tools", "allowed_tools", "allowedtools":
            if value.contains("[") {
                result.allowedTools = parseInlineArray(value)
            } else if !value.isEmpty {
                result.allowedTools = [value]
            }
        case "keywords":
            if value.contains("[") { result.keywords = parseInlineArray(value) }
        // --- v1.1 additions ---
        case "synopsis":
            result.synopsis = String(value.prefix(2048))
        case "synopsishash", "synopsis_hash":
            result.synopsisHash = value
        case "tier":
            result.tier = value.lowercased()
        // ontology flat fields (e.g. ontology.subjects: [foo])
        case "ontology.subjects", "subjects":
            if value.contains("[") { result.ontology.subjects = parseInlineArray(value) }
        case "ontology.produces", "produces":
            if value.contains("[") { result.ontology.produces = parseInlineArray(value) }
        case "ontology.consumes", "consumes":
            if value.contains("[") { result.ontology.consumes = parseInlineArray(value) }
        case "ontology.related", "related":
            if value.contains("[") { result.ontology.related = parseInlineArray(value) }
        case "ontology.triggers", "triggers":
            if value.contains("[") { result.ontology.triggers = parseInlineArray(value) }
        case "ontology.exclusiveof", "exclusiveof":
            if value.contains("[") { result.ontology.exclusiveOf = parseInlineArray(value) }
        // runtime
        case "runtime.implicitinvocation", "implicitinvocation":
            result.runtime.implicitInvocation = value.lowercased() != "false"
        case "runtime.l4xallowed", "l4xallowed":
            result.runtime.l4xAllowed = value.lowercased() == "true"
        case "runtime.mintier", "mintier":
            result.runtime.minTier = value.uppercased()
        case "runtime.maxtier", "maxtier":
            result.runtime.maxTier = value.uppercased()
        // governance
        case "governance.galminimum", "galminimum":
            result.governance.galMinimum = Int(value)
        case "governance.proofrequired", "proofrequired":
            result.governance.proofRequired = value.lowercased() == "true"
        // policy
        case "policy.sandbox", "sandbox":
            result.policy.sandbox = value.lowercased()
        case "policy.userconfirmation", "userconfirmation":
            result.policy.userConfirmation = value.lowercased()
        case "policy.pii", "pii":
            result.policy.pii = value.lowercased() == "true"
        case "policy.secrets", "secrets":
            result.policy.secrets = value.lowercased() != "false"
        // supplyChain
        case "supplychain.repo", "repo":
            result.supplyChain.repo = value
        case "supplychain.revision", "revision":
            result.supplyChain.revision = value
        case "supplychain.digest", "digest":
            result.supplyChain.digest = value
        case "supplychain.slsalevel", "slsalevel":
            result.supplyChain.slsaLevel = Int(value)
        // capabilities
        case "capabilities.requiresnetwork", "requiresnetwork":
            result.capabilities.requiresNetwork = value.lowercased() == "true"
        case "capabilities.requiresfilesystem", "requiresfilesystem":
            result.capabilities.requiresFilesystem = value.lowercased() != "false"
        case "capabilities.requiresgpu", "requiresgpu":
            result.capabilities.requiresGpu = value.lowercased() == "true"
        // --- legacy asm-compat ---
        case "effort":
            result.effort = value
        case "asc":
            if value.contains("[") {
                result.asc = parseInlineArray(value)
            } else if !value.isEmpty {
                result.asc = [value]
            }
        case "lifecycle":
            result.lifecycle = value.lowercased()
        case "ontology_ref", "ontologyref":
            result.ontologyRef = value
        default:
            break
        }
    }

    private func flushMultiline(key: inout String?, items: inout [String], result: inout ParsedFrontmatter) {
        guard let k = key else { return }
        switch k.lowercased() {
        case "tools", "allowed_tools", "allowedtools":
            result.allowedTools = items
        case "asc":
            result.asc = items
        case "keywords":
            result.keywords = items
        case "authors":
            result.authors = items
            if result.creator.isEmpty, let first = items.first { result.creator = first }
        case "ontology.subjects", "subjects":
            result.ontology.subjects = items
        case "ontology.produces", "produces":
            result.ontology.produces = items
        case "ontology.consumes", "consumes":
            result.ontology.consumes = items
        case "ontology.related", "related":
            result.ontology.related = items
        case "ontology.triggers", "triggers":
            result.ontology.triggers = items
        case "ontology.exclusiveof", "exclusiveof":
            result.ontology.exclusiveOf = items
        case "governance.prooftypes", "prooftypes":
            result.governance.proofTypes = items
        case "governance.enforcementpoints", "enforcementpoints":
            result.governance.enforcementPoints = items
        case "governance.emergencyprotocols", "emergencyprotocols":
            result.governance.emergencyProtocols = items
        case "policy.allowedoutputs", "allowedoutputs":
            result.policy.allowedOutputs = items
        default:
            break
        }
        key = nil
        items = []
    }
}
