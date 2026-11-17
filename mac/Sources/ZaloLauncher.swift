import AppKit

/// Mở, tắt và chuyển giữa các phiên Zalo.
///
/// Mỗi tài khoản chạy Zalo gốc với "thư mục nhà" riêng: biến môi trường
/// CFFIXED_USER_HOME khiến Zalo lưu dữ liệu vào ~/Library/Application Support/Betacom/profiles/<id>
/// thay vì thư mục của người dùng, nên các tài khoản không lẫn vào nhau.
final class ZaloLauncher {
    private var pids: [String: pid_t] = [:]
    private let store = Store.shared

    init() {
        NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didTerminateApplicationNotification, object: nil, queue: .main
        ) { [weak self] note in
            guard let self = self,
                  let app = note.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication,
                  let id = self.pids.first(where: { $0.value == app.processIdentifier })?.key
            else { return }
            self.pids[id] = nil
            try? FileManager.default.removeItem(at: self.pidFile(id))
            writeLog("Phiên \(id) đã thoát")
        }
    }

    // MARK: - Tìm Zalo

    func zaloApp() -> URL? {
        var candidates: [URL] = []
        if let custom = store.zaloPath {
            candidates.append(URL(fileURLWithPath: custom))
        }
        candidates.append(URL(fileURLWithPath: "/Applications/Zalo.app"))
        candidates.append(FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Applications/Zalo.app"))
        if let path = NSWorkspace.shared.fullPath(forApplication: "Zalo") {
            candidates.append(URL(fileURLWithPath: path))
        }
        return candidates.first { Bundle(url: $0)?.executableURL != nil }
    }

    /// Bản Zalo tải từ App Store bị macOS giới hạn (sandbox), có thể không tách được dữ liệu.
    func isAppStoreBuild(_ app: URL) -> Bool {
        FileManager.default.fileExists(atPath: app.appendingPathComponent("Contents/_MASReceipt").path)
    }

    // MARK: - Trạng thái

    func isRunning(_ id: String) -> Bool {
        guard let pid = pids[id] else { return false }
        return kill(pid, 0) == 0
    }

    var runningCount: Int {
        store.accounts.filter { isRunning($0.id) }.count
    }

    private func pidFile(_ id: String) -> URL {
        store.profileURL(id).appendingPathComponent("zalo.pid")
    }

    /// Nhận lại các phiên Zalo vẫn đang chạy từ lần mở Betacom trước.
    func restore() {
        guard let exe = zaloApp().flatMap({ Bundle(url: $0)?.executableURL }) else { return }
        for account in store.accounts {
            guard let text = try? String(contentsOf: pidFile(account.id), encoding: .utf8),
                  let pid = pid_t(text.trimmingCharacters(in: .whitespacesAndNewlines)),
                  let app = NSRunningApplication(processIdentifier: pid),
                  app.executableURL?.resolvingSymlinksInPath() == exe.resolvingSymlinksInPath()
            else { continue }
            pids[account.id] = pid
            writeLog("Nhận lại phiên \(account.id) (pid \(pid))")
        }
    }

    // MARK: - Mở / tắt / đưa lên trước

    func start(_ account: Account) throws {
        if isRunning(account.id) {
            show(account.id)
            return
        }
        guard let app = zaloApp(), let exe = Bundle(url: app)?.executableURL else {
            throw LaunchError.zaloNotFound
        }

        let home = store.profileURL(account.id)
        try prepareHome(home)

        var env = ProcessInfo.processInfo.environment
        env["CFFIXED_USER_HOME"] = home.path
        env["HOME"] = home.path
        env["TMPDIR"] = home.appendingPathComponent("tmp").path + "/"
        // Không để Zalo "mượn" danh tính của Betacom.
        env["__CFBundleIdentifier"] = nil
        env["XPC_SERVICE_NAME"] = nil

        let process = Process()
        process.executableURL = exe
        process.environment = env
        process.currentDirectoryURL = home
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        try process.run()

        let pid = process.processIdentifier
        pids[account.id] = pid
        try? String(pid).write(to: pidFile(account.id), atomically: true, encoding: .utf8)
        store.setReopen(account.id, true)
        writeLog("Mở Zalo cho \(account.id) (pid \(pid))")
    }

    func stop(_ id: String) {
        store.setReopen(id, false)
        guard let pid = pids[id] else { return }
        if let app = NSRunningApplication(processIdentifier: pid) {
            app.terminate()
        } else {
            kill(pid, SIGTERM)
        }
        // Zalo không chịu thoát sau vài giây thì buộc tắt.
        DispatchQueue.main.asyncAfter(deadline: .now() + 6) { [weak self] in
            guard let self = self, self.pids[id] == pid, kill(pid, 0) == 0 else { return }
            writeLog("Buộc tắt phiên \(id)")
            NSRunningApplication(processIdentifier: pid)?.forceTerminate()
        }
    }

    func stopAll(rememberForNextLaunch: Bool) {
        for id in pids.keys {
            let reopen = store.account(id)?.reopen ?? false
            stop(id)
            if rememberForNextLaunch { store.setReopen(id, reopen) }
        }
    }

    /// Đưa cửa sổ Zalo của tài khoản lên trước.
    func show(_ id: String) {
        guard let pid = pids[id], let app = NSRunningApplication(processIdentifier: pid) else { return }
        app.unhide()
        app.activate(options: [.activateAllWindows, .activateIgnoringOtherApps])
        // Nếu cửa sổ Zalo đã bị đóng (Zalo vẫn chạy nền), yêu cầu Zalo mở lại cửa sổ.
        // Lần đầu macOS sẽ hỏi quyền "Betacom muốn điều khiển Zalo".
        sendReopen(to: pid)
    }

    private func sendReopen(to pid: pid_t) {
        let target = NSAppleEventDescriptor(processIdentifier: pid)
        let event = NSAppleEventDescriptor(
            eventClass: AEEventClass(kCoreEventClass),
            eventID: AEEventID(kAEReopenApplication),
            targetDescriptor: target,
            returnID: AEReturnID(kAutoGenerateReturnID),
            transactionID: AETransactionID(kAnyTransactionID)
        )
        _ = try? event.sendEvent(options: [.noReply], timeout: 2)
    }

    private func prepareHome(_ home: URL) throws {
        let fm = FileManager.default
        for sub in ["Library/Application Support", "Library/Caches", "Library/Preferences", "Library/Logs", "tmp", "Downloads", "Documents"] {
            try fm.createDirectory(at: home.appendingPathComponent(sub), withIntermediateDirectories: true)
        }
        // Dùng chung Keychain thật của người dùng, nếu không Zalo có thể báo lỗi lưu mật khẩu/khóa.
        let keychains = home.appendingPathComponent("Library/Keychains")
        if (try? keychains.checkResourceIsReachable()) != true {
            let real = fm.homeDirectoryForCurrentUser.appendingPathComponent("Library/Keychains")
            try? fm.createSymbolicLink(at: keychains, withDestinationURL: real)
        }
    }

    enum LaunchError: LocalizedError {
        case zaloNotFound

        var errorDescription: String? {
            switch self {
            case .zaloNotFound:
                return "Chưa tìm thấy Zalo trên máy. Hãy cài Zalo cho Mac, hoặc chọn vị trí Zalo.app trong menu Betacom."
            }
        }
    }
}
