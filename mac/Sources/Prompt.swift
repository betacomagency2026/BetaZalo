import AppKit
import UniformTypeIdentifiers

enum Prompt {
    static func text(title: String, message: String, value: String, ok: String) -> String? {
        NSApp.activate(ignoringOtherApps: true)
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.addButton(withTitle: ok)
        alert.addButton(withTitle: "Hủy")

        let field = NSTextField(frame: NSRect(x: 0, y: 0, width: 260, height: 24))
        field.stringValue = value
        field.placeholderString = "Ví dụ: Bán hàng 1"
        alert.accessoryView = field
        alert.window.initialFirstResponder = field

        guard alert.runModal() == .alertFirstButtonReturn else { return nil }
        let result = field.stringValue.trimmingCharacters(in: .whitespacesAndNewlines)
        return result.isEmpty ? nil : String(result.prefix(40))
    }

    static func confirm(title: String, message: String, ok: String, destructive: Bool = false) -> Bool {
        NSApp.activate(ignoringOtherApps: true)
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.alertStyle = destructive ? .warning : .informational
        let button = alert.addButton(withTitle: ok)
        if destructive { button.hasDestructiveAction = true }
        alert.addButton(withTitle: "Hủy")
        return alert.runModal() == .alertFirstButtonReturn
    }

    static func info(_ title: String, _ message: String) {
        NSApp.activate(ignoringOtherApps: true)
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.runModal()
    }

    static func chooseFile(title: String, types: [UTType], directory: URL? = nil) -> URL? {
        NSApp.activate(ignoringOtherApps: true)
        let panel = NSOpenPanel()
        panel.title = title
        panel.allowedContentTypes = types
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        if let directory = directory { panel.directoryURL = directory }
        return panel.runModal() == .OK ? panel.url : nil
    }
}
