import Foundation

public struct LLMClient {
    private let apiKey: String
    private let session: URLSession
    private let logger: (String) -> Void
    
    public init(apiKey: String, session: URLSession = .shared, logger: @escaping (String) -> Void = { _ in }) {
        self.apiKey = apiKey
        self.session = session
        self.logger = logger
    }
    
    public func generateContent(prompt: String) throws -> String {
        let endpoint = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key=\(apiKey)"
        guard let url = URL(string: endpoint) else {
            throw SkillsError.llmRequestFailed(reason: "Invalid Gemini endpoint URL")
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = 60.0
        
        let requestBody: [String: Any] = [
            "contents": [
                [
                    "parts": [
                        ["text": prompt]
                    ]
                ]
            ],
            "generationConfig": [
                "responseMimeType": "application/json"
            ]
        ]
        
        do {
            request.httpBody = try JSONSerialization.data(withJSONObject: requestBody)
        } catch {
            throw SkillsError.llmRequestFailed(reason: "Failed to serialize request JSON: \(error.localizedDescription)")
        }
        
        logger("  ✓ Dispatching REST request to Gemini API (gemini-2.5-flash)...")
        
        var responseData: Data?
        var responseError: Error?
        var statusCode = 0
        
        let semaphore = DispatchSemaphore(value: 0)
        
        let task = self.session.dataTask(with: request) { data, response, error in
            responseData = data
            responseError = error
            if let httpResponse = response as? HTTPURLResponse {
                statusCode = httpResponse.statusCode
            }
            semaphore.signal()
        }
        task.resume()
        semaphore.wait()
        
        if let error = responseError {
            throw SkillsError.llmRequestFailed(reason: "Network error: \(error.localizedDescription)")
        }
        
        guard let data = responseData else {
            throw SkillsError.llmRequestFailed(reason: "No response data received from API")
        }
        
        if statusCode != 200 {
            let errorResponse = String(data: data, encoding: .utf8) ?? "HTTP \(statusCode)"
            throw SkillsError.llmRequestFailed(reason: "API returned status \(statusCode): \(errorResponse)")
        }
        
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let candidates = json["candidates"] as? [[String: Any]],
              let firstCandidate = candidates.first,
              let content = firstCandidate["content"] as? [String: Any],
              let parts = content["parts"] as? [[String: Any]],
              let firstPart = parts.first,
              let text = firstPart["text"] as? String else {
            throw SkillsError.llmRequestFailed(reason: "Unexpected response format from Gemini API: \(String(data: data, encoding: .utf8) ?? "nil")")
        }
        
        return text
    }
}
