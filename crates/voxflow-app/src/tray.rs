/// System tray icon (StatusNotifierItem via ksni, which manages its own
/// D-Bus thread internally — no manual thread/channel bridging needed).
/// Phase 1 keeps this minimal: Quit and a placeholder Open Settings entry
/// (the real settings UI is Phase 2 scope).
struct VoxFlowTray;

impl ksni::Tray for VoxFlowTray {
    fn id(&self) -> String {
        "voxflow".into()
    }

    fn title(&self) -> String {
        "VoxFlow".into()
    }

    fn icon_name(&self) -> String {
        // Generic mic-ish stand-in until a real icon ships (Phase 2/5
        // packaging work); any freedesktop icon-theme name resolves fine.
        "audio-input-microphone".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Open Settings…".into(),
                activate: Box::new(|_| {
                    tracing::info!("Open Settings requested (not implemented until Phase 2)");
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn spawn() {
    ksni::TrayService::new(VoxFlowTray).spawn();
}
