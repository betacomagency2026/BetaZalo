import Foundation

/// Mở MultiZ cùng máy Mac bằng một LaunchAgent nhỏ trong ~/Library/LaunchAgents.
/// Cách này chạy được từ macOS 12 và không cần quyền đặc biệt.
enum LoginItem {
    private static let label = "vn.sudo.multiz"

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

    /// Ghi lại đường dẫn hiện tại của MultiZ (phòng khi người dùng đã chuyển app sang thư mục khác).
    /// Đồng thời chuyển mục "mở cùng máy" của bản cũ (Zaloo) sang tên mới.
    static func refreshIfEnabled() {
        let old = plistURL.deletingLastPathComponent().appendingPathComponent("vn.sudo.zaloo.plist")
        if FileManager.default.fileExists(atPath: old.path) {
            try? FileManager.default.removeItem(at: old)
            write()
            return
        }
        if isEnabled { write() }
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
