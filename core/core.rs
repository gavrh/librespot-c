use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;
use std::ptr;

use librespot::core;
use url::Url;

use crate::discovery::{credentials_ref, Credentials};
use crate::error::{
    clear_error, set_error, LIBRESPOT_ERR_FAILED, LIBRESPOT_ERR_INVALID_ARGUMENT,
    LIBRESPOT_ERR_INVALID_STRING, LIBRESPOT_OK,
};
use crate::ffi::{ffi_guard, ffi_status};
use crate::runtime::try_runtime;

fn string_from_c<'a>(value: *const c_char, name: &str) -> Result<&'a str, c_int> {
    if value.is_null() {
        set_error(format!("{name} must not be NULL"));
        return Err(LIBRESPOT_ERR_INVALID_ARGUMENT);
    }

    unsafe { CStr::from_ptr(value) }.to_str().map_err(|error| {
        set_error(format!("{name} is not valid UTF-8: {error}"));
        LIBRESPOT_ERR_INVALID_STRING
    })
}

fn session_config_mut<'a>(
    session_config: *mut SessionConfig,
) -> Result<&'a mut SessionConfig, c_int> {
    if session_config.is_null() {
        set_error("session_config must not be NULL");
        return Err(LIBRESPOT_ERR_INVALID_ARGUMENT);
    }

    Ok(unsafe { &mut *session_config })
}

#[repr(C)]
pub struct SessionConfig(core::SessionConfig);

#[no_mangle]
pub extern "C" fn session_config_default() -> *mut SessionConfig {
    ffi_guard(ptr::null_mut(), || {
        clear_error();
        Box::into_raw(Box::new(SessionConfig(core::SessionConfig::default())))
    })
}

#[no_mangle]
pub extern "C" fn session_config_free(session_config: *mut SessionConfig) {
    ffi_guard((), || {
        if session_config.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(session_config));
        }
    });
}

#[no_mangle]
pub extern "C" fn session_config_set_client_id(
    session_config: *mut SessionConfig,
    client_id: *const c_char,
) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };
        let client_id = match string_from_c(client_id, "client_id") {
            Ok(client_id) => client_id,
            Err(status) => return status,
        };

        config.0.client_id = client_id.to_owned();
        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn session_config_set_device_id(
    session_config: *mut SessionConfig,
    device_id: *const c_char,
) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };
        let device_id = match string_from_c(device_id, "device_id") {
            Ok(device_id) => device_id,
            Err(status) => return status,
        };

        config.0.device_id = device_id.to_owned();
        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn session_config_set_proxy(
    session_config: *mut SessionConfig,
    proxy: *const c_char,
) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };
        let proxy = match string_from_c(proxy, "proxy") {
            Ok(proxy) => proxy,
            Err(status) => return status,
        };

        match Url::parse(proxy) {
            Ok(proxy) => {
                config.0.proxy = Some(proxy);
                LIBRESPOT_OK
            }
            Err(error) => {
                set_error(format!("proxy is not a valid URL: {error}"));
                LIBRESPOT_ERR_INVALID_STRING
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn session_config_clear_proxy(session_config: *mut SessionConfig) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };

        config.0.proxy = None;
        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn session_config_set_ap_port(
    session_config: *mut SessionConfig,
    ap_port: u16,
) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };

        config.0.ap_port = Some(ap_port);
        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn session_config_clear_ap_port(session_config: *mut SessionConfig) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };

        config.0.ap_port = None;
        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn session_config_set_tmp_dir(
    session_config: *mut SessionConfig,
    tmp_dir: *const c_char,
) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };
        let tmp_dir = match string_from_c(tmp_dir, "tmp_dir") {
            Ok(tmp_dir) => tmp_dir,
            Err(status) => return status,
        };

        config.0.tmp_dir = PathBuf::from(tmp_dir);
        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn session_config_set_autoplay(
    session_config: *mut SessionConfig,
    autoplay: bool,
) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };

        config.0.autoplay = Some(autoplay);
        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn session_config_clear_autoplay(session_config: *mut SessionConfig) -> c_int {
    ffi_status(|| {
        clear_error();

        let config = match session_config_mut(session_config) {
            Ok(config) => config,
            Err(status) => return status,
        };

        config.0.autoplay = None;
        LIBRESPOT_OK
    })
}

#[repr(C)]
pub struct Session(core::Session);

pub(crate) fn session_clone(session: *mut Session) -> Option<core::Session> {
    if session.is_null() {
        return None;
    }

    unsafe { Some((*session).0.clone()) }
}

pub fn session_box(session: *mut Session) -> Box<core::Session> {
    Box::new(session_clone(session).expect("session pointer must not be NULL"))
}

#[no_mangle]
pub extern "C" fn session_new(session_config: *mut SessionConfig) -> *mut Session {
    ffi_guard(ptr::null_mut(), || {
        clear_error();

        if session_config.is_null() {
            set_error("session_config must not be NULL");
            return ptr::null_mut();
        }

        let config = unsafe { (*session_config).0.clone() };
        let runtime = match try_runtime() {
            Ok(runtime) => runtime,
            Err(error) => {
                set_error(format!("failed to create Tokio runtime: {error}"));
                return ptr::null_mut();
            }
        };

        let _guard = runtime.enter();
        let session = core::Session::new(config, None);

        Box::into_raw(Box::new(Session(session)))
    })
}

#[no_mangle]
pub extern "C" fn session_free(session: *mut Session) {
    ffi_guard((), || {
        if session.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(session));
        }
    });
}

#[no_mangle]
pub extern "C" fn session_connect(session: *mut Session, credentials: *mut Credentials) -> c_int {
    ffi_status(|| {
        clear_error();

        if session.is_null() {
            set_error("session must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }
        if credentials.is_null() {
            set_error("credentials must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let runtime = match try_runtime() {
            Ok(runtime) => runtime,
            Err(error) => {
                set_error(format!("failed to create Tokio runtime: {error}"));
                return LIBRESPOT_ERR_FAILED;
            }
        };
        let credentials = credentials_ref(credentials).clone();

        let result = runtime.block_on(unsafe { &(*session).0 }.connect(credentials, false));
        match result {
            Ok(()) => LIBRESPOT_OK,
            Err(error) => {
                set_error(format!("failed to connect session: {error}"));
                LIBRESPOT_ERR_FAILED
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn session_is_valid(session: *mut Session) -> bool {
    ffi_guard(false, || {
        clear_error();

        if session.is_null() {
            set_error("session must not be NULL");
            return false;
        }

        unsafe { !(*session).0.is_invalid() }
    })
}

#[no_mangle]
pub extern "C" fn session_shutdown(session: *mut Session) {
    ffi_guard((), || {
        if session.is_null() {
            return;
        }

        unsafe {
            (*session).0.shutdown();
        }
    });
}

#[repr(C)]
pub struct SpotifyUri {
    uri: core::SpotifyUri,
    string: CString,
    item_type: CString,
}

fn spotify_uri_from_core(uri: core::SpotifyUri) -> *mut SpotifyUri {
    let string = match CString::new(uri.to_uri()) {
        Ok(string) => string,
        Err(_) => {
            set_error("failed to encode Spotify URI as a C string");
            return ptr::null_mut();
        }
    };
    let item_type = match CString::new(uri.item_type()) {
        Ok(item_type) => item_type,
        Err(_) => {
            set_error("failed to encode Spotify item type as a C string");
            return ptr::null_mut();
        }
    };

    Box::into_raw(Box::new(SpotifyUri {
        uri,
        string,
        item_type,
    }))
}

pub(crate) fn spotify_uri_new_internal(spotify_uri: core::SpotifyUri) -> *mut SpotifyUri {
    spotify_uri_from_core(spotify_uri)
}

#[no_mangle]
pub extern "C" fn spotify_uri_new(uri: *const c_char) -> *mut SpotifyUri {
    ffi_guard(ptr::null_mut(), || {
        clear_error();

        let uri = match string_from_c(uri, "uri") {
            Ok(uri) => uri,
            Err(_) => return ptr::null_mut(),
        };

        match core::SpotifyUri::from_uri(uri) {
            Ok(uri) => spotify_uri_from_core(uri),
            Err(error) => {
                set_error(format!("failed to parse Spotify URI: {error}"));
                ptr::null_mut()
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn spotify_uri_free(spotify_uri: *mut SpotifyUri) {
    ffi_guard((), || {
        if spotify_uri.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(spotify_uri));
        }
    });
}

#[no_mangle]
pub extern "C" fn spotify_uri_to_string(spotify_uri: *mut SpotifyUri) -> *const c_char {
    ffi_guard(ptr::null(), || {
        clear_error();

        if spotify_uri.is_null() {
            set_error("spotify_uri must not be NULL");
            return ptr::null();
        }

        unsafe { (*spotify_uri).string.as_ptr() }
    })
}

#[no_mangle]
pub extern "C" fn spotify_uri_item_type(spotify_uri: *mut SpotifyUri) -> *const c_char {
    ffi_guard(ptr::null(), || {
        clear_error();

        if spotify_uri.is_null() {
            set_error("spotify_uri must not be NULL");
            return ptr::null();
        }

        unsafe { (*spotify_uri).item_type.as_ptr() }
    })
}

#[no_mangle]
pub extern "C" fn spotify_uri_is_playable(spotify_uri: *mut SpotifyUri) -> bool {
    ffi_guard(false, || {
        clear_error();

        if spotify_uri.is_null() {
            set_error("spotify_uri must not be NULL");
            return false;
        }

        unsafe { (*spotify_uri).uri.is_playable() }
    })
}
