import AppKit
import SwiftUI
import VoxFlowCore

final class FloatingBarPanel: NSPanel {
    private var hostingView: NSHostingView<FloatingBarView>?
    private var startTime: Date?

    init(engineHolder: EngineHolder) {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 320, height: 56),
            styleMask: [.nonactivatingPanel, .fullSizeContentView, .hudWindow],
            backing: .buffered,
            defer: false
        )
        level = .floating
        isOpaque = false
        backgroundColor = .clear
        hasShadow = true
        isMovableByWindowBackground = false
        collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary]
        titleVisibility = .hidden
        titlebarAppearsTransparent = true

        let view = FloatingBarView(state: .idle, elapsed: 0)
        hostingView = NSHostingView(rootView: view)
        contentView = hostingView
        orderOut(nil)
    }

    func show(state: FfiUiState) {
        startTime = Date()
        update(state: state, text: nil)
        positionBottomCenter()
        orderFrontRegardless()
    }

    func show(result: FfiDictationResult) {
        update(state: result.uiState, text: result.text.isEmpty ? nil : result.text)
    }

    func hide() {
        orderOut(nil)
    }

    private func update(state: FfiUiState, text: String?) {
        let elapsed = startTime.map { Date().timeIntervalSince($0) } ?? 0
        hostingView?.rootView = FloatingBarView(state: state, elapsed: elapsed, preview: text)
    }

    private func positionBottomCenter() {
        guard let screen = NSScreen.main else { return }
        let frame = screen.visibleFrame
        let x = frame.midX - frame.width * 0.0 - 160
        let y = frame.minY + 24
        setFrameOrigin(NSPoint(x: x, y: y))
    }
}

struct FloatingBarView: View {
    let state: FfiUiState
    let elapsed: TimeInterval
    var preview: String?

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: iconName)
                .foregroundStyle(.primary)

            VStack(alignment: .leading, spacing: 2) {
                Text(label)
                    .font(.system(size: 13, weight: .semibold))
                if let preview, !preview.isEmpty {
                    Text(preview)
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }

            Spacer()

            Text(String(format: "%.1fs", elapsed))
                .font(.system(size: 11, design: .monospaced))
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.ultraThinMaterial, in: Capsule())
        .overlay(Capsule().strokeBorder(.quaternary, lineWidth: 0.5))
        .frame(width: 320)
    }

    private var label: String {
        switch state {
        case .idle: return "Ready"
        case .listening: return "Listening"
        case .cleaning: return "Cleaning"
        case .inserting: return "Inserting"
        case .copied: return "Copied"
        case .done: return "Done"
        case .error: return "Error"
        }
    }

    private var iconName: String {
        switch state {
        case .listening: return "mic.fill"
        case .cleaning, .inserting: return "sparkles"
        case .copied: return "doc.on.clipboard"
        case .done: return "checkmark.circle.fill"
        case .error: return "exclamationmark.triangle.fill"
        default: return "waveform"
        }
    }
}