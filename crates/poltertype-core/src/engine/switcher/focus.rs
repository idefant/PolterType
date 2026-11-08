//! Invalidating caret-dependent state when input moves to another window.

use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::engine::buffer::WordBuffer;

use super::engine::SwitcherEngine;

impl SwitcherEngine {
    pub(super) fn refresh_window_context(&self, buffer: &mut WordBuffer) {
        let Some(current) = self.focus_tracker.focused_window_id() else {
            self.window_focus_unavailable.store(true, Ordering::Relaxed);
            return;
        };
        let unavailable = self.window_focus_unavailable.swap(false, Ordering::Relaxed);
        {
            let mut tracked = self.tracked_window.lock();
            if tracked.as_ref() == Some(&current) {
                return;
            }
            let first_sample = tracked.is_none();
            *tracked = Some(current);
            if first_sample {
                return;
            }
        }
        // `abandon` preserves the old word's remainder at the caret.
        // That remainder belongs to the previous window, not this one.
        let partial_word = unavailable && !buffer.keys().is_empty();
        buffer.reset();
        buffer.mark_context_unclean();
        if partial_word {
            // Recovery may arrive halfway through the new window's word.
            // Its already-typed prefix was just discarded, so refuse its tail.
            buffer.poison();
        }
        *self.last_word.write() = None;
        self.word_history.write().clear();
        *self.word_layout.write() = None;
        *self.word_layout_settles_at.write() = None;
        *self.last_force_switch.write() = None;
        *self.paste_guard_until.write() = Instant::now();
        self.dismiss_suggestions(None);
    }
}
