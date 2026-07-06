import AVFoundation
import Foundation

enum MicrophonePermission {
    static var state: PermissionSnapshot.MicrophoneState {
        switch AVCaptureDevice.authorizationStatus(for: .audio) {
        case .authorized:
            return .authorized
        case .notDetermined:
            return .notDetermined
        case .denied, .restricted:
            return .denied
        @unknown default:
            return .denied
        }
    }

    static var isGranted: Bool {
        state == .authorized
    }

    /// Shows the native macOS microphone prompt only when status is notDetermined.
    static func requestSystemPromptIfNeeded() {
        guard state == .notDetermined else { return }
        AVCaptureDevice.requestAccess(for: .audio) { granted in
            DispatchQueue.main.async {
                if granted {
                    VoxFlowLog.info("Microphone access granted")
                } else {
                    VoxFlowLog.error("Microphone access denied")
                }
            }
        }
    }
}
