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

    static func setEnabled(_ on: Bool) -> Bool {
        if on { return write() }
        do {
            try FileManager.default.removeItem(at: plistURL)
            return true
        } catch {
            writeLog("Không xóa được LaunchAgent: \(error)")
            return !isEnabled
        }
    }

    static func refreshIfEnabled() {
        let fm = FileManager.default
        let dir = plistURL.deletingLastPathComponent()
        let olds = ["vn.sudo.zaloo", "vn.sudo.multiz"]
            .map { dir.appendingPathComponent("\($0).plist") }
            .filter { fm.fileExists(atPath: $0.path) }
        guard !olds.isEmpty || isEnabled else { return }
        guard write() else {
            if !olds.isEmpty {
                writeLog("Giữ LaunchAgent cũ vì chưa ghi được LaunchAgent của Betacom")
            }
            return
        }
        for old in olds {
            do {
                try fm.removeItem(at: old)
                writeLog("Đã gỡ LaunchAgent cũ \(old.lastPathComponent)")
            } catch {
                writeLog("Không gỡ được LaunchAgent cũ \(old.lastPathComponent): \(error)")
            }
        }
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
            let data = try PropertyListSerialization.data(fromPropertyList: plist, format: .xml, options: 0)
            if let current = try? Data(contentsOf: plistURL), current == data { return true }
            try FileManager.default.createDirectory(at: plistURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            try data.write(to: plistURL, options: .atomic)
            return true
        } catch {
            writeLog("Không ghi được LaunchAgent: \(error)")
            return false
        }
    }
}
