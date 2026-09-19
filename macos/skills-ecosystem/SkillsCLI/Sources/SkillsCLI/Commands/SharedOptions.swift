import Foundation
import ArgumentParser
import SkillsCore

struct SharedOptions: ParsableArguments {
    @Flag(name: .long, help: "Output as JSON")
    var json: Bool = false

    @Flag(name: .long, help: "Stable machine-readable JSON envelope (v1)")
    var machine: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    @Option(name: [.customShort("s"), .customLong("scope")], help: "Filter: global, project, or both")
    var scope: String = "both"

    @Option(name: [.customShort("p"), .customLong("tool")], help: "Filter by provider/tool name")
    var tool: String = ""

    @Option(name: .long, help: "Sort by: name, version, or location")
    var sort: String = "name"

    @Flag(name: .long, help: "Show one row per tool instance")
    var flat: Bool = false

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation prompts")
    var yes: Bool = false

    @Flag(name: [.customShort("V"), .customLong("verbose")], help: "Show debug output")
    var verbose: Bool = false

    var formatter: OutputFormatter {
        OutputFormatter(noColor: noColor, json: json, machine: machine)
    }

    var scanOptions: ScanOptions {
        ScanOptions(
            scope: scope,
            providerFilter: tool.isEmpty ? nil : tool,
            includeDisabled: false
        )
    }
}
