//! macOS-only, short-lived accessibility observation after a VoxFlow paste.
//!
//! This deliberately does not install a general key logger. A session starts
//! only after VoxFlow inserts text, stays in the original foreground app for at
//! most fifteen seconds, and retains only the just-inserted suffix in memory.

#[cfg(target_os = "macos")]
mod platform {
    use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
    use core_foundation::string::{CFString, CFStringRef};
    use objc2_app_kit::NSWorkspace;
    use std::ffi::c_void;
    use std::ptr;
    use std::thread;
    use std::time::{Duration, Instant};
    use tauri::{AppHandle, Manager};

    type AXUIElementRef = *mut c_void;
    type AXError = i32;
    const K_AX_ERROR_SUCCESS: AXError = 0;

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXUIElementCreateSystemWide() -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> AXError;
        fn CGPreflightListenEventAccess() -> bool;
        fn AXIsProcessTrusted() -> bool;
    }

    #[derive(Clone)]
    struct Session {
        bundle_id: String,
        inserted_suffix: String,
        insertion_start: usize,
    }

    pub fn begin(app: AppHandle, inserted_text: String, blocklist: Vec<String>) {
        if inserted_text.trim().is_empty()
            || !unsafe { CGPreflightListenEventAccess() || !AXIsProcessTrusted() }
        {
            return;
        }
        let Some(bundle_id) = frontmost_bundle_id() else {
            return;
        };
        if blocklist.iter().any(|id| id == &bundle_id) {
            return;
        }
        let Some(value) = focused_field_value() else {
            return;
        };

        // Clipboard paste leaves the insertion at the end of the focused field
        // in the supported editors. If that invariant cannot be proven, do not
        // observe the field further.
        let inserted_suffix = inserted_text.trim_end().to_string();
        let current = value.trim_end();
        if !current.ends_with(&inserted_suffix) {
            return;
        }
        let insertion_start = current.len().saturating_sub(inserted_suffix.len());
        if !current.is_char_boundary(insertion_start) {
            return;
        }

        let session = Session {
            bundle_id,
            inserted_suffix,
            insertion_start,
        };
        let _ = thread::Builder::new()
            .name("voxflow-edit-learning".into())
            .spawn(move || poll_session(app, session));
    }

    fn poll_session(app: AppHandle, session: Session) {
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(250));
            if frontmost_bundle_id().as_deref() != Some(session.bundle_id.as_str()) {
                return;
            }
            let Some(value) = focused_field_value() else {
                return;
            };
            let current = value.trim_end();
            if current.len() < session.insertion_start
                || !current.is_char_boundary(session.insertion_start)
            {
                return;
            }
            let candidate = &current[session.insertion_start..];
            if candidate == session.inserted_suffix {
                continue;
            }

            // We support the safe, common case: a correction made before the
            // user continues typing after the VoxFlow insertion. Additional
            // text makes the span ambiguous, so it silently falls back.
            let Some(suggestion) = app
                .state::<crate::state::AppState>()
                .engine
                .vocabulary_suggestion_for_edit(session.inserted_suffix.clone(), candidate.into())
            else {
                return;
            };
            crate::windows::show_overlay(&app);
            crate::edit_learning_shortcuts::show(&app, suggestion);
            return;
        }
    }

    fn frontmost_bundle_id() -> Option<String> {
        let workspace = NSWorkspace::sharedWorkspace();
        workspace
            .frontmostApplication()?
            .bundleIdentifier()
            .map(|identifier| identifier.to_string())
    }

    fn focused_field_value() -> Option<String> {
        unsafe {
            let system = AXUIElementCreateSystemWide();
            if system.is_null() {
                return None;
            }
            let mut focused: CFTypeRef = ptr::null();
            // These are CFString constants in Apple's headers, but on newer
            // SDKs they are not exported linker symbols. Their documented AX
            // attribute names are stable and avoid an unnecessary symbol link.
            let focused_attribute = CFString::new("AXFocusedUIElement");
            let focused_result = AXUIElementCopyAttributeValue(
                system,
                focused_attribute.as_concrete_TypeRef(),
                &mut focused,
            );
            CFRelease(system as CFTypeRef);
            if focused_result != K_AX_ERROR_SUCCESS || focused.is_null() {
                return None;
            }

            let mut value: CFTypeRef = ptr::null();
            let value_attribute = CFString::new("AXValue");
            let value_result = AXUIElementCopyAttributeValue(
                focused as AXUIElementRef,
                value_attribute.as_concrete_TypeRef(),
                &mut value,
            );
            CFRelease(focused);
            if value_result != K_AX_ERROR_SUCCESS || value.is_null() {
                return None;
            }

            // `AXValue` is documented as a CFString for editable text fields.
            // Take ownership only for this conversion; no field value is kept.
            let value = CFString::wrap_under_create_rule(value as CFStringRef).to_string();
            Some(value)
        }
    }
}

#[cfg(target_os = "macos")]
pub use platform::begin;

#[cfg(not(target_os = "macos"))]
pub fn begin(_app: tauri::AppHandle, _inserted_text: String, _blocklist: Vec<String>) {}
