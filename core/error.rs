use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

#[allow(dead_code)]
pub(crate) const LIBRESPOT_OK: c_int = 0;
#[allow(dead_code)]
pub(crate) const LIBRESPOT_ERR_INVALID_ARGUMENT: c_int = -1;
#[allow(dead_code)]
pub(crate) const LIBRESPOT_ERR_INVALID_STRING: c_int = -2;
pub(crate) const LIBRESPOT_ERR_FAILED: c_int = -3;

static EMPTY_ERROR: &[u8] = b"\0";

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = RefCell::new(None);
}

pub(crate) fn set_error(message: impl AsRef<str>) {
    let message = CString::new(message.as_ref().replace('\0', " "));

    if let Ok(message) = message {
        LAST_ERROR.with(|last_error| {
            *last_error.borrow_mut() = Some(message);
        });
    }
}

pub(crate) fn clear_error() {
    LAST_ERROR.with(|last_error| {
        *last_error.borrow_mut() = None;
    });
}

#[no_mangle]
pub extern "C" fn librespot_last_error() -> *const c_char {
    LAST_ERROR.with(|last_error| match last_error.borrow().as_ref() {
        Some(message) => message.as_ptr(),
        None => EMPTY_ERROR.as_ptr() as *const c_char,
    })
}

#[no_mangle]
pub extern "C" fn librespot_clear_error() {
    clear_error();
}
