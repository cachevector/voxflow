import AppKit
import ApplicationServices

enum AccessibilityPermission {
    static var isGranted: Bool {
        AXIsProcessTrusted()
    }

    /// Registers this binary in Accessibility settings — call only when user asks to open Settings.
    static func registerInSystemSettings() {
        guard !isGranted else { return }
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: false] as CFDictionary
        _ = AXIsProcessTrustedWithOptions(options)
    }
}
