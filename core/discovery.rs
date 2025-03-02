use librespot::discovery;
use std::ffi::{c_char, CStr};

#[repr(C)]
pub struct Credentials(*mut discovery::Credentials);

pub fn credentials_ref<'a>(credentials: *mut Credentials) -> &'a discovery::Credentials {
    unsafe {
        &*(*credentials).0
    }
}

#[no_mangle]
pub fn credentials_new(access_token: *const c_char) -> *mut Credentials {
    unsafe {
        Box::into_raw(Box::new(
            Credentials(Box::into_raw(Box::new(
                discovery::Credentials::with_access_token(
                    CStr::from_ptr(access_token).to_str().unwrap()
                )
            )))
        ))
    }
}

#[no_mangle]
pub fn credentials_free(credentials: *mut Credentials) {
    if credentials.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(credentials));
    }
}
