//! IOKit power assertion to prevent system idle sleep while the overlay is up.
//!
//! Hand-rolled FFI to IOKit's `IOPMAssertionCreateWithName` /
//! `IOPMAssertionRelease`. We hold a `PreventUserIdleSystemSleep` assertion
//! while presenting (so the Mac doesn't doze with the overlay showing) and
//! release it on unlock. Gated on the `prevent_sleep` setting.

#![allow(dead_code)]

#[cfg(target_os = "macos")]
mod imp {
    use core_foundation::base::TCFType;
    use core_foundation::string::{CFString, CFStringRef};
    use std::os::raw::{c_int, c_uint};

    // IOPMAssertionLevel
    const K_IOPM_ASSERTION_LEVEL_ON: c_uint = 255;
    // Return code for success.
    const K_IO_RETURN_SUCCESS: c_int = 0;

    type IOPMAssertionID = c_uint;

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOPMAssertionCreateWithName(
            assertion_type: CFStringRef,
            assertion_level: c_uint,
            assertion_name: CFStringRef,
            assertion_id: *mut IOPMAssertionID,
        ) -> c_int;

        fn IOPMAssertionRelease(assertion_id: IOPMAssertionID) -> c_int;
    }

    /// Acquire an assertion preventing idle system sleep. Returns its id.
    pub fn acquire() -> Result<u32, String> {
        let assertion_type = CFString::from_static_string("PreventUserIdleSystemSleep");
        let name = CFString::from_static_string("Veil overlay is showing");
        let mut id: IOPMAssertionID = 0;

        // SAFETY: standard IOKit call with valid CFString refs and an out-param.
        let rc = unsafe {
            IOPMAssertionCreateWithName(
                assertion_type.as_concrete_TypeRef(),
                K_IOPM_ASSERTION_LEVEL_ON,
                name.as_concrete_TypeRef(),
                &mut id,
            )
        };

        if rc == K_IO_RETURN_SUCCESS {
            Ok(id)
        } else {
            Err(format!("IOPMAssertionCreateWithName failed ({rc})"))
        }
    }

    /// Release a previously-acquired assertion.
    pub fn release(id: u32) {
        // SAFETY: releasing a valid assertion id; double-release is harmless.
        let rc = unsafe { IOPMAssertionRelease(id) };
        if rc != K_IO_RETURN_SUCCESS {
            log::warn!("IOPMAssertionRelease failed ({rc})");
        }
    }
}

#[cfg(target_os = "macos")]
pub use imp::{acquire, release};

#[cfg(not(target_os = "macos"))]
pub fn acquire() -> Result<u32, String> {
    Err("power assertion is only implemented on macOS".into())
}

#[cfg(not(target_os = "macos"))]
pub fn release(_id: u32) {}
