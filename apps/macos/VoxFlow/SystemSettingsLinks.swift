import AppKit
import Foundation

/// Opens a specific Privacy & Security pane in macOS System Settings.
enum SystemSettingsLinks {
    enum PrivacyPane {
        case microphone
        case accessibility
        case inputMonitoring

        /// Apple anchor identifiers for Privacy panes.
        var anchor: String {
            switch self {
            case .microphone: return "Privacy_Microphone"
            case .accessibility: return "Privacy_Accessibility"
            case .inputMonitoring: return "Privacy_ListenEvent"
            }
        }

        var title: String {
            switch self {
            case .microphone: return "Microphone"
            case .accessibility: return "Accessibility"
            case .inputMonitoring: return "Input Monitoring"
            }
        }
    }

    @discardableResult
    static func open(_ pane: PrivacyPane) -> Bool {
        for url in candidateURLs(for: pane) {
            if NSWorkspace.shared.open(url) {
                return true
            }
        }
        return false
    }

    /// macOS 14+ uses the Privacy & Security extension; older versions use Security preference pane.
    private static func candidateURLs(for pane: PrivacyPane) -> [URL] {
        let anchor = pane.anchor
        let encoded = anchor.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? anchor

        var urls: [URL] = []

        if #available(macOS 14.0, *) {
            urls.append(contentsOf: [
                URL(string: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?\(encoded)"),
                URL(string: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?\(anchor)"),
            ].compactMap { $0 })
        }

        urls.append(contentsOf: [
            URL(string: "x-apple.systempreferences:com.apple.preference.security?\(encoded)"),
            URL(string: "x-apple.systempreferences:com.apple.preference.security?\(anchor)"),
        ].compactMap { $0 })

        return urls
    }
}
