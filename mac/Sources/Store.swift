import Foundation

struct Account: Codable, Equatable {
    var id: String
    var name: String
    var color: Int
    /// Mở lại tài khoản này khi Betacom khởi động (tự bật khi mở, tự tắt khi bấm "Tắt").
    var reopen: Bool

    init(id: String, name: String, color: Int, reopen: Bool = true) {
        self.id = id
        self.name = name
        self.color = color
        self.reopen = reopen
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        id = try c.decode(String.self, forKey: .id)
        name = try c.decode(String.self, forKey: .name)
        color = try c.decodeIfPresent(Int.self, forKey: .color) ?? 0
        reopen = try c.decodeIfPresent(Bool.self, forKey: .reopen) ?? true
    }
}

/// Lưu danh sách tài khoản và cài đặt trong ~/Library/Application Support/Betacom/config.json
final class Store {
    static let shared = Store()

    let root: URL
    private(set) var accounts: [Account] = []
    var zaloPath: String? {
        didSet { save() }
    }

    private struct ConfigFile: Codable {
        var accounts: [Account]
        var zaloPath: String?
    }

    private var configURL: URL { root.appendingPathComponent("config.json") }

    /// ~/Library/Application Support/Betacom
    static let rootURL: URL = {
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let url = support.appendingPathComponent("Betacom", isDirectory: true)
        let fm = FileManager.default
        for name in ["MultiZ", "Zaloo"] where !fm.fileExists(atPath: url.path) {
            let old = support.appendingPathComponent(name, isDirectory: true)
            if fm.fileExists(atPath: old.path) {
                try? fm.moveItem(at: old, to: url)
            }
        }
        try? fm.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }()

    private init() {
        root = Store.rootURL
        load()
    }

    private func load() {
        guard let data = try? Data(contentsOf: configURL) else { return }
        do {
            let file = try JSONDecoder().decode(ConfigFile.self, from: data)
            accounts = file.accounts
            zaloPath = file.zaloPath
        } catch {
            writeLog("config.json lỗi, bắt đầu với cấu hình trống: \(error)")
            try? FileManager.default.copyItem(at: configURL, to: root.appendingPathComponent("config.broken.json"))
        }
    }

    func save() {
        let file = ConfigFile(accounts: accounts, zaloPath: zaloPath)
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        guard let data = try? encoder.encode(file) else { return }
        try? data.write(to: configURL, options: .atomic)
    }

    func profileURL(_ id: String) -> URL {
        root.appendingPathComponent("profiles", isDirectory: true).appendingPathComponent(id, isDirectory: true)
    }

    func account(_ id: String) -> Account? {
        accounts.first { $0.id == id }
    }

    @discardableResult
    func addAccount(name: String) -> Account {
        let id = "acc" + String(Int(Date().timeIntervalSince1970 * 1000), radix: 16)
        let account = Account(id: id, name: name, color: accounts.count % Avatar.palette.count)
        accounts.append(account)
        save()
        return account
    }

    func update(_ account: Account) {
        guard let i = accounts.firstIndex(where: { $0.id == account.id }) else { return }
        accounts[i] = account
        save()
    }

    func setReopen(_ id: String, _ value: Bool) {
        guard var a = account(id), a.reopen != value else { return }
        a.reopen = value
        update(a)
    }

    func removeAccount(_ id: String) {
        accounts.removeAll { $0.id == id }
        save()
        // Cho vào Thùng rác thay vì xóa hẳn, phòng khi người dùng bấm nhầm.
        let profile = profileURL(id)
        if FileManager.default.fileExists(atPath: profile.path) {
            try? FileManager.default.trashItem(at: profile, resultingItemURL: nil)
        }
    }
}

/// Nhật ký kỹ thuật đơn giản (không ghi nội dung tin nhắn).
func writeLog(_ message: String) {
    let url = Store.rootURL.appendingPathComponent("log.txt")
    let formatter = DateFormatter()
    formatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
    let line = "\(formatter.string(from: Date()))  \(message)\n"
    guard let data = line.data(using: .utf8) else { return }

    if let attrs = try? FileManager.default.attributesOfItem(atPath: url.path),
       let size = attrs[.size] as? NSNumber, size.intValue > 1_000_000 {
        try? FileManager.default.removeItem(at: url)
    }
    if let handle = try? FileHandle(forWritingTo: url) {
        handle.seekToEndOfFile()
        handle.write(data)
        handle.closeFile()
    } else {
        try? data.write(to: url)
    }
}
