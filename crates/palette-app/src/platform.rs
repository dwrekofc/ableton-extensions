#[cfg(target_os = "macos")]
pub fn ableton_is_frontmost() -> bool {
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};
    unsafe {
        let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        let application: *mut Object = msg_send![workspace, frontmostApplication];
        if application.is_null() {
            return false;
        }

        let bundle_id: *mut Object = msg_send![application, bundleIdentifier];
        let name: *mut Object = msg_send![application, localizedName];
        is_ableton_identity(ns_string(bundle_id).as_deref(), ns_string(name).as_deref())
    }
}

fn is_ableton_identity(bundle_id: Option<&str>, name: Option<&str>) -> bool {
    bundle_id == Some("com.ableton.live")
        || name.is_some_and(|name| name.starts_with("Ableton Live"))
}

#[cfg(target_os = "macos")]
unsafe fn ns_string(value: *mut objc::runtime::Object) -> Option<String> {
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CStr;

    if value.is_null() {
        return None;
    }
    let bytes: *const std::os::raw::c_char = msg_send![value, UTF8String];
    if bytes.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(bytes) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

#[cfg(not(target_os = "macos"))]
pub fn ableton_is_frontmost() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::is_ableton_identity;

    #[test]
    fn accepts_suite_and_beta_identity_but_not_other_apps() {
        assert!(is_ableton_identity(Some("com.ableton.live"), Some("Live")));
        assert!(is_ableton_identity(None, Some("Ableton Live 12 Beta")));
        assert!(!is_ableton_identity(
            Some("com.apple.finder"),
            Some("Finder")
        ));
    }
}
