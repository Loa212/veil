//! Native macOS lock.
//!
//! Locks the session immediately via the private `SACLockScreenImmediate`
//! function in `login.framework`, resolved at runtime with `dlopen`/`dlsym`.
//! This needs NO permission (no Accessibility), no synthetic keystroke, and
//! works regardless of the "require password after sleep" setting — unlike
//! `pmset displaysleepnow` or the osascript Cmd+Ctrl+Q trick.
//!
//! It is a private API: fine for Homebrew distribution, but bars the Mac App
//! Store and could change across macOS releases (hence the explicit fallbacks
//! aren't worth carrying yet — if the symbol ever vanishes we surface an error).

#[cfg(target_os = "macos")]
pub fn lock_screen() -> Result<(), String> {
    use std::ffi::CString;
    use std::os::raw::{c_int, c_void};

    const LOGIN_FRAMEWORK: &str = "/System/Library/PrivateFrameworks/login.framework/login";
    const SYMBOL: &str = "SACLockScreenImmediate";

    type LockFn = unsafe extern "C" fn() -> c_int;

    let path = CString::new(LOGIN_FRAMEWORK).unwrap();
    let symbol = CString::new(SYMBOL).unwrap();

    // SAFETY: standard dlopen/dlsym against a system framework. We null-check the
    // handle and symbol before calling, and the resolved function takes no args.
    unsafe {
        let handle = libc::dlopen(path.as_ptr(), libc::RTLD_LAZY);
        if handle.is_null() {
            return Err(format!("dlopen({LOGIN_FRAMEWORK}) failed"));
        }

        let sym = libc::dlsym(handle, symbol.as_ptr());
        if sym.is_null() {
            libc::dlclose(handle);
            return Err(format!("dlsym({SYMBOL}) failed"));
        }

        let lock_fn: LockFn = std::mem::transmute::<*mut c_void, LockFn>(sym);
        let rc = lock_fn();
        libc::dlclose(handle);

        log::info!("SACLockScreenImmediate returned {rc}");
        if rc == 0 {
            Ok(())
        } else {
            Err(format!("SACLockScreenImmediate returned {rc}"))
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn lock_screen() -> Result<(), String> {
    Err("native lock is only implemented on macOS".into())
}
