use std::any::Any;
use std::os::raw::c_int;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::error::{set_error, LIBRESPOT_ERR_FAILED};

#[allow(dead_code)]
pub(crate) fn ffi_guard<T, F>(default: T, body: F) -> T
where
    F: FnOnce() -> T,
{
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => value,
        Err(payload) => {
            set_error(panic_message(payload.as_ref()));
            default
        }
    }
}

#[allow(dead_code)]
pub(crate) fn ffi_status<F>(body: F) -> c_int
where
    F: FnOnce() -> c_int,
{
    ffi_guard(LIBRESPOT_ERR_FAILED, body)
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    let message = if let Some(message) = payload.downcast_ref::<&str>() {
        *message
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.as_str()
    } else {
        "unknown panic"
    };

    format!("panic in FFI call: {message}")
}
