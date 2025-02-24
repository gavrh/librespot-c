use librespot::discovery;
use std::ffi::{c_char, CStr};

#[repr(C)]
pub struct Credentials {
    pub credentials: *mut discovery::Credentials,
}

#[no_mangle]
pub fn credentials_new(access_token: *const c_char) -> *mut Credentials {
    unsafe {
        Box::into_raw(Box::new(
            Credentials {
                credentials: Box::into_raw(Box::new(
                    discovery::Credentials::with_access_token(
                        CStr::from_ptr(access_token).to_str().unwrap()
                    )
                ))
            }
        ))
    }
}
