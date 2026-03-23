/// Platform-specific helpers shared across modules.
///
/// This module breaks the circular dependency between `window` and `game_tracker`
/// by providing HWND ↔ isize conversion utilities in a neutral location.

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

/// Convert an isize handle value to a Win32 HWND.
#[cfg(windows)]
#[inline]
pub fn hwnd_from_isize(val: isize) -> HWND {
    HWND(val as *mut std::ffi::c_void)
}

/// Convert a Win32 HWND to an isize handle value.
#[cfg(windows)]
#[inline]
pub fn hwnd_to_isize(hwnd: HWND) -> isize {
    hwnd.0 as isize
}
