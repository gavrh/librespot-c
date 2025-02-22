use librespot::oauth::OAuthClientBuilder;
use librespot::oauth::OAuthClient;
use librespot::oauth::OAuthToken;
use std::ffi::{CStr, CString};
use std::ffi::c_char;

#[repr(C)]
#[derive(Debug)]
pub struct Authenticator {
    builder: *mut OAuthClientBuilder,
}

#[repr(C)]
pub struct OAuth {
    client: *mut OAuthClient,
    token: *mut OAuthToken,
}

#[no_mangle]
pub fn authenticator_new(client_id: *const c_char, redirect_uri: *const c_char) -> *mut Authenticator {
    Box::into_raw(Box::new(
        Authenticator {
            builder: Box::into_raw(Box::new(
                unsafe {
                    OAuthClientBuilder::new(
                        CStr::from_ptr(client_id).to_str().unwrap(),
                        CStr::from_ptr(redirect_uri).to_str().unwrap(),
                        vec![
                            "app-remote-control",
                            "playlist-modify",
                            "playlist-modify-private",
                            "playlist-modify-public",
                            "playlist-read",
                            "playlist-read-collaborative",
                            "playlist-read-private",
                            "streaming",
                            "ugc-image-upload",
                            "user-follow-modify",
                            "user-follow-read",
                            "user-library-modify",
                            "user-library-read",
                            "user-modify",
                            "user-modify-playback-state",
                            "user-modify-private",
                            "user-personalized",
                            "user-read-birthdate",
                            "user-read-currently-playing",
                            "user-read-email",
                            "user-read-play-history",
                            "user-read-playback-position",
                            "user-read-playback-state",
                            "user-read-private",
                            "user-read-recently-played",
                            "user-top-read"
                        ]
                    )
                }
            ))
        }
    ))
}

#[no_mangle]
pub fn authenticator_custom_message(auther: *mut Authenticator, message: *const c_char) {
    if auther.is_null() {
        return;
    }
    unsafe {
        if !(*auther).builder.is_null() {
            let old_builder = Box::from_raw((*auther).builder);
            let new_builder = old_builder.with_custom_message(CStr::from_ptr(message).to_str().unwrap());
            (*auther).builder = Box::into_raw(Box::new(new_builder));
        }
    }
}

#[no_mangle]
pub fn authenticator_open(auther: *mut Authenticator) {
    if auther.is_null() {
        return;
    }
    unsafe {
        if !(*auther).builder.is_null() {
            let old_builder = Box::from_raw((*auther).builder);
            let new_builder = old_builder.open_in_browser();
            (*auther).builder = Box::into_raw(Box::new(new_builder));
        }
    }
}

#[no_mangle]
pub fn authenticator_build(auther: *mut Authenticator) -> *mut OAuth {
    unsafe {
        let builder_raw = Box::from_raw((*auther).builder);
        let oauth_client = builder_raw.build().unwrap();
        let oauth_token = oauth_client.get_access_token().unwrap();

        authenticator_free(auther);
        Box::into_raw(Box::new(
            OAuth {
                client: Box::into_raw(Box::new(oauth_client)),
                token: Box::into_raw(Box::new(oauth_token))
            }
        ))
    }
}

#[no_mangle]
pub fn authenticator_free(auther: *mut Authenticator) {
    if auther.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(auther));
    }
}

#[no_mangle]
pub fn oauth_access_token(auth: *mut OAuth) -> *const c_char {
    unsafe {
        let token_raw = Box::from_raw((*auth).token);
        let cstring_access_token = CString::new(token_raw.access_token).unwrap();
        cstring_access_token.into_raw()
    }
}

#[no_mangle]
pub fn oauth_refresh_token(auth: *mut OAuth) -> *const c_char {
    unsafe {
        let token_raw = Box::from_raw((*auth).token);
        println!("{}", token_raw.refresh_token);
        let cstring_refresh_token = CString::new(token_raw.refresh_token).unwrap();
        cstring_refresh_token.into_raw()
    }
}
