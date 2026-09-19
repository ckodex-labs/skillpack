import Foundation
import ArgumentParser
import SkillsCore

struct EvalCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "eval",
        abstract: "Evaluate a skill against best practices and score it",
        subcommands: [EvalProvidersCommand.self]
    )

    @Argument(help: "Path to skill directory")
    var skillPath: String

    @Flag(name: .long, help: "Output as JSON")
    var json: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let url = URL(fileURLWithPath: (skillPath as NSString).expandingTildeInPath)
        let fm = FileManager.default

        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: url.path, isDirectory: &isDir), isDir.boolValue else {
            fputs("error: '\(skillPath)' is not a directory\n", stderr)
            throw ExitCode.failure
        }

        let mdURL = url.appendingPathComponent("SKILL.md")
        guard fm.fileExists(atPath: mdURL.path) else {
            fputs("error: no SKILL.md found in '\(skillPath)'\n", stderr)
            throw ExitCode.failure
        }

        let parser = FrontmatterParser()
        let fm2 = parser.parse(url: mdURL)
        let counter = TokenCounter()
        let tokens = counter.estimate(url: mdURL)

        var categories: [(id: String, name: String, score: Int, max: Int)] = []
        var totalScore = 0
        let totalMax = 100

        // Category 1: Frontmatter completeness (25 pts)
        var fmScore = 0
        if !fm2.version.isEmpty && fm2.version != "0.0.0" { fmScore += 5 }
        if !fm2.description.isEmpty { fmScore += 5 }
        if !fm2.creator.isEmpty { fmScore += 5 }
        if !fm2.license.isEmpty { fmScore += 5 }
        if !fm2.allowedTools.isEmpty { fmScore += 5 }
        categories.append((id: "frontmatter", name: "Frontmatter", score: fmScore, max: 25))
        totalScore += fmScore

        // Category 2: Content quality (25 pts)
        let content = (try? String(contentsOf: mdURL, encoding: .utf8)) ?? ""
        var contentScore = 0
        let lines = content.components(separatedBy: .newlines)
        if lines.count > 10 { contentScore += 10 }
        if content.contains("## ") { contentScore += 5 }
        if content.contains("Usage") || content.contains("Example") { contentScore += 5 }
        if tokens >= 100 && tokens <= 20000 { contentScore += 5 }
        categories.append((id: "content", name: "Content Quality", score: contentScore, max: 25))
        totalScore += contentScore

        // Category 3: Token size (25 pts)
        var sizeScore = 25
        if tokens > 50000 { sizeScore = 0 }
        else if tokens > 20000 { sizeScore = 10 }
        else if tokens > 10000 { sizeScore = 18 }
        categories.append((id: "size", name: "Token Size", score: sizeScore, max: 25))
        totalScore += sizeScore

        // Category 4: Naming (25 pts)
        var namingScore = 0
        let dirName = url.lastPathComponent
        let nameIsKebab = dirName == dirName.lowercased() && !dirName.contains("_")
        if nameIsKebab { namingScore += 15 }
        if !fm2.name.isEmpty { namingScore += 10 }
        categories.append((id: "naming", name: "Naming", score: namingScore, max: 25))
        totalScore += namingScore

        let grade: String
        switch totalScore {
        case 90...100: grade = "A"
        case 80..<90:  grade = "B"
        case 70..<80:  grade = "C"
        case 60..<70:  grade = "D"
        default:       grade = "F"
        }

        if json {
            struct EvalResult: Codable {
                var skillName: String
                var overallScore: Int
                var grade: String
                var passed: Bool
                var tokenCount: Int
                var categories: [[String: String]]
                var evaluatedAt: String
            }
            let result = EvalResult(
                skillName: dirName,
                overallScore: totalScore,
                grade: grade,
                passed: totalScore >= 60,
                tokenCount: tokens,
                categories: categories.map { ["id": $0.id, "name": $0.name, "score": "\($0.score)", "max": "\($0.max)"] },
                evaluatedAt: ISO8601DateFormatter().string(from: Date())
            )
            let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted]
            if let data = try? enc.encode(result), let str = String(data: data, encoding: .utf8) { print(str) }
        } else {
            let gradeColor: String
            switch grade {
            case "A": gradeColor = "\u{001B}[32m"
            case "B": gradeColor = "\u{001B}[34m"
            case "C": gradeColor = "\u{001B}[33m"
            default:  gradeColor = "\u{001B}[31m"
            }
            print("\n  \u{001B}[1mEval: \(dirName)\u{001B}[0m")
            print("  \(String(repeating: "─", count: 40))")
            print("  Overall score:  \(gradeColor)\(grade)\u{001B}[0m  \(totalScore)/\(totalMax)")
            print("  Tokens:         ~\(tokens)")
            for cat in categories {
                let bar = String(repeating: "█", count: cat.score / 5)
                print("  \(cat.name.padding(toLength: 20, withPad: " ", startingAt: 0))  \(bar.padding(toLength: 5, withPad: "░", startingAt: 0))  \(cat.score)/\(cat.max)")
            }
        }
    }
}

struct EvalProvidersCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "list",
        abstract: "List registered eval providers"
    )

    func run() {
        let providers: [(id: String, version: String, description: String)] = [
            ("quality", "1.0.0", "Built-in quality rubric (frontmatter, content, size, naming)")
        ]
        print("  \u{001B}[1mEval Providers\u{001B}[0m")
        print("  " + String(repeating: "─", count: 50))
        for p in providers {
            print("  \(p.id.padding(toLength: 15, withPad: " ", startingAt: 0))  v\(p.version)  \(p.description)")
        }
    }
}
