use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyMarks {
    pub hotkey_down: Option<u64>,
    pub bar_shown: Option<u64>,
    pub recording_start: Option<u64>,
    pub hotkey_up: Option<u64>,
    pub vad_done: Option<u64>,
    pub transcribe_start: Option<u64>,
    pub transcribe_done: Option<u64>,
    pub insert_start: Option<u64>,
    pub insert_done: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyReport {
    pub bar_show_ms: Option<u64>,
    pub recording_start_ms: Option<u64>,
    pub end_to_vad_ms: Option<u64>,
    pub transcribe_ms: Option<u64>,
    pub insert_ms: Option<u64>,
    pub total_ms: Option<u64>,
}

pub struct LatencyTracker {
    start: Instant,
    marks: LatencyMarks,
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            marks: LatencyMarks::default(),
        }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.marks = LatencyMarks::default();
    }

    fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn mark_hotkey_down(&mut self) {
        self.marks.hotkey_down = Some(self.elapsed_ms());
    }

    pub fn mark_bar_shown(&mut self) {
        self.marks.bar_shown = Some(self.elapsed_ms());
    }

    pub fn mark_recording_start(&mut self) {
        self.marks.recording_start = Some(self.elapsed_ms());
    }

    pub fn mark_hotkey_up(&mut self) {
        self.marks.hotkey_up = Some(self.elapsed_ms());
    }

    pub fn mark_vad_done(&mut self) {
        self.marks.vad_done = Some(self.elapsed_ms());
    }

    pub fn mark_transcribe_start(&mut self) {
        self.marks.transcribe_start = Some(self.elapsed_ms());
    }

    pub fn mark_transcribe_done(&mut self) {
        self.marks.transcribe_done = Some(self.elapsed_ms());
    }

    pub fn mark_insert_start(&mut self) {
        self.marks.insert_start = Some(self.elapsed_ms());
    }

    pub fn mark_insert_done(&mut self) {
        self.marks.insert_done = Some(self.elapsed_ms());
    }

    pub fn report(&self) -> LatencyReport {
        let m = &self.marks;
        let diff = |a: Option<u64>, b: Option<u64>| match (a, b) {
            (Some(a), Some(b)) => Some(b.saturating_sub(a)),
            _ => None,
        };
        LatencyReport {
            bar_show_ms: diff(m.hotkey_down, m.bar_shown),
            recording_start_ms: diff(m.hotkey_down, m.recording_start),
            end_to_vad_ms: diff(m.hotkey_up, m.vad_done),
            transcribe_ms: diff(m.transcribe_start, m.transcribe_done),
            insert_ms: diff(m.insert_start, m.insert_done),
            total_ms: diff(m.hotkey_down, m.insert_done),
        }
    }

    pub fn marks(&self) -> &LatencyMarks {
        &self.marks
    }
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}
