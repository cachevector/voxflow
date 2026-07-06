import CoreGraphics

/// Maps settings hotkey key codes to modifier flags for push-to-talk.
enum HotkeyBinding {
    /// Default: Left Control (key code 59), on every Mac and external keyboard.
    static let defaultKeyCode: UInt16 = 59
    static let defaultLabel = "Left Control"

    static func modifierFlag(forKeyCode keyCode: UInt16) -> CGEventFlags {
        switch keyCode {
        case 59, 62: // Left / Right Control
            return .maskControl
        case 58, 61: // Left / Right Option
            return .maskAlternate
        case 55, 54: // Left / Right Command
            return .maskCommand
        case 56, 60: // Left / Right Shift
            return .maskShift
        default:
            return .maskControl
        }
    }

    static func label(forKeyCode keyCode: UInt16) -> String {
        switch keyCode {
        case 59: return "Left Control"
        case 62: return "Right Control"
        case 58: return "Left Option"
        case 61: return "Right Option"
        case 55: return "Left Command"
        case 54: return "Right Command"
        case 56: return "Left Shift"
        case 60: return "Right Shift"
        default: return defaultLabel
        }
    }

    static func isModifierKey(_ keyCode: UInt16) -> Bool {
        switch keyCode {
        case 59, 62, 58, 61, 55, 54, 56, 60:
            return true
        default:
            return false
        }
    }
}
