use librespot::oauth;
use std::ffi::c_int;
use std::ffi::{CStr, CString};
use std::ffi::c_char;

use crate::utils::OutputRedirect;

#[repr(C)]
#[derive(Debug)]
pub struct OAuthBuilder {
    client_id: *const c_char,
    redirect_uri: *const c_char,
    message: String,
    auto_open: bool,
    scopes: Vec<String>,
}

#[no_mangle]
pub fn oauth_builder_new(client_id: *const c_char, redirect_uri: *const c_char) -> *mut OAuthBuilder {
    Box::into_raw(Box::new(
        OAuthBuilder {
            client_id,
            redirect_uri,
            message: "".to_string(),
            auto_open: false,
            scopes: vec![]
        }
    ))
}

#[no_mangle]
pub fn oauth_builder_free(oauth_builder: *mut OAuthBuilder) {
    if oauth_builder.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(oauth_builder));
    }
}

#[no_mangle]
pub fn oauth_builder_custom_message(oauth_builder: *mut OAuthBuilder, message: *const c_char) {
    unsafe {
        (*oauth_builder).message = CStr::from_ptr(message).to_str().unwrap().to_string();
    }
}

#[no_mangle]
pub fn oauth_builder_auto_open(oauth_builder: *mut OAuthBuilder) {
    unsafe {
        (*oauth_builder).auto_open = true;
    }
}

#[no_mangle]
pub fn oauth_builder_add_scope(oauth_builder: *mut OAuthBuilder, scope: c_int) {
    unsafe {
        match scope {
            1 => (*oauth_builder).scopes.push("app-remote-control".to_string()),
            2 => (*oauth_builder).scopes.push("playlist-modify".to_string()),
            3 => (*oauth_builder).scopes.push("playlist-modify-private".to_string()),
            4 => (*oauth_builder).scopes.push("playlist-modify-public".to_string()),
            5 => (*oauth_builder).scopes.push("playlist-read".to_string()),
            6 => (*oauth_builder).scopes.push("playlist-read-collaborative".to_string()),
            7 => (*oauth_builder).scopes.push("playlist-read-private".to_string()),
            8 => (*oauth_builder).scopes.push("streaming".to_string()),
            9 => (*oauth_builder).scopes.push("ugc-image-upload".to_string()),
            10 => (*oauth_builder).scopes.push("user-follow-modify".to_string()),
            11 => (*oauth_builder).scopes.push("user-follow-read".to_string()),
            12 => (*oauth_builder).scopes.push("user-library-modify".to_string()),
            13 => (*oauth_builder).scopes.push("user-library-read".to_string()),
            14 => (*oauth_builder).scopes.push("user-modify".to_string()),
            15 => (*oauth_builder).scopes.push("user-modify-playback-state".to_string()),
            16 => (*oauth_builder).scopes.push("user-modify-private".to_string()),
            17 => (*oauth_builder).scopes.push("user-personalized".to_string()),
            18 => (*oauth_builder).scopes.push("user-read-birthdate".to_string()),
            19 => (*oauth_builder).scopes.push("user-read-currently-playing".to_string()),
            20 => (*oauth_builder).scopes.push("user-read-email".to_string()),
            21 => (*oauth_builder).scopes.push("user-read-play-history".to_string()),
            22 => (*oauth_builder).scopes.push("user-read-playback-position".to_string()),
            23 => (*oauth_builder).scopes.push("user-read-playback-state".to_string()),
            24 => (*oauth_builder).scopes.push("user-read-private".to_string()),
            25 => (*oauth_builder).scopes.push("user-read-recently-played".to_string()),
            26 => (*oauth_builder).scopes.push("user-top-read".to_string()),
            27 => {
                for i in 1..=26 {
                    oauth_builder_add_scope(oauth_builder, i);
                }
            },
            _ => return
        }
    }
}

#[repr(C)]
pub struct OAuth {
    client: *mut oauth::OAuthClient,
    token: *mut oauth::OAuthToken,
}

#[no_mangle]
pub fn oauth_build(oauth_builder: *mut OAuthBuilder) -> *mut OAuth {
    unsafe {
        let mut librespot_oauth_builder = oauth::OAuthClientBuilder::new(
            CStr::from_ptr((*oauth_builder).client_id).to_str().unwrap(),
            CStr::from_ptr((*oauth_builder).redirect_uri).to_str().unwrap(),
            (*oauth_builder).scopes.iter().map(|s| s.as_str()).collect()
        );
        if !(&(*oauth_builder)).message.is_empty() {
            librespot_oauth_builder = librespot_oauth_builder.with_custom_message(&(*oauth_builder).message);
        }
        if (*oauth_builder).auto_open {
            librespot_oauth_builder = librespot_oauth_builder.open_in_browser();
        }

        let oauth_client = librespot_oauth_builder.build().unwrap();
        let oauth_token: oauth::OAuthToken;
        
        if (*oauth_builder).auto_open {
            let redirect = OutputRedirect::redirect();
            oauth_token = oauth_client.get_access_token().unwrap();
            redirect.restore();
        } else {
            oauth_token = oauth_client.get_access_token().unwrap();
        }

        oauth_builder_free(oauth_builder);
        Box::into_raw(Box::new(
            OAuth {
                client: Box::into_raw(Box::new(oauth_client)),
                token: Box::into_raw(Box::new(oauth_token))
            }
        ))
    }
}

#[no_mangle]
pub fn oauth_free(oauth: *mut OAuth) {
    if oauth.is_null() {
        return
    }
    unsafe {
        drop(Box::from_raw(oauth));
    }
}

#[no_mangle]
pub fn oauth_access_token(oauth: *mut OAuth) -> *const c_char {
    unsafe {
        CString::new((*(*oauth).token).access_token.clone()).unwrap().into_raw()
    }
}

#[no_mangle]
pub fn oauth_refresh_token(oauth: *mut OAuth) -> *const c_char {
    unsafe {
        CString::new((*(*oauth).token).refresh_token.clone()).unwrap().into_raw()
    }
}

#[no_mangle]
pub fn oauth_refresh_auth(oauth: *mut OAuth, refresh_token: *const c_char) {
    unsafe {
        (*(*oauth).token) = (*(*oauth).client)
            .refresh_token(
                CStr::from_ptr(refresh_token)
                .to_str().unwrap()
            ).unwrap();
    }
}

#[no_mangle]
pub fn oauth_expires_at(oauth: *mut OAuth) -> u64 {
    unsafe {
        let expiration = (*(*oauth).token)
            .expires_at.duration_since(std::time::Instant::now())
            .as_secs();

        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() + expiration,
            Err(_) => 0,
        }
    }
}
