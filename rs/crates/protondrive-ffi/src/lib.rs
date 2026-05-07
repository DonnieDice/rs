// Phase 4 deliverable. C ABI surface lives here.
// unsafe_code intentionally allowed in this crate only.

// Placeholder export so the crate links cleanly before Phase 4.
#[no_mangle]
pub extern "C" fn protondrive_version() -> *const std::ffi::c_char {
    static VERSION: &std::ffi::CStr =
        unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"0.1.0\0") };
    VERSION.as_ptr()
}
