import AppKit

/// Ảnh đại diện trong menu: ảnh người dùng chọn, hoặc chữ viết tắt trên nền màu.
enum Avatar {
    /// (nền, chữ) — cùng bảng màu với MultiZ bản Windows.
    static let palette: [(NSColor, NSColor)] = [
        (hex(0xE1F5EE), hex(0x085041)),
        (hex(0xEEEDFE), hex(0x3C3489)),
        (hex(0xFAECE7), hex(0x712B13)),
        (hex(0xE6F1FB), hex(0x0C447C)),
        (hex(0xFBEAF0), hex(0x72243E)),
        (hex(0xFAEEDA), hex(0x633806)),
        (hex(0xEAF3DE), hex(0x27500A)),
        (hex(0xF1EFE8), hex(0x444441)),
    ]

    private static var cache: [String: NSImage] = [:]

    static func fileURL(_ id: String) -> URL {
        Store.shared.profileURL(id).appendingPathComponent("avatar.png")
    }

    static func hasCustom(_ id: String) -> Bool {
        FileManager.default.fileExists(atPath: fileURL(id).path)
    }

    private static func custom(_ id: String) -> NSImage? {
        if let img = cache[id] { return img }
        guard hasCustom(id), let img = NSImage(contentsOf: fileURL(id)) else { return nil }
        cache[id] = img
        return img
    }

    /// Ảnh tròn cho menu; `running` thêm chấm xanh ở góc.
    static func image(for account: Account, running: Bool, size: CGFloat = 20) -> NSImage {
        let photo = custom(account.id)
        let colors = palette[account.color % palette.count]
        let text = initials(account.name)

        return NSImage(size: NSSize(width: size, height: size), flipped: false) { rect in
            let circle = NSBezierPath(ovalIn: rect.insetBy(dx: 0.5, dy: 0.5))
            NSGraphicsContext.saveGraphicsState()
            circle.addClip()
            if let photo = photo {
                photo.draw(in: rect, from: .zero, operation: .sourceOver, fraction: running ? 1 : 0.55)
            } else {
                (running ? colors.0 : NSColor(white: 0.93, alpha: 1)).setFill()
                NSBezierPath(rect: rect).fill()
                let para = NSMutableParagraphStyle()
                para.alignment = .center
                let attrs: [NSAttributedString.Key: Any] = [
                    .font: NSFont.systemFont(ofSize: size * 0.4, weight: .semibold),
                    .foregroundColor: running ? colors.1 : NSColor.gray,
                    .paragraphStyle: para,
                ]
                let str = text as NSString
                let h = str.size(withAttributes: attrs).height
                str.draw(in: NSRect(x: 0, y: (size - h) / 2, width: size, height: h), withAttributes: attrs)
            }
            NSGraphicsContext.restoreGraphicsState()

            if running {
                let d = size * 0.38
                let dot = NSRect(x: size - d, y: 0, width: d, height: d)
                NSColor.white.setFill()
                NSBezierPath(ovalIn: dot).fill()
                hex(0x1D9E75).setFill()
                NSBezierPath(ovalIn: dot.insetBy(dx: d * 0.18, dy: d * 0.18)).fill()
            }
            return true
        }
    }

    static func initials(_ name: String) -> String {
        let words = name.split(separator: " ").map(String.init)
        if words.isEmpty { return "?" }
        if words.count == 1 { return String(words[0].prefix(2)).uppercased() }
        return (String(words.first!.prefix(1)) + String(words.last!.prefix(1))).uppercased()
    }

    /// Cắt vuông phần giữa ảnh, thu về 128x128 và lưu PNG trong thư mục tài khoản.
    static func importImage(from url: URL, for id: String) throws {
        guard let source = NSImage(contentsOf: url), source.size.width > 0, source.size.height > 0 else {
            throw AvatarError.unreadable
        }
        let px = 128
        guard let rep = NSBitmapImageRep(
            bitmapDataPlanes: nil, pixelsWide: px, pixelsHigh: px, bitsPerSample: 8, samplesPerPixel: 4,
            hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0
        ) else { throw AvatarError.unreadable }

        let side = min(source.size.width, source.size.height)
        let from = NSRect(x: (source.size.width - side) / 2, y: (source.size.height - side) / 2, width: side, height: side)

        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
        NSGraphicsContext.current?.imageInterpolation = .high
        source.draw(in: NSRect(x: 0, y: 0, width: px, height: px), from: from, operation: .copy, fraction: 1)
        NSGraphicsContext.restoreGraphicsState()

        guard let data = rep.representation(using: .png, properties: [:]) else { throw AvatarError.unreadable }
        try FileManager.default.createDirectory(at: fileURL(id).deletingLastPathComponent(), withIntermediateDirectories: true)
        try data.write(to: fileURL(id), options: .atomic)
        cache[id] = nil
    }

    static func remove(_ id: String) {
        try? FileManager.default.removeItem(at: fileURL(id))
        cache[id] = nil
    }

    enum AvatarError: LocalizedError {
        case unreadable
        var errorDescription: String? { "Không đọc được ảnh này. Hãy thử ảnh PNG hoặc JPG khác." }
    }
}

func hex(_ rgb: Int) -> NSColor {
    NSColor(
        srgbRed: CGFloat((rgb >> 16) & 0xFF) / 255,
        green: CGFloat((rgb >> 8) & 0xFF) / 255,
        blue: CGFloat(rgb & 0xFF) / 255,
        alpha: 1
    )
}
