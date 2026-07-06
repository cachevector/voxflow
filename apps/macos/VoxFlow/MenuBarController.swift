import AppKit

final class MenuBarController {
    private weak var appDelegate: AppDelegate?
    private let statusItem: NSStatusItem
    private let dictateItem = NSMenuItem(title: "Start Dictation", action: #selector(toggleDictation), keyEquivalent: "d")

    init(appDelegate: AppDelegate) {
        self.appDelegate = appDelegate
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        if let button = statusItem.button {
            if let image = NSImage(systemSymbolName: "waveform.circle.fill", accessibilityDescription: "VoxFlow") {
                image.isTemplate = true
                button.image = image
            }
            button.toolTip = "VoxFlow — Hold Left Control or use Start Dictation"
        }

        let menu = NSMenu()
        menu.addItem(NSMenuItem(title: "VoxFlow", action: nil, keyEquivalent: ""))
        menu.addItem(.separator())
        dictateItem.keyEquivalentModifierMask = [.command, .shift]
        menu.addItem(dictateItem)
        menu.addItem(.separator())
        menu.addItem(NSMenuItem(title: "Settings…", action: #selector(openSettings), keyEquivalent: ","))
        menu.addItem(NSMenuItem(title: "Reconnect Hotkey", action: #selector(reconnectHotkey), keyEquivalent: "r"))
        menu.addItem(NSMenuItem(title: "Show Log File", action: #selector(showLog), keyEquivalent: "l"))
        menu.addItem(.separator())
        menu.addItem(NSMenuItem(title: "Quit VoxFlow", action: #selector(quit), keyEquivalent: "q"))
        menu.items.forEach { $0.target = self }
        statusItem.menu = menu
    }

    @objc private func toggleDictation() {
        appDelegate?.dictation?.toggleRecording()
        updateDictateLabel()
    }

    private func updateDictateLabel() {
        let recording = appDelegate?.dictation?.isRecording ?? false
        dictateItem.title = recording ? "Stop & Transcribe" : "Start Dictation"
    }

    @objc private func openSettings() {
        NSApp.sendAction(Selector(("showSettingsWindow:")), to: nil, from: nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    @objc private func reconnectHotkey() {
        guard let appDelegate else { return }
        PermissionsCoordinator.recheck(appDelegate: appDelegate)
    }

    @objc private func showLog() {
        VoxFlowLog.revealLogFile()
    }

    @objc private func quit() {
        NSApp.terminate(nil)
    }
}
