import Foundation

#if canImport(FoundationModels)
import FoundationModels
#endif

public struct AppleFoundationModelsClient {
    private let logger: (String) -> Void
    
    public init(logger: @escaping (String) -> Void = { _ in }) {
        self.logger = logger
    }
    
    public var isAvailable: Bool {
        if ProcessInfo.processInfo.environment["DISABLE_APPLE_INTELLIGENCE"] != nil {
            return false
        }
        #if canImport(FoundationModels)
        if #available(macOS 26.0, iOS 18.0, *) {
            switch SystemLanguageModel.default.availability {
            case .available:
                return true
            default:
                return false
            }
        }
        #endif
        return false
    }
    
    public func generateContent(prompt: String) throws -> String {
        #if canImport(FoundationModels)
        if #available(macOS 26.0, iOS 18.0, *) {
            logger("  [Apple Intelligence] Initializing on-device LLM session...")
            
            let session = LanguageModelSession()
            
            logger("  [Apple Intelligence] Running local inference...")
            let semaphore = DispatchSemaphore(value: 0)
            var responseText = ""
            var sessionError: Error?
            
            Task {
                do {
                    let result = try await session.respond(to: prompt)
                    responseText = result.content
                } catch {
                    sessionError = error
                }
                semaphore.signal()
            }
            
            semaphore.wait()
            
            if let error = sessionError {
                throw error
            }
            
            logger("  [Apple Intelligence] local inference complete.")
            return responseText
        }
        #endif
        throw SkillsError.custom("Apple Intelligence FoundationModels is not available on this system.")
    }
}
