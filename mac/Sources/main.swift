import AppKit

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
// Chỉ hiện trên thanh menu, không có biểu tượng trên Dock.
app.setActivationPolicy(.accessory)
app.run()
