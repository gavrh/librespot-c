use librespot::oauth::OAuthClientBuilder;
use librespot::oauth::OAuthClient;
use librespot::oauth::OAuthToken;
use std::ffi::CStr;
use std::ffi::c_char;

#[repr(C)]
#[derive(Debug)]
pub struct Authenticator {
    builder: *mut OAuthClientBuilder,
}

impl Authenticator {
    pub fn impl_new(client_id: &CStr, redirect_uri: &CStr) -> Self {
        Self {
            builder: Box::into_raw(Box::new(
                OAuthClientBuilder::new(
                    client_id.to_str().unwrap(),
                    redirect_uri.to_str().unwrap(),
                    vec![]
                )
            ))
        }
    }

    pub fn impl_builder_custom_message(&mut self, message: &CStr) {
        unsafe {
            if !self.builder.is_null() {
                let old_builder = Box::from_raw(self.builder);
                let new_builder = old_builder.with_custom_message(message.to_str().unwrap());
                self.builder = Box::into_raw(Box::new(new_builder));
            }
        }
    }

    pub fn impl_builder_open(&mut self) {
        unsafe {
            if !self.builder.is_null() {
                let old_builder = Box::from_raw(self.builder);
                let new_builder = old_builder.open_in_browser();
                self.builder = Box::into_raw(Box::new(new_builder));
            }
        }
    }

    pub fn impl_builder_build(&self) -> Result<OAuth, ()> {
        unsafe {
            if !self.builder.is_null() {
                let builder_raw = Box::from_raw(self.builder);
                let oauth_client = builder_raw.build().unwrap();
                let oauth_token = oauth_client.get_access_token().unwrap();
                Ok(OAuth {
                    client: Box::into_raw(Box::new(oauth_client)),
                    token: Box::into_raw(Box::new(oauth_token))
                })
            } else { Err(()) }
        }
    }
}

#[repr(C)]
pub struct OAuth {
    client: *mut OAuthClient,
    token: *mut OAuthToken,
}

#[no_mangle]
pub fn authenticator_new(client_id: *const c_char, redirect_uri: *const c_char) -> *mut Authenticator {
    unsafe {
        Box::into_raw(Box::new(
            Authenticator::impl_new(
                CStr::from_ptr(client_id),
                CStr::from_ptr(redirect_uri)
            )
        ))
    }
}

#[no_mangle]
pub fn authenticator_custom_message(auth: *mut Authenticator, message: *const c_char) {
    if auth.is_null() {
        return;
    }
    unsafe {
        (*auth).impl_builder_custom_message(CStr::from_ptr(message));
    }
}

#[no_mangle]
pub fn authenticator_open(auth: *mut Authenticator) {
    if auth.is_null() {
        return;
    }
    unsafe {
        (*auth).impl_builder_open();
    }
}

#[no_mangle]
pub fn authenticator_build(auth: *mut Authenticator) -> *mut OAuth {
    unsafe {
        Box::into_raw(Box::new(
            (*auth).impl_builder_build().unwrap()
        ))
    }
}
