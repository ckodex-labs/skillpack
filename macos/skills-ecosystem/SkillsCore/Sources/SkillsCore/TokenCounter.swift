import Foundation

public struct TokenCounter {
    public init() {}

    /// Estimate token count using word+space heuristic (mirrors asm's `estimateTokenCount`).
    /// Returns words.count + spaces.count which approximates BPE token count.
    public func estimate(text: String) -> Int {
        let words = text.components(separatedBy: .whitespacesAndNewlines).filter { !$0.isEmpty }
        let spaces = max(0, words.count - 1)
        return words.count + spaces
    }

    public func estimate(url: URL) -> Int {
        guard let content = try? String(contentsOf: url, encoding: .utf8) else { return 0 }
        return estimate(text: content)
    }
}
