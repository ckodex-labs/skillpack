import Foundation
import Security

/// CLIENT-SPEC.md §6.1: macOS Keychain token storage.
///
/// Wraps Security framework calls to read/write the SkillPack bearer token
/// in the user's default keychain. Service identifier: `io.ckodex.skillpack`.
public enum KeychainTokenStore {
    private static let service = "io.ckodex.skillpack"
    private static let account = "skillpack-token"

    /// Save or update the token in the default keychain.
    /// - Returns: `true` on success.
    @discardableResult
    public static func save(token: String) -> Bool {
        delete()
        guard let data = token.data(using: .utf8) else { return false }
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
        ]
        return SecItemAdd(query as CFDictionary, nil) == errSecSuccess
    }

    /// Load the token from the default keychain.
    /// - Returns: The token string, or `nil` if not found.
    public static func load() -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard status == errSecSuccess, let data = result as? Data else { return nil }
        return String(data: data, encoding: .utf8)
    }

    /// Delete the token from the default keychain.
    /// - Returns: `true` on success or if the item was not found.
    @discardableResult
    public static func delete() -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
        let status = SecItemDelete(query as CFDictionary)
        return status == errSecSuccess || status == errSecItemNotFound
    }
}
