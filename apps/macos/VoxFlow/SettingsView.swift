import SwiftUI
import VoxFlowCore

struct SettingsView: View {
    let engineHolder: EngineHolder
    @State private var apiKey: String = ""
    @State private var qualityMode: String = "Hybrid"
    @State private var clipboardRestore = true
    @State private var dashboard: FfiCostDashboard?

    var body: some View {
        TabView {
            GeneralSettingsTab(apiKey: $apiKey, qualityMode: $qualityMode, clipboardRestore: $clipboardRestore, onSave: save)
                .tabItem { Label("General", systemImage: "gearshape") }

            CostControlTab(dashboard: dashboard)
                .tabItem { Label("Cost Control", systemImage: "indianrupeesign.circle") }

            PrivacySettingsTab()
                .tabItem { Label("Privacy", systemImage: "hand.raised") }

            LicenseSettingsTab(engineHolder: engineHolder)
                .tabItem { Label("License", systemImage: "key") }
        }
        .frame(width: 520, height: 400)
        .onAppear(perform: load)
    }

    private func load() {
        let s = engineHolder.core.getSettings()
        apiKey = s.openaiApiKey ?? ""
        qualityMode = s.qualityMode
        clipboardRestore = s.clipboardRestore
        dashboard = engineHolder.core.costDashboard()
    }

    private func save() {
        var s = engineHolder.core.getSettings()
        s.openaiApiKey = apiKey.isEmpty ? nil : apiKey
        s.qualityMode = qualityMode
        s.clipboardRestore = clipboardRestore
        engineHolder.core.saveSettings(settings: s)
        dashboard = engineHolder.core.costDashboard()
    }
}

struct GeneralSettingsTab: View {
    @Binding var apiKey: String
    @Binding var qualityMode: String
    @Binding var clipboardRestore: Bool
    let onSave: () -> Void

    var body: some View {
        Form {
            Section("Hotkey") {
                Text("Hold **Left Control** to dictate (push-to-talk). Change `hotkey` in settings.json if needed.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Section("OpenAI BYOK") {
                SecureField("API Key", text: $apiKey)
                Text("Uses gpt-4o-mini-transcribe by default (~$0.003/min)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Section("Mode") {
                Picker("Quality", selection: $qualityMode) {
                    Text("Hybrid").tag("Hybrid")
                    Text("Economy").tag("Economy")
                    Text("Balanced").tag("Balanced")
                    Text("Accurate").tag("Accurate")
                    Text("Offline").tag("Offline")
                }
                Toggle("Restore clipboard after paste", isOn: $clipboardRestore)
            }
            Button("Save", action: onSave)
        }
        .padding()
    }
}

struct CostControlTab: View {
    let dashboard: FfiCostDashboard?

    var body: some View {
        Form {
            if let d = dashboard {
                LabeledContent("Minutes this month") {
                    Text(String(format: "%.1f", d.minutesTranscribed))
                }
                LabeledContent("Estimated cost") {
                    Text(String(format: "$%.2f (₹%.0f)", d.estimatedUsd, d.estimatedInr))
                }
                LabeledContent("Projected monthly") {
                    Text(String(format: "$%.2f (₹%.0f)", d.projectedMonthlyUsd, d.projectedMonthlyInr))
                }
                LabeledContent("Local vs cloud") {
                    Text(String(format: "%.0f%% local / %.0f%% cloud", d.localPercentage, d.cloudPercentage))
                }
                if !d.capWarnings.isEmpty {
                    Section("Warnings") {
                        ForEach(d.capWarnings, id: \.self) { w in
                            Text(w).foregroundStyle(.orange)
                        }
                    }
                }
            } else {
                Text("Loading usage…")
            }
        }
        .padding()
    }
}

struct PrivacySettingsTab: View {
    var body: some View {
        Form {
            Text("History is stored locally. Cloud is used only in hybrid/cloud modes.")
                .foregroundStyle(.secondary)
            Toggle("Never save audio (always on)", isOn: .constant(true))
                .disabled(true)

            Section("Permissions") {
                Button("Open Microphone settings") {
                    SystemSettingsLinks.open(.microphone)
                }
                Button("Open Accessibility settings") {
                    SystemSettingsLinks.open(.accessibility)
                }
                Button("Open Input Monitoring settings") {
                    SystemSettingsLinks.open(.inputMonitoring)
                }
            }
        }
        .padding()
    }
}

struct LicenseSettingsTab: View {
    let engineHolder: EngineHolder
    @State private var licenseKey = ""
    @State private var message = ""

    var body: some View {
        Form {
            TextField("License key (VFPRO-…)", text: $licenseKey)
            Button("Activate") {
                message = engineHolder.core.validateLicense(key: licenseKey)
            }
            if !message.isEmpty {
                Text(message).foregroundStyle(.secondary)
            }
        }
        .padding()
    }
}
