import AppKit
import Darwin

final class ZaloLauncher {
    private static let forceAfter: TimeInterval = 5
    private static let giveUpAfter: TimeInterval = 7
    private static let automationDenied = -1743

    private var pids: [String: pid_t] = [:]
    private var starts: [String: UInt64] = [:]
    private var processes: [String: Process] = [:]
    private var stopping: Set<String> = []
    private var waiters: [String: [(Bool) -> Void]] = [:]
    private var warnedAutomation = false
    private let store = Store.shared

    init() {
        NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didTerminateApplicationNotification, object: nil, queue: .main
        ) { [weak self] note in
            guard let self = self,
                  let app = note.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication,
                  let id = self.pids.first(where: { $0.value == app.processIdentifier })?.key
            else { return }
            self.markExited(id)
        }
    }

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

    func isAppStoreBuild(_ app: URL) -> Bool {
        FileManager.default.fileExists(atPath: app.appendingPathComponent("Contents/_MASReceipt").path)
    }

    static func startTime(_ pid: pid_t) -> UInt64? {
        var info = proc_bsdinfo()
        let size = Int32(MemoryLayout<proc_bsdinfo>.size)
        guard proc_pidinfo(pid, PROC_PIDTBSDINFO, 0, &info, size) == size else { return nil }
        return info.pbi_start_tvsec &* 1_000_000 &+ info.pbi_start_tvusec
    }

    private func alive(_ id: String) -> Bool {
        guard let pid = pids[id] else { return false }
        guard let start = starts[id] else { return kill(pid, 0) == 0 }
        return Self.startTime(pid) == start
    }

    func isRunning(_ id: String) -> Bool {
        alive(id)
    }

    var runningCount: Int {
        store.accounts.filter { isRunning($0.id) }.count
    }

    private func pidFile(_ id: String) -> URL {
        store.profileURL(id).appendingPathComponent("zalo.pid")
    }

    private func writePidFile(_ id: String, pid: pid_t, start: UInt64?) {
        let text = start.map { "\(pid) \($0)" } ?? String(pid)
        do {
            try text.write(to: pidFile(id), atomically: true, encoding: .utf8)
        } catch {
            writeLog("Không ghi được file pid của \(id): \(error)")
        }
    }

    func restore() {
        guard let exe = zaloApp().flatMap({ Bundle(url: $0)?.executableURL?.resolvingSymlinksInPath() }) else { return }
        for account in store.accounts {
            let file = pidFile(account.id)
            guard let text = try? String(contentsOf: file, encoding: .utf8) else { continue }
            let parts = text.split(whereSeparator: { $0 == " " || $0.isNewline })
            guard let first = parts.first, let pid = pid_t(first) else {
                try? FileManager.default.removeItem(at: file)
                continue
            }
            let saved = parts.count > 1 ? UInt64(parts[1]) : nil
            let actual = Self.startTime(pid)
            let app = NSRunningApplication(processIdentifier: pid)
            let sameExe = app?.executableURL?.resolvingSymlinksInPath() == exe
            let sameStart = saved == nil || (actual != nil && actual == saved)
            guard actual != nil, sameExe, sameStart else {
                try? FileManager.default.removeItem(at: file)
                writeLog("Bỏ file pid cũ của \(account.id) (pid \(pid) không còn là phiên Zalo này)")
                continue
            }
            pids[account.id] = pid
            starts[account.id] = actual
            if saved == nil { writePidFile(account.id, pid: pid, start: actual) }
            writeLog("Nhận lại phiên \(account.id) (pid \(pid))")
        }
    }

    func start(_ account: Account) throws {
        if isRunning(account.id) {
            show(account.id)
            return
        }
        if pids[account.id] != nil { markExited(account.id) }
        guard let app = zaloApp(), let exe = Bundle(url: app)?.executableURL else {
            throw LaunchError.zaloNotFound
        }

        let home = store.profileURL(account.id)
        try prepareHome(home)

        var env = ProcessInfo.processInfo.environment
        env["CFFIXED_USER_HOME"] = home.path
        env["HOME"] = home.path
        env["TMPDIR"] = home.appendingPathComponent("tmp").path + "/"
        env["__CFBundleIdentifier"] = nil
        env["XPC_SERVICE_NAME"] = nil

        let id = account.id
        let process = Process()
        process.executableURL = exe
        process.environment = env
        process.currentDirectoryURL = home
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        process.terminationHandler = { [weak self] p in
            let pid = p.processIdentifier
            DispatchQueue.main.async {
                guard let self = self, self.pids[id] == pid else { return }
                self.markExited(id)
            }
        }
        try process.run()

        let pid = process.processIdentifier
        let start = Self.startTime(pid)
        pids[id] = pid
        starts[id] = start
        processes[id] = process
        writePidFile(id, pid: pid, start: start)
        store.setReopen(id, true)
        writeLog("Mở Zalo cho \(id) (pid \(pid))")
    }

    private func markExited(_ id: String) {
        guard let pid = pids.removeValue(forKey: id) else { return }
        starts[id] = nil
        processes[id] = nil
        stopping.remove(id)
        try? FileManager.default.removeItem(at: pidFile(id))
        writeLog("Phiên \(id) đã thoát (pid \(pid))")
        finish(id, exited: true)
    }

    private func finish(_ id: String, exited: Bool) {
        let list = waiters.removeValue(forKey: id) ?? []
        list.forEach { $0(exited) }
    }

    func stop(_ id: String, completion: ((Bool) -> Void)? = nil) {
        store.setReopen(id, false)
        guard let pid = pids[id] else {
            completion?(true)
            return
        }
        guard alive(id) else {
            markExited(id)
            completion?(true)
            return
        }
        if let completion = completion { waiters[id, default: []].append(completion) }
        guard !stopping.contains(id) else { return }
        stopping.insert(id)
        if let app = NSRunningApplication(processIdentifier: pid) {
            app.terminate()
        } else if let process = processes[id], process.isRunning {
            process.terminate()
        }
        poll(id, pid: pid, since: Date(), forced: false)
    }

    private func poll(_ id: String, pid: pid_t, since: Date, forced: Bool) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
            guard let self = self, self.pids[id] == pid, self.stopping.contains(id) else { return }
            if !self.alive(id) {
                self.markExited(id)
                return
            }
            let elapsed = Date().timeIntervalSince(since)
            var forced = forced
            if !forced && elapsed >= Self.forceAfter {
                forced = true
                writeLog("Buộc tắt phiên \(id)")
                if let app = NSRunningApplication(processIdentifier: pid) {
                    app.forceTerminate()
                } else {
                    kill(pid, SIGKILL)
                }
            }
            if elapsed >= Self.giveUpAfter {
                self.stopping.remove(id)
                writeLog("Phiên \(id) chưa thoát sau \(Int(Self.giveUpAfter)) giây")
                self.finish(id, exited: false)
                return
            }
            self.poll(id, pid: pid, since: since, forced: forced)
        }
    }

    func stopAll(rememberForNextLaunch: Bool, completion: (() -> Void)? = nil) {
        let group = DispatchGroup()
        for id in Array(pids.keys) {
            let reopen = store.account(id)?.reopen ?? false
            group.enter()
            stop(id) { _ in group.leave() }
            if rememberForNextLaunch { store.setReopen(id, reopen) }
        }
        group.notify(queue: .main) { completion?() }
    }

    func show(_ id: String) {
        guard alive(id), let pid = pids[id], let app = NSRunningApplication(processIdentifier: pid) else { return }
        app.unhide()
        app.activate(options: [.activateAllWindows, .activateIgnoringOtherApps])
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
        do {
            _ = try event.sendEvent(options: [.noReply], timeout: 2)
        } catch {
            let code = (error as NSError).code
            writeLog("Không gửi được yêu cầu mở lại cửa sổ tới Zalo (pid \(pid)): mã \(code)")
            if code == Self.automationDenied && !warnedAutomation {
                warnedAutomation = true
                Prompt.info(
                    "Betacom chưa có quyền điều khiển Zalo",
                    "Nếu cửa sổ Zalo đã đóng, Betacom cần quyền này để mở lại. Vào Cài đặt hệ thống → Quyền riêng tư & Bảo mật → Tự động hóa, bật Zalo trong mục Betacom."
                )
            }
        }
    }

    private func prepareHome(_ home: URL) throws {
        let fm = FileManager.default
        for sub in ["Library/Application Support", "Library/Caches", "Library/Preferences", "Library/Logs", "tmp", "Downloads", "Documents"] {
            try fm.createDirectory(at: home.appendingPathComponent(sub), withIntermediateDirectories: true)
        }
        linkKeychains(home)
    }

    private func linkKeychains(_ home: URL) {
        let fm = FileManager.default
        let link = home.appendingPathComponent("Library/Keychains")
        let real = fm.homeDirectoryForCurrentUser.appendingPathComponent("Library/Keychains")
        let type = (try? fm.attributesOfItem(atPath: link.path))?[.type] as? FileAttributeType
        if let type = type {
            guard type == .typeSymbolicLink else {
                writeLog("\(link.path) không phải symlink, giữ nguyên")
                return
            }
            if (try? fm.destinationOfSymbolicLink(atPath: link.path)) == real.path { return }
            do {
                try fm.removeItem(at: link)
            } catch {
                writeLog("Không xóa được symlink Keychain sai tại \(link.path): \(error)")
                return
            }
            writeLog("Tạo lại symlink Keychain tại \(link.path)")
        }
        do {
            try fm.createSymbolicLink(at: link, withDestinationURL: real)
        } catch {
            writeLog("Không tạo được symlink Keychain tại \(link.path): \(error)")
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
