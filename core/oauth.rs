use librespot::oauth;
use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::error::{
    clear_error, set_error, LIBRESPOT_ERR_FAILED, LIBRESPOT_ERR_INVALID_ARGUMENT,
    LIBRESPOT_ERR_INVALID_STRING, LIBRESPOT_OK,
};
use crate::ffi::{ffi_guard, ffi_status};
use crate::utils::OutputRedirect;

const OAUTH_SCOPE_ALL: c_int = 27;

const OAUTH_SCOPES: &[(c_int, &str)] = &[
    (1, "app-remote-control"),
    (2, "playlist-modify"),
    (3, "playlist-modify-private"),
    (4, "playlist-modify-public"),
    (5, "playlist-read"),
    (6, "playlist-read-collaborative"),
    (7, "playlist-read-private"),
    (8, "streaming"),
    (9, "ugc-image-upload"),
    (10, "user-follow-modify"),
    (11, "user-follow-read"),
    (12, "user-library-modify"),
    (13, "user-library-read"),
    (14, "user-modify"),
    (15, "user-modify-playback-state"),
    (16, "user-modify-private"),
    (17, "user-personalized"),
    (18, "user-read-birthdate"),
    (19, "user-read-currently-playing"),
    (20, "user-read-email"),
    (21, "user-read-play-history"),
    (22, "user-read-playback-position"),
    (23, "user-read-playback-state"),
    (24, "user-read-private"),
    (25, "user-read-recently-played"),
    (26, "user-top-read"),
];

#[repr(C)]
#[derive(Debug)]
pub struct OAuthBuilder {
    client_id: CString,
    redirect_uri: CString,
    message: CString,
    auto_open: bool,
    scopes: Vec<String>,
}

#[repr(C)]
pub struct OAuth {
    client: oauth::OAuthClient,
    token: oauth::OAuthToken,
    access_token: CString,
    refresh_token: CString,
}

fn to_c_string(value: &str, name: &str) -> Result<CString, c_int> {
    match CString::new(value) {
        Ok(value) => Ok(value),
        Err(_) => {
            set_error(format!("{name} contains a NUL byte"));
            Err(LIBRESPOT_ERR_INVALID_STRING)
        }
    }
}

fn copy_c_string(value: *const c_char, name: &str) -> Result<CString, c_int> {
    if value.is_null() {
        set_error(format!("{name} must not be NULL"));
        return Err(LIBRESPOT_ERR_INVALID_ARGUMENT);
    }

    let value = unsafe { CStr::from_ptr(value) };
    let value = match value.to_str() {
        Ok(value) => value,
        Err(_) => {
            set_error(format!("{name} is not valid UTF-8"));
            return Err(LIBRESPOT_ERR_INVALID_STRING);
        }
    };

    to_c_string(value, name)
}

fn scope_name(scope: c_int) -> Option<&'static str> {
    OAUTH_SCOPES
        .iter()
        .find(|entry| entry.0 == scope)
        .map(|(_, name)| *name)
}

fn add_scope(oauth_builder: &mut OAuthBuilder, scope: &str) {
    if !oauth_builder
        .scopes
        .iter()
        .any(|existing| existing.as_str() == scope)
    {
        oauth_builder.scopes.push(scope.to_string());
    }
}

fn token_c_strings(token: &oauth::OAuthToken) -> Result<(CString, CString), c_int> {
    let access_token = to_c_string(&token.access_token, "access token")?;
    let refresh_token = to_c_string(&token.refresh_token, "refresh token")?;

    Ok((access_token, refresh_token))
}

#[no_mangle]
pub extern "C" fn oauth_builder_new(
    client_id: *const c_char,
    redirect_uri: *const c_char,
) -> *mut OAuthBuilder {
    ffi_guard(ptr::null_mut(), || {
        clear_error();

        let client_id = match copy_c_string(client_id, "client_id") {
            Ok(client_id) => client_id,
            Err(_) => return ptr::null_mut(),
        };
        let redirect_uri = match copy_c_string(redirect_uri, "redirect_uri") {
            Ok(redirect_uri) => redirect_uri,
            Err(_) => return ptr::null_mut(),
        };

        Box::into_raw(Box::new(OAuthBuilder {
            client_id,
            redirect_uri,
            message: CString::default(),
            auto_open: false,
            scopes: Vec::new(),
        }))
    })
}

#[no_mangle]
pub extern "C" fn oauth_builder_free(oauth_builder: *mut OAuthBuilder) {
    ffi_guard((), || {
        if oauth_builder.is_null() {
            return;
        }
        unsafe {
            drop(Box::from_raw(oauth_builder));
        }
    });
}

#[no_mangle]
pub extern "C" fn oauth_builder_custom_message(
    oauth_builder: *mut OAuthBuilder,
    message: *const c_char,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if oauth_builder.is_null() {
            set_error("oauth_builder must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let message = match copy_c_string(message, "message") {
            Ok(message) => message,
            Err(status) => return status,
        };

        unsafe {
            (*oauth_builder).message = message;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn oauth_builder_auto_open(oauth_builder: *mut OAuthBuilder) -> c_int {
    ffi_status(|| {
        clear_error();

        if oauth_builder.is_null() {
            set_error("oauth_builder must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*oauth_builder).auto_open = true;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn oauth_builder_add_scope(
    oauth_builder: *mut OAuthBuilder,
    scope: c_int,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if oauth_builder.is_null() {
            set_error("oauth_builder must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let oauth_builder = unsafe { &mut *oauth_builder };

        if scope == OAUTH_SCOPE_ALL {
            for &(_, name) in OAUTH_SCOPES {
                add_scope(oauth_builder, name);
            }
            return LIBRESPOT_OK;
        }

        match scope_name(scope) {
            Some(name) => {
                add_scope(oauth_builder, name);
                LIBRESPOT_OK
            }
            None => {
                set_error(format!("invalid OAuth scope: {scope}"));
                LIBRESPOT_ERR_INVALID_ARGUMENT
            }
        }
    })
}

fn build_oauth(oauth_builder: *mut OAuthBuilder) -> *mut OAuth {
    if oauth_builder.is_null() {
        set_error("oauth_builder must not be NULL");
        return ptr::null_mut();
    }

    let oauth_builder = unsafe { &*oauth_builder };

    let client_id = match oauth_builder.client_id.to_str() {
        Ok(client_id) => client_id,
        Err(_) => {
            set_error("client_id is not valid UTF-8");
            return ptr::null_mut();
        }
    };
    let redirect_uri = match oauth_builder.redirect_uri.to_str() {
        Ok(redirect_uri) => redirect_uri,
        Err(_) => {
            set_error("redirect_uri is not valid UTF-8");
            return ptr::null_mut();
        }
    };
    let message = match oauth_builder.message.to_str() {
        Ok(message) => message,
        Err(_) => {
            set_error("message is not valid UTF-8");
            return ptr::null_mut();
        }
    };

    let mut client_builder = oauth::OAuthClientBuilder::new(
        client_id,
        redirect_uri,
        oauth_builder.scopes.iter().map(String::as_str).collect(),
    );

    if !message.is_empty() {
        client_builder = client_builder.with_custom_message(message);
    }
    if oauth_builder.auto_open {
        client_builder = client_builder.open_in_browser();
    }

    let client = match client_builder.build() {
        Ok(client) => client,
        Err(error) => {
            set_error(format!("failed to build OAuth client: {error}"));
            return ptr::null_mut();
        }
    };

    let redirect = if oauth_builder.auto_open {
        Some(OutputRedirect::redirect())
    } else {
        None
    };
    let token = client.get_access_token();
    drop(redirect);

    let token = match token {
        Ok(token) => token,
        Err(error) => {
            set_error(format!("failed to get OAuth access token: {error}"));
            return ptr::null_mut();
        }
    };

    let (access_token, refresh_token) = match token_c_strings(&token) {
        Ok(tokens) => tokens,
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(OAuth {
        client,
        token,
        access_token,
        refresh_token,
    }))
}

#[no_mangle]
pub extern "C" fn oauth_build(oauth_builder: *mut OAuthBuilder) -> *mut OAuth {
    ffi_guard(ptr::null_mut(), || {
        clear_error();
        build_oauth(oauth_builder)
    })
}

#[no_mangle]
pub extern "C" fn oauth_free(oauth: *mut OAuth) {
    ffi_guard((), || {
        if oauth.is_null() {
            return;
        }
        unsafe {
            drop(Box::from_raw(oauth));
        }
    });
}

#[no_mangle]
pub extern "C" fn oauth_access_token(oauth: *mut OAuth) -> *const c_char {
    ffi_guard(ptr::null(), || {
        clear_error();

        if oauth.is_null() {
            set_error("oauth must not be NULL");
            return ptr::null();
        }

        unsafe { (*oauth).access_token.as_ptr() }
    })
}

#[no_mangle]
pub extern "C" fn oauth_refresh_token(oauth: *mut OAuth) -> *const c_char {
    ffi_guard(ptr::null(), || {
        clear_error();

        if oauth.is_null() {
            set_error("oauth must not be NULL");
            return ptr::null();
        }

        unsafe { (*oauth).refresh_token.as_ptr() }
    })
}

#[no_mangle]
pub extern "C" fn oauth_refresh_auth(
    oauth: *mut OAuth,
    refresh_token: *const c_char,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if oauth.is_null() {
            set_error("oauth must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let refresh_token = match copy_c_string(refresh_token, "refresh_token") {
            Ok(refresh_token) => refresh_token,
            Err(status) => return status,
        };
        let refresh_token = match refresh_token.to_str() {
            Ok(refresh_token) => refresh_token,
            Err(_) => {
                set_error("refresh_token is not valid UTF-8");
                return LIBRESPOT_ERR_INVALID_STRING;
            }
        };

        let oauth = unsafe { &mut *oauth };
        let token = match oauth.client.refresh_token(refresh_token) {
            Ok(token) => token,
            Err(error) => {
                set_error(format!("failed to refresh OAuth token: {error}"));
                return LIBRESPOT_ERR_FAILED;
            }
        };

        let (access_token, refresh_token) = match token_c_strings(&token) {
            Ok(tokens) => tokens,
            Err(status) => return status,
        };

        oauth.token = token;
        oauth.access_token = access_token;
        oauth.refresh_token = refresh_token;

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn oauth_expires_at(oauth: *mut OAuth) -> u64 {
    ffi_guard(0, || {
        if oauth.is_null() {
            return 0;
        }

        let oauth = unsafe { &*oauth };
        let remaining = match oauth
            .token
            .expires_at
            .checked_duration_since(Instant::now())
        {
            Some(remaining) => remaining,
            None => return 0,
        };
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(now) => now,
            Err(_) => return 0,
        };

        now.as_secs().saturating_add(remaining.as_secs())
    })
}
