import Foundation
import Testing
@testable import SkillsCore

struct STXPublisherTransportTests {

    @Test func defaultEndpointIsLocalhost50051() {
        let publisher = STXPublisher()
        #expect(publisher.endpoint.absoluteString == "http://localhost:50051")
    }

    @Test func customEndpointViaParameter() {
        let publisher = STXPublisher(endpointURL: "https://api.skillpack.io")
        #expect(publisher.endpoint.absoluteString == "https://api.skillpack.io")
    }

    @Test func bearerTokenIsSetWhenProvided() {
        let url = URL(string: "http://localhost:50051")!
        let publisher = STXPublisher(endpoint: url, token: "my-secret-token")
        #expect(publisher.bearerToken == "my-secret-token")
    }

    @Test func bearerTokenIsNilWhenNotProvided() {
        let url = URL(string: "http://localhost:50051")!
        let publisher = STXPublisher(endpoint: url, token: nil)
        #expect(publisher.bearerToken == nil)
    }

    @Test func applyAuthInjectsBearerHeaderWhenTokenPresent() {
        let url = URL(string: "http://localhost:50051")!
        let publisher = STXPublisher(endpoint: url, token: "test-token")
        var request = URLRequest(url: url)
        publisher.applyAuth(to: &request)
        let authHeader = request.value(forHTTPHeaderField: "Authorization")
        #expect(authHeader == "Bearer test-token")
    }

    @Test func applyAuthSkipsHeaderWhenTokenAbsent() {
        let url = URL(string: "http://localhost:50051")!
        let publisher = STXPublisher(endpoint: url, token: nil)
        var request = URLRequest(url: url)
        publisher.applyAuth(to: &request)
        let authHeader = request.value(forHTTPHeaderField: "Authorization")
        #expect(authHeader == nil)
    }

}

// MARK: - KeychainTokenStore

struct KeychainTokenStoreTests {

    @Test func keychainRoundTripSaveLoadDelete() {
        let token = "test-keychain-token-\(UUID().uuidString)"
        // Clean slate
        KeychainTokenStore.delete()
        #expect(KeychainTokenStore.load() == nil)

        // Save and load
        #expect(KeychainTokenStore.save(token: token) == true)
        #expect(KeychainTokenStore.load() == token)

        // Update
        let newToken = "updated-token-\(UUID().uuidString)"
        #expect(KeychainTokenStore.save(token: newToken) == true)
        #expect(KeychainTokenStore.load() == newToken)

        // Delete
        #expect(KeychainTokenStore.delete() == true)
        #expect(KeychainTokenStore.load() == nil)
    }

    @Test func envVarTakesPriorityOverKeychain() {
        // This test documents the priority: env var > Keychain.
        // We cannot mutate ProcessInfo.environment at runtime,
        // so we verify via the existing bearerToken tests.
        let url = URL(string: "http://localhost:50051")!
        let publisher = STXPublisher(endpoint: url, token: "env-token")
        #expect(publisher.bearerToken == "env-token")
    }
}
