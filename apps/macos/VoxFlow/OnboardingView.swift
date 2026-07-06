import SwiftUI
import VoxFlowCore

struct OnboardingView: View {
    let engineHolder: EngineHolder
    @State private var step = 0
    @State private var apiKey = ""
    @State private var inviteCode = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Welcome to VoxFlow")
                .font(.title2.bold())

            switch step {
            case 0:
                Text("VoxFlow is a fast, native voice input layer. Hold **Left Control** to dictate anywhere.")
                PermissionChecklist()
            case 1:
                Text("Bring your own OpenAI key (optional). Stay local-only if you prefer.")
                SecureField("OpenAI API Key", text: $apiKey)
            case 2:
                Text("Have a beta invite code?")
                TextField("Invite code (INV-…)", text: $inviteCode)
            default:
                Text("You're ready. Hold Left Control, speak, release — text appears.")
            }

            HStack {
                if step > 0 {
                    Button("Back") { step -= 1 }
                }
                Spacer()
                Button(step >= 3 ? "Finish" : "Continue") {
                    if step >= 3 {
                        finish()
                    } else {
                        step += 1
                    }
                }
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding(24)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private func finish() {
        var s = engineHolder.core.getSettings()
        s.openaiApiKey = apiKey.isEmpty ? nil : apiKey
        s.onboardingComplete = true
        if !inviteCode.isEmpty, engineHolder.core.betaInviteValid(code: inviteCode) {
            s.onboardingComplete = true
        }
        engineHolder.core.saveSettings(settings: s)
        NSApp.keyWindow?.close()
    }
}

struct PermissionChecklist: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Grant these permissions in System Settings:")
                .font(.subheadline)
                .foregroundStyle(.secondary)

            PermissionRow(
                title: "Microphone",
                detail: "Required for recording",
                systemImage: "mic",
                pane: .microphone
            )
            PermissionRow(
                title: "Accessibility",
                detail: "Paste text into other apps",
                systemImage: "hand.point.up.left",
                pane: .accessibility
            )
            PermissionRow(
                title: "Input Monitoring",
                detail: "Global Left Control hotkey (hold to dictate)",
                systemImage: "keyboard",
                pane: .inputMonitoring
            )
        }
    }
}

private struct PermissionRow: View {
    let title: String
    let detail: String
    let systemImage: String
    let pane: SystemSettingsLinks.PrivacyPane

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: systemImage)
                .frame(width: 20)
                .foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.body.weight(.medium))
                Text(detail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("Open") {
                SystemSettingsLinks.open(pane)
            }
            .controlSize(.small)
        }
    }
}
