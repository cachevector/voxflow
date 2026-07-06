import AppKit
import AVFoundation

enum PermissionKind: CaseIterable {
    case inputMonitoring
    case accessibility
    case microphone

    var label: String {
        switch self {
        case .inputMonitoring:
            return "Input Monitoring — for Left Control hotkey (optional if using menu bar dictation)"
        case .accessibility:
            return "Accessibility — for auto-paste into other apps"
        case .microphone:
            return "Microphone — required to record"
        }
    }

    var settingsPane: SystemSettingsLinks.PrivacyPane {
        switch self {
        case .inputMonitoring: return .inputMonitoring
        case .accessibility: return .accessibility
        case .microphone: return .microphone
        }
    }

    var requiredForBasicUse: Bool {
        self == .microphone
    }
}

struct PermissionSnapshot {
    enum MicrophoneState {
        case authorized, notDetermined, denied
    }

    let inputMonitoring: Bool
    let accessibility: Bool
    let microphone: MicrophoneState
    let hotkeyActive: Bool

    static func current(hotkeyActive: Bool = false) -> PermissionSnapshot {
        PermissionSnapshot(
            inputMonitoring: InputMonitoringPermission.isGranted,
            accessibility: AccessibilityPermission.isGranted,
            microphone: MicrophonePermission.state,
            hotkeyActive: hotkeyActive
        )
    }

    var canDictate: Bool {
        microphone == .authorized
    }

    var missingRequired: [PermissionKind] {
        var missing: [PermissionKind] = []
        if microphone == .denied { missing.append(.microphone) }
        return missing
    }

    var missingOptional: [PermissionKind] {
        var missing: [PermissionKind] = []
        if !hotkeyActive && !inputMonitoring { missing.append(.inputMonitoring) }
        if !accessibility { missing.append(.accessibility) }
        return missing
    }

    func log() {
        VoxFlowLog.info(
            "Permissions — mic=\(microphone) hotkeyActive=\(hotkeyActive) " +
            "inputMonitoring=\(inputMonitoring) accessibility=\(accessibility)"
        )
    }
}

enum PermissionsCoordinator {
    private static var didShowLaunchPrompt = false

    static func ensureOnLaunch(appDelegate: AppDelegate) {
        if PermissionSnapshot.current().microphone == .notDetermined {
            MicrophonePermission.requestSystemPromptIfNeeded()
        }

        let hotkeyOk = appDelegate.startHotkeyMonitorIfAllowed()
        var snapshot = PermissionSnapshot.current(hotkeyActive: hotkeyOk)
        snapshot.log()

        if !snapshot.canDictate {
            promptOnce(context: .launch, snapshot: snapshot)
            return
        }

        if snapshot.missingRequired.isEmpty && (hotkeyOk || snapshot.missingOptional.isEmpty) {
            VoxFlowLog.info("Ready — hold Left Control or menu bar → Start Dictation (⌘⇧D)")
            return
        }

        // Mic OK but hotkey/accessibility missing — log only, don't nag if hotkey works.
        if hotkeyOk {
            VoxFlowLog.info("Ready — hold Left Control to dictate")
            return
        }

        promptOnce(context: .launch, snapshot: snapshot)
    }

    static func recheck(appDelegate: AppDelegate) {
        let hotkeyOk = appDelegate.startHotkeyMonitorIfAllowed()
        let snapshot = PermissionSnapshot.current(hotkeyActive: hotkeyOk)
        snapshot.log()

        if hotkeyOk && snapshot.canDictate {
            VoxFlowLog.info("Hotkey reconnected — hold Left Control to dictate")
            return
        }
        promptOnce(context: .manualRetry, snapshot: snapshot)
    }

    private enum PromptContext { case launch, manualRetry }

    private static func promptOnce(context: PromptContext, snapshot: PermissionSnapshot) {
        var missing = snapshot.missingRequired + snapshot.missingOptional
        if missing.isEmpty { missing = [.microphone] }
        if !snapshot.canDictate {
            missing = snapshot.missingRequired
        } else if context == .launch {
            // Mic works, hotkey doesn't — one-time hint about menu bar fallback.
            missing = snapshot.missingOptional
            guard !missing.isEmpty else { return }
        }

        guard !missing.isEmpty else { return }
        if context == .launch {
            guard !didShowLaunchPrompt else { return }
            didShowLaunchPrompt = true
        }

        for kind in missing {
            VoxFlowLog.error("Missing: \(kind.label)")
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.35) {
            PermissionAlert.show(missing: missing, canUseMenuBar: snapshot.canDictate)
        }
    }
}

enum PermissionAlert {
    static func show(missing: [PermissionKind], canUseMenuBar: Bool) {
        guard !missing.isEmpty else { return }

        let lines = missing.map { "• \($0.label)" }.joined(separator: "\n")
        let fallback = canUseMenuBar
            ? "\n\nYou can still dictate now: menu bar → Start Dictation (⌘⇧D)."
            : ""

        let alert = NSAlert()
        alert.messageText = "VoxFlow Setup"
        alert.informativeText =
            "For full functionality, enable in System Settings → Privacy & Security:\n\n" +
            "\(lines)\(fallback)"
        alert.alertStyle = .informational
        alert.addButton(withTitle: "Open System Settings")
        alert.addButton(withTitle: "OK")
        if alert.runModal() == .alertFirstButtonReturn, let first = missing.first {
            register(first)
            SystemSettingsLinks.open(first.settingsPane)
        }
    }

    private static func register(_ kind: PermissionKind) {
        switch kind {
        case .inputMonitoring: InputMonitoringPermission.registerInSystemSettings()
        case .accessibility: AccessibilityPermission.registerInSystemSettings()
        case .microphone: break
        }
    }
}
