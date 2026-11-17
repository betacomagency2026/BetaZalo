import Foundation

enum LoginItem {
    private static let label = "com.betacom.betazalo"

    private static var plistURL: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/LaunchAgents", isDirectory: true)
            .appendingPathComponent("\(label).plist")
    }

    static var isEnabled: Bool {
        FileManager.default.fileExists(atPath: plistURL.path)
    }

    static var isTranslocated: Bool {
        Bundle.main.bundlePath.contains("/AppTranslocation/")
    }

    @discardableResult
    static func setEnabled(_ on: Bool) -> Bool {
        if on { return write() }
        do {
            try FileManager.default.removeItem(at: plistURL)
        } catch {
            writeLog("Không xóa được LaunchAgent: \(error)")
        }
        return true
    }

    static func refreshIfEnabled() {
        let dir = plistURL.deletingLastPathComponent()
        var hadOld = false
        for name in ["vn.sudo.zaloo", "vn.sudo.multiz"] {
            let old = dir.appendingPathComponent("\(name).plist")
            if FileManager.default.fileExists(atPath: old.path) {
                try? FileManager.default.removeItem(at: old)
                hadOld = true
            }
        }
        if hadOld || isEnabled { write() }
    }

    @discardableResult
    private static func write() -> Bool {
        if isTranslocated {
            writeLog("Không ghi LaunchAgent vì Betacom đang chạy từ App Translocation: \(Bundle.main.bundlePath)")
            return false
        }
        guard let exe = Bundle.main.executablePath else { return false }
        let plist: [String: Any] = [
            "Label": label,
            "ProgramArguments": [exe],
            "RunAtLoad": true,
            "ProcessType": "Interactive",
            "AbandonProcessGroup": true,
        ]
        do {
            try FileManager.default.createDirectory(at: plistURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            let data = try PropertyListSerialization.data(fromPropertyList: plist, format: .xml, options: 0)
            try data.write(to: plistURL, options: .atomic)
            return true
        } catch {
            writeLog("Không ghi được LaunchAgent: \(error)")
            return false
        }
    }
}
