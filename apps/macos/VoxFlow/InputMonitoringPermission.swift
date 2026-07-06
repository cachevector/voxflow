import ApplicationServices

enum InputMonitoringPermission {
    static var isGranted: Bool {
        CGPreflightListenEventAccess()
    }

    /// Registers this binary in Input Monitoring — call only when user asks to open Settings.
    @discardableResult
    static func registerInSystemSettings() -> Bool {
        guard !isGranted else { return true }
        return CGRequestListenEventAccess()
    }
}
