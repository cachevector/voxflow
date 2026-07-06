import AppKit
import ApplicationServices
import Foundation
import VoxFlowCore

final class EngineHolder {
    let core: VoxFlowEngine

    init() {
        core = VoxFlowEngine(apiKey: nil)
    }

    func activeAppBundleId() -> String? {
        NSWorkspace.shared.frontmostApplication?.bundleIdentifier
    }
}

final class MacTextInserter {
    /// Paste via Cmd+V; always copies to clipboard first so text is never lost.
    @discardableResult
    static func paste(text: String, restoreClipboard: Bool) -> Bool {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return false }

        let pasteboard = NSPasteboard.general
        let previous = pasteboard.string(forType: .string)

        pasteboard.clearContents()
        pasteboard.setString(text, forType: .string)

        // Try synthetic paste — works with or without Accessibility on some apps via clipboard focus.
        let source = CGEventSource(stateID: .combinedSessionState)
        let cmdDown = CGEvent(keyboardEventSource: source, virtualKey: 0x37, keyDown: true)
        cmdDown?.flags = .maskCommand
        let vDown = CGEvent(keyboardEventSource: source, virtualKey: 0x09, keyDown: true)
        vDown?.flags = .maskCommand
        let vUp = CGEvent(keyboardEventSource: source, virtualKey: 0x09, keyDown: false)
        vUp?.flags = .maskCommand
        let cmdUp = CGEvent(keyboardEventSource: source, virtualKey: 0x37, keyDown: false)

        cmdDown?.post(tap: .cgSessionEventTap)
        vDown?.post(tap: .cgSessionEventTap)
        vUp?.post(tap: .cgSessionEventTap)
        cmdUp?.post(tap: .cgSessionEventTap)

        if restoreClipboard, let previous {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                pasteboard.clearContents()
                pasteboard.setString(previous, forType: .string)
            }
        }

        // Text is on clipboard regardless — user can Cmd+V manually if synthetic paste blocked.
        return AXIsProcessTrusted()
    }

    func insert(text: String, restoreClipboard: Bool) -> Bool {
        Self.paste(text: text, restoreClipboard: restoreClipboard)
    }
}
