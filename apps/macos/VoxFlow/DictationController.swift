import AppKit
import VoxFlowCore

/// Shared push-to-talk dictation pipeline (hotkey or menu bar).
final class DictationController {
    private(set) weak var engineHolder: EngineHolder?
    private weak var floatingBar: FloatingBarPanel?
    private let workQueue = DispatchQueue(label: "com.maskedsyntax.voxflow.dictation", qos: .userInitiated)
    private(set) var isRecording = false

    init(engineHolder: EngineHolder, floatingBar: FloatingBarPanel) {
        self.engineHolder = engineHolder
        self.floatingBar = floatingBar
    }

    func beginRecording() {
        guard !isRecording, let holder = engineHolder else { return }
        isRecording = true
        VoxFlowLog.info("dictation begin")
        let state = holder.core.onHotkeyDown()
        floatingBar?.show(state: state)
    }

    func endRecording() {
        guard isRecording else { return }
        isRecording = false
        VoxFlowLog.info("dictation end — transcribing")

        workQueue.async { [weak self] in
            guard let self, let holder = self.engineHolder else { return }
            let appId = holder.activeAppBundleId()
            let result = holder.core.onHotkeyUp(activeAppId: appId)

            DispatchQueue.main.async {
                VoxFlowLog.info(
                    "dictation finished success=\(result.success) textLen=\(result.text.count) " +
                    "state=\(String(describing: result.uiState))"
                )
                self.deliver(text: result.text)
                self.floatingBar?.show(result: result)
                DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
                    self.floatingBar?.hide()
                }
            }
        }
    }

    func toggleRecording() {
        if isRecording { endRecording() } else { beginRecording() }
    }

    private func deliver(text: String) {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            VoxFlowLog.error("empty transcript — speak longer or check mic/API key")
            return
        }
        guard let holder = engineHolder else { return }
        let restore = holder.core.getSettings().clipboardRestore
        if MacTextInserter.paste(text: text, restoreClipboard: restore) {
            VoxFlowLog.info("text delivered")
        } else {
            VoxFlowLog.info("text copied to clipboard")
        }
    }
}
