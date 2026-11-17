import Foundation

/// Mở Betacom cùng máy Mac bằng một LaunchAgent nhỏ trong ~/Library/LaunchAgents.
/// Cách này chạy được từ macOS 12 và không cần quyền đặc biệt.
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

    static func setEnabled(_ on: Bool) {
        if on {
            write()
        } else {
            try? FileManager.default.removeItem(at: plistURL)
        }
    }

    /// Ghi lại đường dẫn hiện tại của Betacom (phòng khi người dùng đã chuyển app sang thư mục khác).
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

    private static func write() {
        guard let exe = Bundle.main.executablePath else { return }
        let plist: [String: Any] = [
            "Label": label,
            "ProgramArguments": [exe],
            "RunAtLoad": true,
            "ProcessType": "Interactive",
        ]
        do {
            try FileManager.default.createDirectory(at: plistURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            let data = try PropertyListSerialization.data(fromPropertyList: plist, format: .xml, options: 0)
            try data.write(to: plistURL, options: .atomic)
        } catch {
            writeLog("Không ghi được LaunchAgent: \(error)")
        }
    }
}
