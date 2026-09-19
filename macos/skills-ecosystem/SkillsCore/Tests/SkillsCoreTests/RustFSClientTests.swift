import Testing
import Foundation
@testable import SkillsCore

@Suite(.serialized) struct RustFSClientTests {
    class MockURLProtocol: URLProtocol {
        static var requestHandler: ((URLRequest) throws -> (HTTPURLResponse, Data?))?
        
        override class func canInit(with request: URLRequest) -> Bool {
            return true
        }
        
        override class func canonicalRequest(for request: URLRequest) -> URLRequest {
            return request
        }
        
        override func startLoading() {
            guard let handler = MockURLProtocol.requestHandler else {
                return
            }
            
            do {
                let (response, data) = try handler(request)
                client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
                if let data = data {
                    client?.urlProtocol(self, didLoad: data)
                }
                client?.urlProtocolDidFinishLoading(self)
            } catch {
                client?.urlProtocol(self, didFailWithError: error)
            }
        }
        
        override func stopLoading() {}
    }
    
    private func makeMockSession() -> URLSession {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [MockURLProtocol.self]
        return URLSession(configuration: configuration)
    }
    
    @Test func testPutObject() throws {
        let session = makeMockSession()
        let client = RustFSClient(endpointURL: "http://localhost:9000", session: session)
        
        let expectedData = Data("my payload content".utf8)
        
        MockURLProtocol.requestHandler = { request in
            #expect(request.url?.absoluteString == "http://localhost:9000/my-bucket/my-key")
            #expect(request.httpMethod == "PUT")
            #expect(request.value(forHTTPHeaderField: "Content-Type") == "text/plain")
            
            let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
            return (response, nil)
        }
        
        try client.putObject(bucket: "my-bucket", key: "my-key", data: expectedData)
    }
    
    @Test func testGetObject() throws {
        let session = makeMockSession()
        let client = RustFSClient(endpointURL: "http://localhost:9000", session: session)
        
        let expectedData = Data("some stored data".utf8)
        
        MockURLProtocol.requestHandler = { request in
            #expect(request.url?.absoluteString == "http://localhost:9000/my-bucket/my-key")
            #expect(request.httpMethod == "GET")
            
            let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
            return (response, expectedData)
        }
        
        let retrieved = try client.getObject(bucket: "my-bucket", key: "my-key")
        #expect(retrieved == expectedData)
    }
    
    @Test func testDeleteObject() throws {
        let session = makeMockSession()
        let client = RustFSClient(endpointURL: "http://localhost:9000", session: session)
        
        MockURLProtocol.requestHandler = { request in
            #expect(request.url?.absoluteString == "http://localhost:9000/my-bucket/my-key")
            #expect(request.httpMethod == "DELETE")
            
            let response = HTTPURLResponse(url: request.url!, statusCode: 204, httpVersion: nil, headerFields: nil)!
            return (response, nil)
        }
        
        try client.deleteObject(bucket: "my-bucket", key: "my-key")
    }
    
    @Test func testCheckHealthSuccess() throws {
        let session = makeMockSession()
        let client = RustFSClient(endpointURL: "http://localhost:9000", session: session)
        
        MockURLProtocol.requestHandler = { request in
            #expect(request.url?.absoluteString == "http://localhost:9000/my-bucket/healthcheck")
            #expect(request.httpMethod == "GET")
            
            let response = HTTPURLResponse(url: request.url!, statusCode: 404, httpVersion: nil, headerFields: nil)!
            return (response, nil)
        }
        
        let isHealthy = client.checkHealth(bucket: "my-bucket")
        #expect(isHealthy == true)
    }
    
    @Test func testCheckHealthFailure() throws {
        let session = makeMockSession()
        let client = RustFSClient(endpointURL: "http://localhost:9000", session: session)
        
        MockURLProtocol.requestHandler = { request in
            throw NSError(domain: NSURLErrorDomain, code: NSURLErrorCannotConnectToHost, userInfo: nil)
        }
        
        let isHealthy = client.checkHealth(bucket: "my-bucket")
        #expect(isHealthy == false)
    }
}
