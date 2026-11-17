import AppKit

if let bundleID = Bundle.main.bundleIdentifier {
    let me = ProcessInfo.processInfo.processIdentifier
    if let other = NSRunningApplication.runningApplications(withBundleIdentifier: bundleID)
        .first(where: { $0.processIdentifier != me }) {
        let pid = other.processIdentifier
        let marker = (try? String(contentsOf: AppDelegate.quittingMarker, encoding: .utf8))
            .flatMap { pid_t($0.trimmingCharacters(in: .whitespacesAndNewlines)) }
        if marker == pid {
            let deadline = Date().addingTimeInterval(ZaloLauncher.giveUpAfter + 3)
            while kill(pid, 0) == 0 && Date() < deadline {
                usleep(200_000)
            }
        }
        if kill(pid, 0) == 0 {
            DistributedNotificationCenter.default().postNotificationName(
                AppDelegate.showMenuNotification, object: nil, userInfo: nil, deliverImmediately: true
            )
            exit(0)
        }
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()
