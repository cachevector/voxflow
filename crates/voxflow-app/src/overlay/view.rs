use gpui::{div, prelude::*, rgba, white, Context, Render, Window};
use voxflow_core::{StateEvent, UiState};

pub struct OverlayView {
    pub event: StateEvent,
}

impl OverlayView {
    fn label(&self) -> &'static str {
        // Calm, non-technical labels only — the spec explicitly rules out
        // anything implying network/API mechanics ("Uploading", "Waiting
        // for API", "Processing request").
        match self.event.ui_state {
            UiState::Idle => "",
            UiState::Listening => "Listening",
            UiState::Cleaning => "Cleaning",
            UiState::Inserting => "Inserting",
            UiState::Copied => "Copied — press Ctrl+V",
            UiState::Done => "Done",
            UiState::Error => "Something went wrong",
        }
    }
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
            .child(self.label())
    }
}
