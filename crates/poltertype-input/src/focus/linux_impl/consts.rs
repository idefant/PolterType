//! Tunables for the Linux focus tracker.

use std::time::Duration;

/// How long a focus answer may be served from cache. The wordlist-
/// profile watcher asks every 250 ms and the engine asks on every word
/// boundary; one real IPC/X11 round-trip per 150 ms window keeps all
/// of them cheap while staying far below human app-switching cadence.
pub(crate) const FOCUS_CACHE_TTL: Duration = Duration::from_millis(150);

/// `ATSPI_COORD_TYPE_WINDOW` — extents relative to the object's
/// toplevel window. Chosen over `SCREEN` (0) deliberately: a
/// native-Wayland toolkit cannot know its global position, so its
/// SCREEN answers are anchored at the window's *initial* placement
/// and go stale the moment the compositor re-tiles it (observed live
/// with kate on Hyprland). Window-relative extents stay correct; the
/// consumer composes them with the compositor's live window rect.
pub(crate) const COORD_TYPE_WINDOW: u32 = 1;

/// Exclusive top/overlay layers receive keyboard input without replacing
/// `activewindow`. Noninteractive notifications must not break a word.
pub(crate) const KEYBOARD_CONTEXT_QUERY: &str = r#"repl if type(hl.get_layers)~="function" or type(hl.get_active_window)~="function" then return "unavailable" end local ids={} for _,l in ipairs(hl.get_layers()) do if l.mapped and l.interactivity==1 and l.layer>=2 then table.insert(ids,l.address) end end table.sort(ids) if #ids>0 then return "layers:"..table.concat(ids,",") end local w=hl.get_active_window() return w and ("window:"..w.address) or "none""#;
