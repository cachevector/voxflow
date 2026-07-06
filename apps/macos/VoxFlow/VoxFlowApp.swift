import SwiftUI

@main
struct VoxFlowApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        Settings {
            SettingsView(engineHolder: appDelegate.engineHolder)
        }
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    let engineHolder = EngineHolder()
    private var menuBar: MenuBarController?
    private var floatingBar: FloatingBarPanel?
    private(set) var dictation: DictationController?
    private(set) var hotkeyMonitor: HotkeyMonitor?

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        VoxFlowLog.info("VoxFlow launched from \(Bundle.main.bundlePath)")

        floatingBar = FloatingBarPanel(engineHolder: engineHolder)
        dictation = DictationController(engineHolder: engineHolder, floatingBar: floatingBar!)
        hotkeyMonitor = HotkeyMonitor(dictation: dictation!)
        menuBar = MenuBarController(appDelegate: self)

        // Don't block launch on audio init — can stall the main thread.
        DispatchQueue.global(qos: .userInitiated).async { [engineHolder] in
            engineHolder.core.prewarm()
            VoxFlowLog.info("audio prewarm complete")
        }

        PermissionsCoordinator.ensureOnLaunch(appDelegate: self)

        if !engineHolder.core.getSettings().onboardingComplete {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                self.showOnboarding()
            }
        }
    }

    @discardableResult
    func startHotkeyMonitorIfAllowed() -> Bool {
        hotkeyMonitor?.start() ?? false
    }

    func showOnboarding() {
        let onboarding = NSHostingController(rootView: OnboardingView(engineHolder: engineHolder))
        let window = NSWindow(contentViewController: onboarding)
        window.title = "Welcome to VoxFlow"
        window.setContentSize(NSSize(width: 520, height: 420))
        window.center()
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }
}
