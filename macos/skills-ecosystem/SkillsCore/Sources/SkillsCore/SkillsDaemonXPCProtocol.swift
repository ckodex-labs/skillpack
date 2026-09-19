import Foundation

@objc(SkillsDaemonXPCProtocol)
public protocol SkillsDaemonXPCProtocol {
    func syncSkills(dryRun: Bool, noIndex: Bool, onlyAgent: String?, withReply reply: @escaping (Bool, String, Int32) -> Void)
    func getStatus(withReply reply: @escaping (String, Int32, [String]) -> Void)
    func getExtendedStatus(withReply reply: @escaping (String, Int32, [String], String, String, String) -> Void)
    func checkBoundary(targetPath: String, withReply reply: @escaping (Bool, String) -> Void)
}
