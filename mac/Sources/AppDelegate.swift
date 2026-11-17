import AppKit
import UniformTypeIdentifiers

let appVersion = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "0.3.4"

enum Links {
    static let developer = URL(string: "https://github.com/betacomagency2026/BetaZalo")!
    static let update = URL(string: "https://github.com/betacomagency2026/BetaZalo/releases")!
    static let zaloDownload = URL(string: "https://zalo.me/pc")!
}

final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private let store = Store.shared
    private let launcher = ZaloLauncher()
    private var statusItem: NSStatusItem!
    private var stopOnQuit = false

    func applicationDidFinishLaunching(_ notification: Notification) {
        writeLog("=== Betacom \(appVersion) khởi động ===")

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem.button {
            if let icon = NSImage(systemSymbolName: "bubble.left.and.bubble.right.fill", accessibilityDescription: "Betacom") {
                icon.isTemplate = true
                button.image = icon
            } else {
                button.title = "Betacom"
            }
            button.toolTip = "Betacom – nhiều tài khoản Zalo"
        }
        let menu = NSMenu()
        menu.delegate = self
        menu.autoenablesItems = false
        statusItem.menu = menu

        LoginItem.refreshIfEnabled()
        launcher.restore()
        reopenPrevious()

        if store.readOnly {
            DispatchQueue.main.async {
                Prompt.info(
                    "Không đọc được cấu hình",
                    "Betacom không đọc hoặc không sao lưu được \(self.store.configURL.path). Thay đổi trong lần chạy này sẽ không được lưu để tránh ghi đè cấu hình cũ."
                )
            }
        }

        if store.accounts.isEmpty {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { self.welcome() }
        }
    }

    private func reopenPrevious() {
        guard launcher.zaloApp() != nil else { return }
        let ids = store.accounts.filter { $0.reopen && !launcher.isRunning($0.id) }.map(\.id)
        for (index, id) in ids.enumerated() {
            DispatchQueue.main.asyncAfter(deadline: .now() + 1.5 * Double(index)) { [weak self] in
                guard let self = self, let account = self.store.account(id), account.reopen,
                      !self.launcher.isRunning(id) else { return }
                do {
                    try self.launcher.start(account)
                } catch {
                    writeLog("Không mở lại được \(id): \(error)")
                }
            }
        }
    }

    private func welcome() {
        let message = """
        Betacom đã nằm trên thanh menu, ở góc trên bên phải màn hình (biểu tượng hai bong bóng chat).

        Bấm vào biểu tượng đó → "Thêm tài khoản…" để mở một Zalo mới, rồi quét mã QR bằng điện thoại.
        """
        Prompt.info("Chào mừng bạn đến với Betacom", message)
        if launcher.zaloApp() == nil { askForZalo() }
    }

    func menuNeedsUpdate(_ menu: NSMenu) {
        menu.removeAllItems()

        let running = launcher.runningCount
        let header = running == 0 ? "Betacom" : "Betacom · \(running) tài khoản đang chạy"
        menu.addItem(disabled(header))
        menu.addItem(.separator())

        if launcher.zaloApp() == nil {
            menu.addItem(item("Chưa tìm thấy Zalo – Tải Zalo cho Mac…", #selector(openLink(_:)), link: Links.zaloDownload))
            menu.addItem(item("Chọn vị trí Zalo.app…", #selector(chooseZalo)))
            menu.addItem(.separator())
        }

        for (index, account) in store.accounts.enumerated() {
            let isOn = launcher.isRunning(account.id)
            let key = index < 9 ? String(index + 1) : ""
            let mi = item(account.name, #selector(openAccount(_:)), key: key)
            mi.representedObject = account.id
            mi.image = Avatar.image(for: account, running: isOn)
            mi.toolTip = isOn ? "Đang chạy – bấm để chuyển tới" : "Đang tắt – bấm để mở"
            menu.addItem(mi)
        }
        if !store.accounts.isEmpty { menu.addItem(.separator()) }

        menu.addItem(item("Thêm tài khoản…", #selector(addAccount), key: "n"))
        if !store.accounts.isEmpty {
            let manage = NSMenuItem(title: "Quản lý tài khoản", action: nil, keyEquivalent: "")
            manage.submenu = manageMenu()
            menu.addItem(manage)
            let stopAll = item("Tắt tất cả Zalo", #selector(stopAll))
            stopAll.isEnabled = running > 0
            menu.addItem(stopAll)
        }

        menu.addItem(.separator())
        let login = item("Mở Betacom cùng máy Mac", #selector(toggleLogin))
        login.state = LoginItem.isEnabled ? .on : .off
        menu.addItem(login)
        menu.addItem(item("Chọn vị trí Zalo.app…", #selector(chooseZalo)))
        menu.addItem(item("Mở thư mục dữ liệu", #selector(openDataFolder)))

        menu.addItem(.separator())
        menu.addItem(item("Kiểm tra phiên bản mới…", #selector(openLink(_:)), link: Links.update))
        menu.addItem(item("Phát triển bởi Betacom", #selector(openLink(_:)), link: Links.developer))
        menu.addItem(item("Giới thiệu Betacom", #selector(about)))

        menu.addItem(.separator())
        menu.addItem(item("Thoát Betacom", #selector(quit), key: "q"))
    }

    private func manageMenu() -> NSMenu {
        let menu = NSMenu()
        menu.autoenablesItems = false
        for account in store.accounts {
            let isOn = launcher.isRunning(account.id)
            let sub = NSMenu()
            sub.autoenablesItems = false

            let toggle = item(isOn ? "Tắt" : "Mở", isOn ? #selector(stopAccount(_:)) : #selector(openAccount(_:)))
            toggle.representedObject = account.id
            sub.addItem(toggle)
            sub.addItem(.separator())
            for (title, action) in [
                ("Đổi tên…", #selector(renameAccount(_:))),
                ("Đổi ảnh đại diện…", #selector(changeAvatar(_:))),
            ] {
                let mi = item(title, action)
                mi.representedObject = account.id
                sub.addItem(mi)
            }
            let reset = item("Dùng lại chữ viết tắt", #selector(resetAvatar(_:)))
            reset.representedObject = account.id
            reset.isEnabled = Avatar.hasCustom(account.id)
            sub.addItem(reset)
            sub.addItem(.separator())
            let remove = item("Xóa tài khoản…", #selector(removeAccount(_:)))
            remove.representedObject = account.id
            sub.addItem(remove)

            let mi = NSMenuItem(title: account.name, action: nil, keyEquivalent: "")
            mi.image = Avatar.image(for: account, running: isOn, size: 18)
            mi.submenu = sub
            menu.addItem(mi)
        }
        return menu
    }

    private func item(_ title: String, _ action: Selector, key: String = "", link: URL? = nil) -> NSMenuItem {
        let mi = NSMenuItem(title: title, action: action, keyEquivalent: key)
        mi.target = self
        if let link = link { mi.representedObject = link }
        return mi
    }

    private func disabled(_ title: String) -> NSMenuItem {
        let mi = NSMenuItem(title: title, action: nil, keyEquivalent: "")
        mi.isEnabled = false
        return mi
    }

    @objc private func openAccount(_ sender: NSMenuItem) {
        guard let id = sender.representedObject as? String, let account = store.account(id) else { return }
        if launcher.isRunning(id) {
            launcher.show(id)
            return
        }
        start(account)
    }

    private func start(_ account: Account) {
        guard let app = launcher.zaloApp() else {
            askForZalo()
            return
        }
        if launcher.isAppStoreBuild(app) {
            writeLog("Cảnh báo: đang dùng Zalo bản App Store")
        }
        do {
            try launcher.start(account)
        } catch {
            writeLog("Không mở được Zalo: \(error)")
            Prompt.info("Không mở được Zalo", error.localizedDescription)
        }
    }

    @objc private func stopAccount(_ sender: NSMenuItem) {
        guard let id = sender.representedObject as? String else { return }
        launcher.stop(id)
    }

    @objc private func stopAll() {
        launcher.stopAll(rememberForNextLaunch: false)
    }

    @objc private func addAccount() {
        let suggestion = "Zalo \(store.accounts.count + 1)"
        guard let name = Prompt.text(
            title: "Thêm tài khoản Zalo",
            message: "Đặt tên để dễ phân biệt. Sau đó một cửa sổ Zalo mới sẽ mở ra, bạn quét mã QR bằng Zalo trên điện thoại.",
            value: suggestion,
            ok: "Thêm"
        ) else { return }
        let account = store.addAccount(name: name)
        writeLog("Thêm tài khoản \(account.id)")
        start(account)
    }

    @objc private func renameAccount(_ sender: NSMenuItem) {
        guard let id = sender.representedObject as? String, var account = store.account(id) else { return }
        guard let name = Prompt.text(title: "Đổi tên tài khoản", message: "", value: account.name, ok: "Lưu") else { return }
        account.name = name
        store.update(account)
    }

    @objc private func changeAvatar(_ sender: NSMenuItem) {
        guard let id = sender.representedObject as? String else { return }
        let pictures = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Pictures")
        guard let url = Prompt.chooseFile(title: "Chọn ảnh đại diện", types: [.image], directory: pictures) else { return }
        do {
            try Avatar.importImage(from: url, for: id)
        } catch {
            Prompt.info("Không đổi được ảnh", error.localizedDescription)
        }
    }

    @objc private func resetAvatar(_ sender: NSMenuItem) {
        guard let id = sender.representedObject as? String else { return }
        Avatar.remove(id)
    }

    @objc private func removeAccount(_ sender: NSMenuItem) {
        guard let id = sender.representedObject as? String, let account = store.account(id) else { return }
        let ok = Prompt.confirm(
            title: "Xóa tài khoản \"\(account.name)\"?",
            message: "Dữ liệu đăng nhập và tin nhắn lưu trên máy của tài khoản này sẽ được chuyển vào Thùng rác. Tài khoản Zalo của bạn không bị ảnh hưởng, bạn có thể thêm lại và quét mã QR bất cứ lúc nào.",
            ok: "Xóa",
            destructive: true
        )
        guard ok else { return }
        store.removeAccount(id)
        writeLog("Đã xóa tài khoản \(id)")
        launcher.stop(id) { [weak self] exited in
            guard let self = self else { return }
            Avatar.remove(id)
            let path = self.store.profileURL(id).path
            if !exited {
                Prompt.info(
                    "Chưa dọn được dữ liệu",
                    "Zalo của tài khoản này chưa thoát nên dữ liệu vẫn nằm ở \(path). Hãy thoát Zalo đó rồi xóa thư mục này nếu không cần nữa."
                )
            } else if !self.store.trashProfile(id) {
                Prompt.info("Chưa dọn được dữ liệu", "Không chuyển được \(path) vào Thùng rác. Hãy xóa thư mục này bằng tay nếu không cần nữa.")
            }
        }
    }

    @objc private func toggleLogin() {
        let enable = !LoginItem.isEnabled
        guard !LoginItem.setEnabled(enable) else { return }
        let logPath = store.root.appendingPathComponent("log.txt").path
        if !enable {
            Prompt.info("Chưa tắt được", "Không xóa được LaunchAgent của Betacom. Chi tiết trong \(logPath).")
        } else if LoginItem.isTranslocated {
            Prompt.info(
                "Chưa bật được",
                "Betacom đang chạy từ vị trí tạm của macOS (App Translocation). Hãy chuyển Betacom.app vào thư mục Applications, mở lại rồi bật lại mục này."
            )
        } else {
            Prompt.info("Chưa bật được", "Không ghi được LaunchAgent của Betacom. Chi tiết trong \(logPath).")
        }
    }

    @objc private func chooseZalo() {
        guard let url = Prompt.chooseFile(
            title: "Chọn Zalo.app",
            types: [.application],
            directory: URL(fileURLWithPath: "/Applications")
        ) else { return }
        guard Bundle(url: url)?.executableURL != nil else {
            Prompt.info("Không phải ứng dụng hợp lệ", "Hãy chọn Zalo.app trong thư mục Applications.")
            return
        }
        store.zaloPath = url.path
        writeLog("Người dùng chọn Zalo: \(url.path)")
    }

    private func askForZalo() {
        NSApp.activate(ignoringOtherApps: true)
        let alert = NSAlert()
        alert.messageText = "Chưa tìm thấy Zalo trên máy"
        alert.informativeText = "Betacom dùng chính ứng dụng Zalo cho Mac. Hãy cài Zalo (bản tải từ zalo.me), hoặc chỉ cho Betacom biết vị trí Zalo.app."
        alert.addButton(withTitle: "Tải Zalo cho Mac")
        alert.addButton(withTitle: "Chọn Zalo.app…")
        alert.addButton(withTitle: "Để sau")
        switch alert.runModal() {
        case .alertFirstButtonReturn: NSWorkspace.shared.open(Links.zaloDownload)
        case .alertSecondButtonReturn: chooseZalo()
        default: break
        }
    }

    @objc private func openDataFolder() {
        NSWorkspace.shared.open(store.root)
    }

    @objc private func openLink(_ sender: NSMenuItem) {
        if let url = sender.representedObject as? URL {
            NSWorkspace.shared.open(url)
        }
    }

    @objc private func about() {
        var zalo = launcher.zaloApp()?.path ?? "(chưa tìm thấy)"
        if let app = launcher.zaloApp(), launcher.isAppStoreBuild(app) {
            zalo += "\n(Bản App Store — nếu các tài khoản bị lẫn nhau, hãy dùng bản tải từ zalo.me)"
        }
        let message = """
        Chạy nhiều tài khoản Zalo trên cùng một máy Mac.

        Phát triển bởi Betacom — https://github.com/betacomagency2026/BetaZalo
        Giấy phép MIT — miễn phí, mã nguồn mở, không quảng cáo.

        Quyền riêng tư:
        • Betacom không kết nối Internet và không thu thập dữ liệu gì.
        • Mỗi tài khoản Zalo có một thư mục dữ liệu riêng trên máy bạn.
        • Betacom không sửa và không chèn mã vào Zalo.

        Betacom không phải sản phẩm chính thức của Zalo / VNG.

        Zalo: \(zalo)
        Dữ liệu: \(store.root.path)
        """
        Prompt.info("Betacom \(appVersion)", message)
    }

    @objc private func quit() {
        if launcher.runningCount > 0 {
            NSApp.activate(ignoringOtherApps: true)
            let alert = NSAlert()
            alert.messageText = "Thoát Betacom"
            alert.informativeText = "Đang có \(launcher.runningCount) tài khoản Zalo chạy. Bạn muốn tắt luôn các Zalo này không?"
            alert.addButton(withTitle: "Tắt Zalo và thoát")
            alert.addButton(withTitle: "Chỉ thoát Betacom")
            alert.addButton(withTitle: "Hủy")
            switch alert.runModal() {
            case .alertFirstButtonReturn:
                stopOnQuit = true
            case .alertSecondButtonReturn:
                break
            default:
                return
            }
        }
        NSApp.terminate(nil)
    }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        guard stopOnQuit, launcher.runningCount > 0 else { return .terminateNow }
        stopOnQuit = false
        var replied = false
        let reply = {
            guard !replied else { return }
            replied = true
            NSApp.reply(toApplicationShouldTerminate: true)
        }
        try? String(ProcessInfo.processInfo.processIdentifier).write(to: AppDelegate.quittingMarker, atomically: true, encoding: .utf8)
        launcher.stopAll(rememberForNextLaunch: true, completion: reply)
        DispatchQueue.main.asyncAfter(deadline: .now() + ZaloLauncher.giveUpAfter + 1, execute: reply)
        return .terminateLater
    }

    func applicationWillTerminate(_ notification: Notification) {
        try? FileManager.default.removeItem(at: AppDelegate.quittingMarker)
    }

    static var quittingMarker: URL {
        Store.rootURL.appendingPathComponent("quitting.pid")
    }
}
