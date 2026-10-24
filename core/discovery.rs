use crate::error::{clear_error, set_error};
use crate::ffi::ffi_guard;
use librespot::discovery;
use std::ffi::{c_char, CStr};
use std::ptr::null_mut;

#[repr(C)]
pub struct Credentials(discovery::Credentials);

pub fn credentials_ref<'a>(credentials: *mut Credentials) -> &'a discovery::Credentials {
    unsafe { &(*credentials).0 }
}

#[no_mangle]
pub fn credentials_new(access_token: *const c_char) -> *mut Credentials {
    ffi_guard(null_mut(), || {
        clear_error();

        if access_token.is_null() {
            set_error("access token must not be NULL");
            return null_mut();
        }

        let access_token = unsafe { CStr::from_ptr(access_token) };
        let access_token = match access_token.to_str() {
            Ok(access_token) => access_token,
            Err(_) => {
                set_error("access token must be valid UTF-8");
                return null_mut();
            }
        };

        let credentials = discovery::Credentials::with_access_token(access_token);
        Box::into_raw(Box::new(Credentials(credentials)))
    })
}

#[no_mangle]
pub fn credentials_free(credentials: *mut Credentials) {
    ffi_guard((), || {
        if credentials.is_null() {
            return;
        }
        unsafe {
            drop(Box::from_raw(credentials));
        }
    });
}
