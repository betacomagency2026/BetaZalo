import Foundation

struct Account: Codable, Equatable {
    var id: String
    var name: String
    var color: Int
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

final class Store {
    static let shared = Store()

    let root: URL
    private(set) var accounts: [Account] = []
    private(set) var readOnly = false
    private var loading = false
    var zaloPath: String? {
        didSet { if !loading { save() } }
    }

    static func isSafeID(_ id: String) -> Bool {
        !id.isEmpty && id.utf8.allSatisfy { c in
            (c >= 48 && c <= 57) || (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || c == 95 || c == 45
        }
    }

    private struct ConfigFile: Codable {
        var accounts: [Account]
        var zaloPath: String?
    }

    var configURL: URL { root.appendingPathComponent("config.json") }

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
        let fm = FileManager.default
        guard fm.fileExists(atPath: configURL.path) else { return }
        let data: Data
        do {
            data = try Data(contentsOf: configURL)
        } catch {
            readOnly = true
            writeLog("Không đọc được config.json, tạm không ghi cấu hình: \(error)")
            return
        }
        do {
            let file = try JSONDecoder().decode(ConfigFile.self, from: data)
            var seen = Set<String>()
            let valid = file.accounts.filter { Store.isSafeID($0.id) && seen.insert($0.id).inserted }
            if valid.count != file.accounts.count {
                let dropped = file.accounts.count - valid.count
                if backupConfig() {
                    writeLog("Bỏ \(dropped) tài khoản có id không hợp lệ hoặc trùng trong config.json")
                } else {
                    readOnly = true
                }
            }
            loading = true
            accounts = valid
            zaloPath = file.zaloPath
            loading = false
        } catch {
            if backupConfig() {
                writeLog("config.json lỗi, bắt đầu với cấu hình trống: \(error)")
            } else {
                readOnly = true
                writeLog("config.json lỗi, tạm không ghi cấu hình: \(error)")
            }
        }
    }

    private func backupConfig() -> Bool {
        let backup = root.appendingPathComponent("config.broken-\(fileStamp()).json")
        do {
            try FileManager.default.copyItem(at: configURL, to: backup)
            writeLog("Đã sao lưu config.json sang \(backup.lastPathComponent)")
            return true
        } catch {
            writeLog("Không sao lưu được config.json: \(error)")
            return false
        }
    }

    func save() {
        guard !readOnly else { return }
        let file = ConfigFile(accounts: accounts, zaloPath: zaloPath)
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        do {
            let data = try encoder.encode(file)
            try data.write(to: configURL, options: .atomic)
        } catch {
            writeLog("Không lưu được config.json: \(error)")
        }
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
    }

    func trashProfile(_ id: String) -> Bool {
        guard Store.isSafeID(id), account(id) == nil else {
            writeLog("Không dọn profile của id \"\(id)\": id không hợp lệ hoặc vẫn đang dùng")
            return false
        }
        let profile = profileURL(id)
        guard FileManager.default.fileExists(atPath: profile.path) else { return true }
        do {
            try FileManager.default.trashItem(at: profile, resultingItemURL: nil)
            return true
        } catch {
            writeLog("Không chuyển \(profile.path) vào Thùng rác: \(error)")
            return false
        }
    }
}

private let logLock = NSLock()

private let logFormatter: DateFormatter = {
    let f = DateFormatter()
    f.locale = Locale(identifier: "en_US_POSIX")
    f.dateFormat = "yyyy-MM-dd HH:mm:ss"
    return f
}()

private let stampFormatter: DateFormatter = {
    let f = DateFormatter()
    f.locale = Locale(identifier: "en_US_POSIX")
    f.dateFormat = "yyyyMMdd-HHmmss"
    return f
}()

func fileStamp() -> String {
    logLock.lock()
    defer { logLock.unlock() }
    return stampFormatter.string(from: Date())
}

func writeLog(_ message: String) {
    logLock.lock()
    defer { logLock.unlock() }
    let fm = FileManager.default
    let url = Store.rootURL.appendingPathComponent("log.txt")
    let line = "\(logFormatter.string(from: Date()))  \(message)\n"
    guard let data = line.data(using: .utf8) else { return }

    if let attrs = try? fm.attributesOfItem(atPath: url.path),
       let size = attrs[.size] as? NSNumber, size.intValue > 1_000_000 {
        let old = Store.rootURL.appendingPathComponent("log.old.txt")
        try? fm.removeItem(at: old)
        try? fm.moveItem(at: url, to: old)
    }
    if let handle = try? FileHandle(forWritingTo: url) {
        _ = try? handle.seekToEnd()
        try? handle.write(contentsOf: data)
        try? handle.close()
    } else {
        try? data.write(to: url)
    }
}
