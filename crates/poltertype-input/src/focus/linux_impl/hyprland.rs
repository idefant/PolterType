//! Hyprland focus tracker.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::focus::{CaretHint, FocusTracker, FocusedWindowGeometry};

use super::atspi_caret::AtspiCaretWatcher;
use super::hyprland_ipc::{
    active_window_reply, keyboard_context_reply, parse_active_window, parse_active_window_id,
    parse_active_window_rect, parse_keyboard_context,
};
use super::proc_exe::exe_basename_for_pid;
use super::types::CaretSample;

/// Focus via Hyprland's `activewindow` IPC query. Prefers the window's
/// PID resolved through `/proc` (the exact analogue of the Windows
/// tracker's process-image basename); falls back to the window class
/// when `/proc` is unreadable (sandboxed apps).
pub(crate) struct HyprlandFocusTracker {
    /// Shared AT-SPI caret watcher; `None` when the a11y bus is
    /// unavailable (the tooltip then anchors to the window).
    caret: Option<Arc<AtspiCaretWatcher>>,
    context_query_supported: AtomicBool,
}

impl HyprlandFocusTracker {
    pub(crate) fn new(caret: Option<Arc<AtspiCaretWatcher>>) -> Self {
        Self {
            caret,
            context_query_supported: AtomicBool::new(false),
        }
    }
}

impl FocusTracker for HyprlandFocusTracker {
    fn focused_exe(&self) -> Option<String> {
        let reply = active_window_reply()?;
        let (pid, class) = parse_active_window(&reply);
        pid.and_then(exe_basename_for_pid).or(class)
    }

    fn focused_window_id(&self) -> Option<String> {
        if let Some(context) = keyboard_context_reply()
            .as_deref()
            .and_then(parse_keyboard_context)
        {
            self.context_query_supported.store(true, Ordering::Relaxed);
            return Some(context);
        }
        // After a successful layer-aware query, an outage is unknown focus,
        // not a move back to the window underneath the still-open menu.
        if self.context_query_supported.load(Ordering::Relaxed) {
            return None;
        }
        parse_active_window_id(&active_window_reply()?)
    }

    fn focused_window_geometry(&self) -> Option<FocusedWindowGeometry> {
        let reply = active_window_reply()?;
        let (x, y, width, height) = parse_active_window_rect(&reply)?;
        let (pid, _) = parse_active_window(&reply);
        Some(FocusedWindowGeometry {
            x,
            y,
            width,
            height,
            pid,
        })
    }

    fn caret_hint(&self) -> Option<CaretHint> {
        self.caret.as_ref()?.latest().map(CaretSample::into_hint)
    }

    fn backend_name(&self) -> &'static str {
        "linux-hyprland-ipc"
    }
}
