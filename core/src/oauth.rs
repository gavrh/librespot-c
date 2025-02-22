use librespot::oauth::OAuthClientBuilder;
use librespot::oauth::OAuthClient;
use librespot::oauth::OAuthToken;
use std::ffi::c_int;
use std::ffi::{CStr, CString};
use std::ffi::c_char;

#[repr(C)]
#[derive(Debug)]
pub struct Authenticator {
    client_id: *const c_char,
    redirect_uri: *const c_char,
    message: String,
    auto_open: bool,
    scopes: Vec<String>,
}

#[no_mangle]
pub fn authenticator_new(client_id: *const c_char, redirect_uri: *const c_char) -> *mut Authenticator {
    Box::into_raw(Box::new(
        Authenticator {
            client_id,
            redirect_uri,
            message: "".to_string(),
            auto_open: false,
            scopes: vec![]
        }
    ))
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
pub fn authenticator_custom_message(auther: *mut Authenticator, message: *const c_char) {
    if auther.is_null() {
        return;
    }
    unsafe {
        (*auther).message = CStr::from_ptr(message).to_str().unwrap().to_string();
    }
}

#[no_mangle]
pub fn authenticator_auto_open(auther: *mut Authenticator) {
    if auther.is_null() {
        return;
    }
    unsafe {
        (*auther).auto_open = true;
    }
}

#[no_mangle]
pub fn authenticator_add_scope(auther: *mut Authenticator, scope: c_int) {
    if auther.is_null() {
        return;
    }
    unsafe {
        match scope {
            1 => (*auther).scopes.push("app-remote-control".to_string()),
            2 => (*auther).scopes.push("playlist-modify".to_string()),
            3 => (*auther).scopes.push("playlist-modify-private".to_string()),
            4 => (*auther).scopes.push("playlist-modify-public".to_string()),
            5 => (*auther).scopes.push("playlist-read".to_string()),
            6 => (*auther).scopes.push("playlist-read-collaborative".to_string()),
            7 => (*auther).scopes.push("playlist-read-private".to_string()),
            8 => (*auther).scopes.push("streaming".to_string()),
            9 => (*auther).scopes.push("ugc-image-upload".to_string()),
            10 => (*auther).scopes.push("user-follow-modify".to_string()),
            11 => (*auther).scopes.push("user-follow-read".to_string()),
            12 => (*auther).scopes.push("user-library-modify".to_string()),
            13 => (*auther).scopes.push("user-library-read".to_string()),
            14 => (*auther).scopes.push("user-modify".to_string()),
            15 => (*auther).scopes.push("user-modify-playback-state".to_string()),
            16 => (*auther).scopes.push("user-modify-private".to_string()),
            17 => (*auther).scopes.push("user-personalized".to_string()),
            18 => (*auther).scopes.push("user-read-birthdate".to_string()),
            19 => (*auther).scopes.push("user-read-currently-playing".to_string()),
            20 => (*auther).scopes.push("user-read-email".to_string()),
            21 => (*auther).scopes.push("user-read-play-history".to_string()),
            22 => (*auther).scopes.push("user-read-playback-position".to_string()),
            23 => (*auther).scopes.push("user-read-playback-state".to_string()),
            24 => (*auther).scopes.push("user-read-private".to_string()),
            25 => (*auther).scopes.push("user-read-recently-played".to_string()),
            26 => (*auther).scopes.push("user-top-read".to_string()),
            27 => {
                (*auther).scopes.push("app-remote-control".to_string());
                (*auther).scopes.push("playlist-modify".to_string());
                (*auther).scopes.push("playlist-modify-private".to_string());
                (*auther).scopes.push("playlist-modify-public".to_string());
                (*auther).scopes.push("playlist-read".to_string());
                (*auther).scopes.push("playlist-read-collaborative".to_string());
                (*auther).scopes.push("playlist-read-private".to_string());
                (*auther).scopes.push("streaming".to_string());
                (*auther).scopes.push("ugc-image-upload".to_string());
                (*auther).scopes.push("user-follow-modify".to_string());
                (*auther).scopes.push("user-follow-read".to_string());
                (*auther).scopes.push("user-library-modify".to_string());
                (*auther).scopes.push("user-library-read".to_string());
                (*auther).scopes.push("user-modify".to_string());
                (*auther).scopes.push("user-modify-playback-state".to_string());
                (*auther).scopes.push("user-modify-private".to_string());
                (*auther).scopes.push("user-personalized".to_string());
                (*auther).scopes.push("user-read-birthdate".to_string());
                (*auther).scopes.push("user-read-currently-playing".to_string());
                (*auther).scopes.push("user-read-email".to_string());
                (*auther).scopes.push("user-read-play-history".to_string());
                (*auther).scopes.push("user-read-playback-position".to_string());
                (*auther).scopes.push("user-read-playback-state".to_string());
                (*auther).scopes.push("user-read-private".to_string());
                (*auther).scopes.push("user-read-recently-played".to_string());
                (*auther).scopes.push("user-top-read".to_string());
            },
            _ => return
        }
    }
}

#[repr(C)]
pub struct OAuth {
    client: *mut OAuthClient,
    token: *mut OAuthToken,
}

#[no_mangle]
pub fn authenticator_build(auther: *mut Authenticator) -> *mut OAuth {
    unsafe {
        let mut oauth_builder = OAuthClientBuilder::new(
            CStr::from_ptr((*auther).client_id).to_str().unwrap(),
            CStr::from_ptr((*auther).redirect_uri).to_str().unwrap(),
            (*auther).scopes.iter().map(|s| s.as_str()).collect()
        );
        if !(*auther).message.is_empty() {
            oauth_builder = oauth_builder.with_custom_message(&(*auther).message);
        }
        if (*auther).auto_open {
            oauth_builder = oauth_builder.open_in_browser();
        }

        let oauth_client = oauth_builder.build().unwrap();
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
pub fn oauth_free(auth: *mut OAuth) {
    if auth.is_null() {
        return
    }
    unsafe {
        drop(Box::from_raw(auth));
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
