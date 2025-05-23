use librespot::core;

use crate::discovery::{credentials_free, credentials_ref, Credentials};
use crate::runtime::runtime;

#[repr(C)]
pub struct SessionConfig(*mut core::SessionConfig);

#[no_mangle]
pub fn session_config_default() -> *mut SessionConfig {
    Box::into_raw(Box::new(
        SessionConfig(Box::into_raw(Box::new(
            core::SessionConfig::default()
        )))
    ))
}

#[no_mangle]
pub fn session_config_free(session_config: *mut SessionConfig) {
    if session_config.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(session_config));
    }
}

#[repr(C)]
pub struct Session(*mut core::Session);

pub fn session_box(session: *mut Session) -> Box<core::Session> {
    unsafe {
        Box::from_raw((*session).0)
    }
}

#[no_mangle]
pub fn session_new(session_config: *mut SessionConfig) -> *mut Session {
    unsafe {
        let new_session = Box::into_raw(Box::new(
            Session(Box::into_raw(Box::new(
                runtime().block_on(async {
                    core::Session::new(
                        *Box::from_raw((*session_config).0),
                        None
                    )
                })
            )))
        ));

        session_config_free(session_config);
        return new_session;
    }
}

#[no_mangle]
pub fn session_free(session: *mut Session) {
    if session.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(session));
    }
}

#[no_mangle]
pub fn session_connect(session: *mut Session, credentials: *mut Credentials) {
    unsafe {
        let credentials_ref = credentials_ref(credentials);

        runtime().block_on(async {
            let session_ref = &mut *(*session).0;
            session_ref.connect(credentials_ref.clone(), false).await.unwrap();
        });

        credentials_free(credentials);
    }
}

#[repr(C)]
pub struct SpotifyId(*mut core::SpotifyId);

pub fn spotify_id_new_internal(spotify_id: core::SpotifyId) -> *mut SpotifyId {
    Box::into_raw(Box::new(
        SpotifyId(Box::into_raw(Box::new(
            spotify_id
        )))
    ))
}

#[no_mangle]
pub fn spotify_id_new() {}

#[no_mangle]
pub fn spotify_id_free(spotify_id: *mut SpotifyId) {
    if spotify_id.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(spotify_id));
    }
}
