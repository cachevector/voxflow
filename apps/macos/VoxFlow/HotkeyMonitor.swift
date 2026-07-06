import AppKit
import ApplicationServices
import VoxFlowCore

final class HotkeyMonitor {
    private weak var dictation: DictationController?
    private var eventTap: CFMachPort?
    private var tapThread: Thread?
    private var globalMonitor: Any?
    private var localMonitor: Any?
    private var targetKeyCode: UInt16 = HotkeyBinding.defaultKeyCode
    private var modifierMask: CGEventFlags = .maskControl
    private var isModifierHeld = false
    private(set) var isActive = false

    init(dictation: DictationController) {
        self.dictation = dictation
    }

    func reloadBinding() {
        guard let settings = dictation?.engineHolder?.core.getSettings() else { return }
        targetKeyCode = UInt16(settings.hotkeyKeyCode)
        modifierMask = HotkeyBinding.modifierFlag(forKeyCode: targetKeyCode)
        VoxFlowLog.info("hotkey bound to \(settings.hotkeyLabel) (keyCode \(targetKeyCode))")
    }

    @discardableResult
    func start() -> Bool {
        reloadBinding()
        stop()

        // A listen-only tap is created even WITHOUT Input Monitoring, but silently
        // receives zero key events. So we must gate on the real permission first.
        guard CGPreflightListenEventAccess() else {
            VoxFlowLog.error("hotkey: Input Monitoring not granted — requesting access")
            // Adds this exact binary to System Settings → Input Monitoring and prompts.
            CGRequestListenEventAccess()
            isActive = false
            return false
        }

        if startEventTapThread() {
            isActive = true
            VoxFlowLog.info("hotkey: CGEvent tap active — hold \(HotkeyBinding.label(forKeyCode: targetKeyCode)) to dictate")
            return true
        }

        if startNSEventMonitors() {
            isActive = true
            VoxFlowLog.info("hotkey: NSEvent monitor active")
            return true
        }

        VoxFlowLog.error("hotkey: could not start — use menu bar → Start Dictation")
        isActive = false
        return false
    }

    func stop() {
        isActive = false
        if let tap = eventTap {
            CGEvent.tapEnable(tap: tap, enable: false)
            eventTap = nil
        }
        tapThread?.cancel()
        tapThread = nil
        if let globalMonitor {
            NSEvent.removeMonitor(globalMonitor)
            self.globalMonitor = nil
        }
        if let localMonitor {
            NSEvent.removeMonitor(localMonitor)
            self.localMonitor = nil
        }
    }

    private func startEventTapThread() -> Bool {
        final class TapBox: @unchecked Sendable {
            var tap: CFMachPort?
            var ok = false
        }
        let box = TapBox()
        let sem = DispatchSemaphore(value: 0)

        let thread = Thread { [weak self] in
            guard let self else {
                sem.signal()
                return
            }

            let mask: CGEventMask =
                (1 << CGEventType.flagsChanged.rawValue)
                | (1 << CGEventType.keyDown.rawValue)
                | (1 << CGEventType.keyUp.rawValue)

            guard let tap = CGEvent.tapCreate(
                tap: .cgAnnotatedSessionEventTap,
                place: .headInsertEventTap,
                options: .listenOnly,
                eventsOfInterest: mask,
                callback: { _, type, event, refcon in
                    guard let refcon else { return Unmanaged.passUnretained(event) }
                    let monitor = Unmanaged<HotkeyMonitor>.fromOpaque(refcon).takeUnretainedValue()
                    return monitor.handle(event: event, type: type)
                },
                userInfo: Unmanaged.passUnretained(self).toOpaque()
            ) else {
                sem.signal()
                return
            }

            box.tap = tap
            box.ok = true
            sem.signal()
            let source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source, .commonModes)
            CGEvent.tapEnable(tap: tap, enable: true)
            CFRunLoopRun()
        }
        thread.name = "VoxFlow-HotkeyTap"
        thread.start()

        _ = sem.wait(timeout: .now() + 2.0)

        guard box.ok, let tap = box.tap else {
            thread.cancel()
            return false
        }
        eventTap = tap
        tapThread = thread
        return true
    }

    private func startNSEventMonitors() -> Bool {
        let mask: NSEvent.EventTypeMask = [.flagsChanged, .keyDown, .keyUp]
        globalMonitor = NSEvent.addGlobalMonitorForEvents(matching: mask) { [weak self] in
            self?.handle(nsEvent: $0)
        }
        localMonitor = NSEvent.addLocalMonitorForEvents(matching: mask) { [weak self] event in
            self?.handle(nsEvent: event)
            return event
        }
        return globalMonitor != nil
    }

    private func handle(nsEvent event: NSEvent) {
        let keyCode = UInt16(event.keyCode)
        switch event.type {
        case .flagsChanged:
            let pressed = event.modifierFlags.intersection(.deviceIndependentFlagsMask).contains(
                NSEvent.ModifierFlags(rawValue: UInt(modifierMask.rawValue))
            )
            handlePress(pressed, keyCode: keyCode)
        case .keyDown where keyCode == targetKeyCode:
            handlePress(true, keyCode: keyCode)
        case .keyUp where keyCode == targetKeyCode:
            handlePress(false, keyCode: keyCode)
        default:
            break
        }
    }

    private func handle(event: CGEvent, type: CGEventType) -> Unmanaged<CGEvent>? {
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
            if let tap = eventTap {
                CGEvent.tapEnable(tap: tap, enable: true)
                VoxFlowLog.info("hotkey: re-enabled event tap")
            }
            return Unmanaged.passUnretained(event)
        }

        let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
        switch type {
        case .flagsChanged:
            if HotkeyBinding.isModifierKey(targetKeyCode), keyCode != 0, keyCode != targetKeyCode {
                return Unmanaged.passUnretained(event)
            }
            handlePress(event.flags.contains(modifierMask), keyCode: keyCode)
        case .keyDown where keyCode == targetKeyCode:
            handlePress(true, keyCode: keyCode)
        case .keyUp where keyCode == targetKeyCode:
            handlePress(false, keyCode: keyCode)
        default:
            break
        }
        return Unmanaged.passUnretained(event)
    }

    private func handlePress(_ pressed: Bool, keyCode: UInt16) {
        if pressed && !isModifierHeld {
            isModifierHeld = true
            VoxFlowLog.info("hotkey down (key \(keyCode))")
            DispatchQueue.main.async { [weak self] in self?.dictation?.beginRecording() }
        } else if !pressed && isModifierHeld {
            isModifierHeld = false
            VoxFlowLog.info("hotkey up")
            DispatchQueue.main.async { [weak self] in self?.dictation?.endRecording() }
        }
    }
}
