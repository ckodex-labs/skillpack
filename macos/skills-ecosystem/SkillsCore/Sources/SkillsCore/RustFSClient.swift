import Foundation

public struct RustFSClient {
    private let endpoint: URL
    private let session: URLSession
    private let logger: (String) -> Void
    
    public init(endpointURL: String? = nil, session: URLSession = .shared, logger: @escaping (String) -> Void = { _ in }) {
        let envEndpoint = ProcessInfo.processInfo.environment["RUSTFS_ENDPOINT"] ?? "http://localhost:9000"
        let urlStr = endpointURL ?? envEndpoint
        self.endpoint = URL(string: urlStr) ?? URL(string: "http://localhost:9000")!
        self.session = session
        self.logger = logger
    }
    
    public func putObject(bucket: String, key: String, data: Data) throws {
        let objectURL = endpoint.appendingPathComponent(bucket).appendingPathComponent(key)
        var request = URLRequest(url: objectURL)
        request.httpMethod = "PUT"
        request.httpBody = data
        request.setValue("text/plain", forHTTPHeaderField: "Content-Type")
        
        let semaphore = DispatchSemaphore(value: 0)
        var taskError: Error?
        var statusCode: Int = 0
        
        let task = session.dataTask(with: request) { _, response, error in
            if let error = error {
                taskError = error
            } else if let httpResponse = response as? HTTPURLResponse {
                statusCode = httpResponse.statusCode
            }
            semaphore.signal()
        }
        task.resume()
        semaphore.wait()
        
        if let error = taskError {
            throw error
        }
        
        guard statusCode >= 200 && statusCode < 300 else {
            throw SkillsError.custom("RustFS PUT failed with HTTP status \(statusCode) for key \(key)")
        }
    }
    
    public func getObject(bucket: String, key: String) throws -> Data? {
        let objectURL = endpoint.appendingPathComponent(bucket).appendingPathComponent(key)
        var request = URLRequest(url: objectURL)
        request.httpMethod = "GET"
        
        let semaphore = DispatchSemaphore(value: 0)
        var taskError: Error?
        var taskData: Data?
        var statusCode: Int = 0
        
        let task = session.dataTask(with: request) { data, response, error in
            if let error = error {
                taskError = error
            } else if let httpResponse = response as? HTTPURLResponse {
                statusCode = httpResponse.statusCode
                if statusCode == 200 {
                    taskData = data
                }
            }
            semaphore.signal()
        }
        task.resume()
        semaphore.wait()
        
        if let error = taskError {
            throw error
        }
        
        if statusCode == 404 {
            return nil
        }
        
        guard statusCode >= 200 && statusCode < 300 else {
            throw SkillsError.custom("RustFS GET failed with HTTP status \(statusCode) for key \(key)")
        }
        
        return taskData
    }
    
    public func deleteObject(bucket: String, key: String) throws {
        let objectURL = endpoint.appendingPathComponent(bucket).appendingPathComponent(key)
        var request = URLRequest(url: objectURL)
        request.httpMethod = "DELETE"
        
        let semaphore = DispatchSemaphore(value: 0)
        var taskError: Error?
        var statusCode: Int = 0
        
        let task = session.dataTask(with: request) { _, response, error in
            if let error = error {
                taskError = error
            } else if let httpResponse = response as? HTTPURLResponse {
                statusCode = httpResponse.statusCode
            }
            semaphore.signal()
        }
        task.resume()
        semaphore.wait()
        
        if let error = taskError {
            throw error
        }
        
        guard (statusCode >= 200 && statusCode < 300) || statusCode == 404 else {
            throw SkillsError.custom("RustFS DELETE failed with HTTP status \(statusCode) for key \(key)")
        }
    }
    
    public func checkHealth(bucket: String = "skills") -> Bool {
        let objectURL = endpoint.appendingPathComponent(bucket).appendingPathComponent("healthcheck")
        var request = URLRequest(url: objectURL)
        request.httpMethod = "GET"
        request.timeoutInterval = 2.0
        
        let semaphore = DispatchSemaphore(value: 0)
        var taskError: Error?
        var statusCode: Int = 0
        
        let task = session.dataTask(with: request) { _, response, error in
            if let error = error {
                taskError = error
            } else if let httpResponse = response as? HTTPURLResponse {
                statusCode = httpResponse.statusCode
            }
            semaphore.signal()
        }
        task.resume()
        
        let waitResult = semaphore.wait(timeout: .now() + .seconds(2))
        if waitResult == .timedOut {
            task.cancel()
            return false
        }
        
        if taskError != nil {
            return false
        }
        
        return statusCode > 0
    }
}
