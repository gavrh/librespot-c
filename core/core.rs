use librespot::core;
use crate::discovery::Credentials;
use tokio::runtime::Runtime;

#[repr(C)]
pub struct SessionConfig {
    session_config: *mut core::SessionConfig
}

#[no_mangle]
pub fn session_config_default() -> *mut SessionConfig {
    Box::into_raw(Box::new(
        SessionConfig {
            session_config: Box::into_raw(Box::new(
                core::SessionConfig::default()
            ))
        }
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
pub struct Session {
    session: *mut core::Session,
}

#[no_mangle]
pub fn session_new(session_config: *mut SessionConfig) -> *mut Session {
    unsafe {
        let rt = Runtime::new().expect("Failed to create separate runtime.");
        let new_session = Box::into_raw(Box::new(
            Session {
                session: Box::into_raw(Box::new(
                    rt.block_on(async {
                        core::Session::new(
                            *Box::from_raw((*session_config).session_config),
                            None
                        )
                    })
                ))
            }
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
        let rt = Runtime::new().expect("Failed to create spearate runtime.");
        let box_session = Box::from_raw((*session).session);
        let raw_credentials = *Box::from_raw((*credentials).credentials);
        rt.block_on(async {
            box_session.connect(raw_credentials, false).await.unwrap();
        })
    }
}
