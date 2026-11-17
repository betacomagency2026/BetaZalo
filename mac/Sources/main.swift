import AppKit

if let bundleID = Bundle.main.bundleIdentifier {
    let me = ProcessInfo.processInfo.processIdentifier
    if let other = NSRunningApplication.runningApplications(withBundleIdentifier: bundleID)
        .first(where: { $0.processIdentifier != me }) {
        other.activate(options: [])
        exit(0)
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()
