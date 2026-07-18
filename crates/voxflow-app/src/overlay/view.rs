use gpui::{div, prelude::*, rgba, white, Context, Render, Window};

/// Phase 0 spike placeholder: just enough visual content (a rounded,
/// semi-opaque pill with a label) to confirm the overlay window itself
/// renders, positions, and stacks correctly. The real waveform/state-label
/// rendering lands in Phase 2.
pub struct OverlayView {
    pub label: String,
}

impl Render for OverlayView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(rgba(0x1a1a1aee))
            .text_color(white())
            .text_size(gpui::rems(1.0))
            .child(self.label.clone())
    }
}
