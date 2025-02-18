use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use librespot::core::{Session, SessionConfig};
use librespot::playback::config::PlayerConfig;
use librespot::playback::mixer::VolumeGetter;
use librespot::playback::player::Player;

#[no_mangle]
pub extern "C" fn spotify_play(track_id: *const c_char) {
    let track_id = unsafe { CStr::from_ptr(track_id).to_string_lossy() };

    println!("Playing track: {}", track_id);
}

#[repr(C)]
pub struct CPlayer {
    player: Arc<Mutex<Player>>,
}

#[no_mangle]
pub extern "C" fn player_new() -> *mut CPlayer {
    let player_config = PlayerConfig::default();
    let session_config = SessionConfig::default();
    let session = Session::new(session_config, None);
    let player = Player::new(player_config, );
    let c_player = CPlayer {
        player: Arc::new(Mutex::new(player)),
    };
    Box::into_raw(Box::new(c_player))
}

#[no_mangle]
pub extern "C" fn player_free(player: *mut CPlayer) {
    if player.is_null() { return; }
    unsafe { drop(Box::from_raw(player)) };
}
