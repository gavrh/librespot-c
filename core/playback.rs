use std::ffi::{c_char, c_uchar, CStr};
use std::os::raw::c_int;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;

use librespot::{
    core,
    playback::{audio_backend, config, mixer, player},
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::core::{
    session_clone, spotify_uri_free, spotify_uri_new_internal, Session, SpotifyUri,
};
use crate::error::{
    clear_error, set_error, LIBRESPOT_ERR_INVALID_ARGUMENT, LIBRESPOT_ERR_INVALID_STRING,
    LIBRESPOT_OK,
};
use crate::ffi::{ffi_guard, ffi_status};

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

fn parse_spotify_uri(spotify_uri: *const c_char) -> Result<core::SpotifyUri, c_int> {
    let spotify_uri = string_from_c(spotify_uri, "spotify_uri")?;
    let uri = core::SpotifyUri::from_uri(spotify_uri).map_err(|error| {
        set_error(format!("failed to parse Spotify URI: {error}"));
        LIBRESPOT_ERR_INVALID_ARGUMENT
    })?;

    if !uri.is_playable() {
        set_error(format!("Spotify URI is not playable: {spotify_uri}"));
        return Err(LIBRESPOT_ERR_INVALID_ARGUMENT);
    }

    Ok(uri)
}

#[repr(C)]
pub struct MixerConfig(mixer::MixerConfig);

#[no_mangle]
pub extern "C" fn mixer_config_default() -> *mut MixerConfig {
    ffi_guard(ptr::null_mut(), || {
        clear_error();
        Box::into_raw(Box::new(MixerConfig(mixer::MixerConfig::default())))
    })
}

#[no_mangle]
pub extern "C" fn mixer_config_free(mixer_config: *mut MixerConfig) {
    ffi_guard((), || {
        if mixer_config.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(mixer_config));
        }
    });
}

#[repr(C)]
pub struct Mixer(Arc<dyn mixer::Mixer>);

#[no_mangle]
pub extern "C" fn mixer_new(
    mixer_config: *mut MixerConfig,
    mixer_name: *const c_char,
) -> *mut Mixer {
    ffi_guard(ptr::null_mut(), || {
        clear_error();

        if mixer_config.is_null() {
            set_error("mixer_config must not be NULL");
            return ptr::null_mut();
        }

        let mixer_name = if mixer_name.is_null() {
            None
        } else {
            match string_from_c(mixer_name, "mixer_name") {
                Ok(mixer_name) => Some(mixer_name),
                Err(_) => return ptr::null_mut(),
            }
        };

        let mixer_fn = match mixer::find(mixer_name) {
            Some(mixer_fn) => mixer_fn,
            None => {
                set_error(format!(
                    "mixer not found: {}",
                    mixer_name.unwrap_or("default")
                ));
                return ptr::null_mut();
            }
        };

        let config = unsafe { (*mixer_config).0.clone() };
        match mixer_fn(config) {
            Ok(mixer) => Box::into_raw(Box::new(Mixer(mixer))),
            Err(error) => {
                set_error(format!("failed to open mixer: {error}"));
                ptr::null_mut()
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn mixer_free(mixer: *mut Mixer) {
    ffi_guard((), || {
        if mixer.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(mixer));
        }
    });
}

#[no_mangle]
pub extern "C" fn mixer_get_volume(mixer: *mut Mixer) -> u16 {
    ffi_guard(0, || {
        clear_error();

        if mixer.is_null() {
            set_error("mixer must not be NULL");
            return 0;
        }

        unsafe { (*mixer).0.volume() }
    })
}

#[no_mangle]
pub extern "C" fn mixer_set_volume(mixer: *mut Mixer, volume: u16) -> c_int {
    ffi_status(|| {
        clear_error();

        if mixer.is_null() {
            set_error("mixer must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*mixer).0.set_volume(volume);
        }

        LIBRESPOT_OK
    })
}

#[repr(C)]
pub struct PlayerConfig(config::PlayerConfig);

#[no_mangle]
pub extern "C" fn player_config_default() -> *mut PlayerConfig {
    ffi_guard(ptr::null_mut(), || {
        clear_error();

        let player_config = config::PlayerConfig {
            position_update_interval: Some(Duration::from_millis(500)),
            ..config::PlayerConfig::default()
        };

        Box::into_raw(Box::new(PlayerConfig(player_config)))
    })
}

#[no_mangle]
pub extern "C" fn player_config_free(player_config: *mut PlayerConfig) {
    ffi_guard((), || {
        if player_config.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(player_config));
        }
    });
}

#[repr(C)]
pub struct Player(player::Player);

#[no_mangle]
pub extern "C" fn player_new(
    player_config: *mut PlayerConfig,
    session: *mut Session,
    mixer: *mut Mixer,
    audio_backend: *const c_char,
) -> *mut Player {
    ffi_guard(ptr::null_mut(), || {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return ptr::null_mut();
        }
        if session.is_null() {
            set_error("session must not be NULL");
            return ptr::null_mut();
        }
        if mixer.is_null() {
            set_error("mixer must not be NULL");
            return ptr::null_mut();
        }

        let audio_backend_name = if audio_backend.is_null() {
            "rodio".to_owned()
        } else {
            match string_from_c(audio_backend, "audio_backend") {
                Ok(audio_backend) => audio_backend.to_owned(),
                Err(_) => return ptr::null_mut(),
            }
        };

        let sink_builder = match audio_backend::find(Some(audio_backend_name.clone())) {
            Some(sink_builder) => sink_builder,
            None => {
                set_error(format!("audio backend not found: {audio_backend_name}"));
                return ptr::null_mut();
            }
        };

        let config = unsafe { (*player_config).0.clone() };
        let session = match session_clone(session) {
            Some(session) => session,
            None => {
                set_error("session must not be NULL");
                return ptr::null_mut();
            }
        };
        let mixer = unsafe { (*mixer).0.clone() };

        let player = player::Player::new(
            config,
            session,
            mixer.get_soft_volume(),
            move || sink_builder(None, config::AudioFormat::S16),
        );

        let player = match Arc::try_unwrap(player) {
            Ok(player) => player,
            Err(_) => {
                set_error("failed to take ownership of player");
                return ptr::null_mut();
            }
        };

        Box::into_raw(Box::new(Player(player)))
    })
}

#[no_mangle]
pub extern "C" fn player_free(player: *mut Player) {
    ffi_guard((), || {
        if player.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(player));
        }
    });
}

#[no_mangle]
pub extern "C" fn player_is_valid(player: *mut Player) -> bool {
    ffi_guard(false, || {
        clear_error();

        if player.is_null() {
            set_error("player must not be NULL");
            return false;
        }

        unsafe { !(*player).0.is_invalid() }
    })
}

#[no_mangle]
pub extern "C" fn player_load(
    player: *mut Player,
    spotify_uri: *const c_char,
    start_playing: c_uchar,
    position_ms: u32,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player.is_null() {
            set_error("player must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let uri = match parse_spotify_uri(spotify_uri) {
            Ok(uri) => uri,
            Err(status) => return status,
        };

        unsafe {
            (*player).0.load(uri, start_playing != 0, position_ms);
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_preload(player: *mut Player, spotify_uri: *const c_char) -> c_int {
    ffi_status(|| {
        clear_error();

        if player.is_null() {
            set_error("player must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let uri = match parse_spotify_uri(spotify_uri) {
            Ok(uri) => uri,
            Err(status) => return status,
        };

        unsafe {
            (*player).0.preload(uri);
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_play(player: *mut Player) -> c_int {
    ffi_status(|| {
        clear_error();

        if player.is_null() {
            set_error("player must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player).0.play();
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_pause(player: *mut Player) -> c_int {
    ffi_status(|| {
        clear_error();

        if player.is_null() {
            set_error("player must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player).0.pause();
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_seek(player: *mut Player, position_ms: u32) -> c_int {
    ffi_status(|| {
        clear_error();

        if player.is_null() {
            set_error("player must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player).0.seek(position_ms);
        }

        LIBRESPOT_OK
    })
}

#[repr(C)]
pub struct PlayerChannel {
    receiver: UnboundedReceiver<player::PlayerEvent>,
    spotify_uris: Vec<*mut SpotifyUri>,
}

fn channel_uri(
    spotify_uris: &mut Vec<*mut SpotifyUri>,
    spotify_uri: core::SpotifyUri,
) -> Option<*mut SpotifyUri> {
    let spotify_uri = spotify_uri_new_internal(spotify_uri);
    if spotify_uri.is_null() {
        return None;
    }

    spotify_uris.push(spotify_uri);
    Some(spotify_uri)
}

#[no_mangle]
pub extern "C" fn player_channel_get(player: *mut Player) -> *mut PlayerChannel {
    ffi_guard(ptr::null_mut(), || {
        clear_error();

        if player.is_null() {
            set_error("player must not be NULL");
            return ptr::null_mut();
        }

        let receiver = unsafe { (*player).0.get_player_event_channel() };

        Box::into_raw(Box::new(PlayerChannel {
            receiver,
            spotify_uris: Vec::new(),
        }))
    })
}

#[no_mangle]
pub extern "C" fn player_channel_free(player_channel: *mut PlayerChannel) {
    ffi_guard((), || {
        if player_channel.is_null() {
            return;
        }

        unsafe {
            let player_channel = Box::from_raw(player_channel);

            for spotify_uri in player_channel.spotify_uris.iter().copied() {
                spotify_uri_free(spotify_uri);
            }

            drop(player_channel);
        }
    });
}

#[no_mangle]
pub extern "C" fn player_channel_poll(
    player_channel: *mut PlayerChannel,
    player_event: *mut PlayerEvent,
) -> bool {
    ffi_guard(false, || {
        clear_error();

        if player_channel.is_null() {
            set_error("player_channel must not be NULL");
            return false;
        }

        let player_channel = unsafe { &mut *player_channel };

        for spotify_uri in player_channel.spotify_uris.drain(..) {
            spotify_uri_free(spotify_uri);
        }

        if player_event.is_null() {
            set_error("player_event must not be NULL");
            return false;
        }

        loop {
            let event = match player_channel.receiver.try_recv() {
                Ok(event) => event,
                Err(_) => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_NONE;
                    }
                    return false;
                }
            };

            return match event {
                player::PlayerEvent::PlayRequestIdChanged { play_request_id } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_PLAY_REQUEST_ID_CHANGED;
                        (*player_event).data.play_request_id_changed =
                            PlayerEventPlayRequestIdChanged { play_request_id };
                    }
                    true
                }

                player::PlayerEvent::Stopped {
                    play_request_id,
                    track_id,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_STOPPED;
                        (*player_event).data.stopped = PlayerEventStopped {
                            play_request_id,
                            track_id,
                        };
                    }
                    true
                }

                player::PlayerEvent::Loading {
                    play_request_id,
                    track_id,
                    position_ms,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_LOADING;
                        (*player_event).data.loading = PlayerEventLoading {
                            play_request_id,
                            track_id,
                            position_ms,
                        };
                    }
                    true
                }

                player::PlayerEvent::Preloading { track_id } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_PRELOADING;
                        (*player_event).data.preloading = PlayerEventPreloading { track_id };
                    }
                    true
                }

                player::PlayerEvent::Playing {
                    play_request_id,
                    track_id,
                    position_ms,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_PLAYING;
                        (*player_event).data.playing = PlayerEventPlaying {
                            play_request_id,
                            track_id,
                            position_ms,
                        };
                    }
                    true
                }

                player::PlayerEvent::Paused {
                    play_request_id,
                    track_id,
                    position_ms,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_PAUSED;
                        (*player_event).data.paused = PlayerEventPaused {
                            play_request_id,
                            track_id,
                            position_ms,
                        };
                    }
                    true
                }

                player::PlayerEvent::TimeToPreloadNextTrack {
                    play_request_id,
                    track_id,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event =
                            PlayerEventType::PLAYER_EVENT_TIME_TO_PRELOAD_NEXT_TRACK;
                        (*player_event).data.time_to_preload_next_track =
                            PlayerEventTimeToPreloadNextTrack {
                                play_request_id,
                                track_id,
                            };
                    }
                    true
                }

                player::PlayerEvent::EndOfTrack {
                    play_request_id,
                    track_id,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_END_OF_TRACK;
                        (*player_event).data.end_of_track = PlayerEventEndOfTrack {
                            play_request_id,
                            track_id,
                        };
                    }
                    true
                }

                player::PlayerEvent::Unavailable {
                    play_request_id,
                    track_id,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_UNAVAILABLE;
                        (*player_event).data.unavailable = PlayerEventUnavailable {
                            play_request_id,
                            track_id,
                        };
                    }
                    true
                }

                player::PlayerEvent::VolumeChanged { volume } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_VOLUME_CHANGED;
                        (*player_event).data.volume_changed = PlayerEventVolumeChanged { volume };
                    }
                    true
                }

                player::PlayerEvent::PositionCorrection {
                    play_request_id,
                    track_id,
                    position_ms,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_POSITION_CORRECTION;
                        (*player_event).data.position_correction = PlayerEventPositionCorrection {
                            play_request_id,
                            track_id,
                            position_ms,
                        };
                    }
                    true
                }

                player::PlayerEvent::PositionChanged {
                    play_request_id,
                    track_id,
                    position_ms,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_POSITION_CHANGED;
                        (*player_event).data.position_changed = PlayerEventPositionChanged {
                            play_request_id,
                            track_id,
                            position_ms,
                        };
                    }
                    true
                }

                player::PlayerEvent::Seeked {
                    play_request_id,
                    track_id,
                    position_ms,
                } => {
                    let Some(track_id) = channel_uri(&mut player_channel.spotify_uris, track_id) else {
                        continue;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SEEKED;
                        (*player_event).data.seeked = PlayerEventSeeked {
                            play_request_id,
                            track_id,
                            position_ms,
                        };
                    }
                    true
                }

                player::PlayerEvent::TrackChanged { .. } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_TRACK_CHANGED;
                    }
                    true
                }

                player::PlayerEvent::SessionConnected { .. } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SESSION_CONNECTED;
                    }
                    true
                }

                player::PlayerEvent::SessionDisconnected { .. } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SESSION_DISCONNECTED;
                    }
                    true
                }

                player::PlayerEvent::SessionClientChanged { .. } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SESSION_CLIENT_CHANGED;
                    }
                    true
                }

                player::PlayerEvent::ShuffleChanged { .. } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SHUFFLE_CHANGED;
                    }
                    true
                }

                player::PlayerEvent::RepeatChanged { .. } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_REPEAT_CHANGED;
                    }
                    true
                }

                _ => continue,
            };
        }
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPlayRequestIdChanged {
    pub play_request_id: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventStopped {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventLoading {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
    pub position_ms: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPreloading {
    pub track_id: *mut SpotifyUri,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPlaying {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
    pub position_ms: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPaused {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
    pub position_ms: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventTimeToPreloadNextTrack {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventEndOfTrack {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventUnavailable {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventVolumeChanged {
    pub volume: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPositionCorrection {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
    pub position_ms: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPositionChanged {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
    pub position_ms: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventSeeked {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyUri,
    pub position_ms: u32,
}

#[repr(C)]
pub union PlayerEventData {
    pub play_request_id_changed: PlayerEventPlayRequestIdChanged,
    pub stopped: PlayerEventStopped,
    pub loading: PlayerEventLoading,
    pub preloading: PlayerEventPreloading,
    pub playing: PlayerEventPlaying,
    pub paused: PlayerEventPaused,
    pub time_to_preload_next_track: PlayerEventTimeToPreloadNextTrack,
    pub end_of_track: PlayerEventEndOfTrack,
    pub unavailable: PlayerEventUnavailable,
    pub volume_changed: PlayerEventVolumeChanged,
    pub position_correction: PlayerEventPositionCorrection,
    pub position_changed: PlayerEventPositionChanged,
    pub seeked: PlayerEventSeeked,
}

#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
pub enum PlayerEventType {
    PLAYER_EVENT_NONE,
    PLAYER_EVENT_PLAY_REQUEST_ID_CHANGED,
    PLAYER_EVENT_STOPPED,
    PLAYER_EVENT_LOADING,
    PLAYER_EVENT_PRELOADING,
    PLAYER_EVENT_PLAYING,
    PLAYER_EVENT_PAUSED,
    PLAYER_EVENT_TIME_TO_PRELOAD_NEXT_TRACK,
    PLAYER_EVENT_END_OF_TRACK,
    PLAYER_EVENT_UNAVAILABLE,
    PLAYER_EVENT_VOLUME_CHANGED,
    PLAYER_EVENT_POSITION_CORRECTION,
    PLAYER_EVENT_POSITION_CHANGED,
    PLAYER_EVENT_SEEKED,
    PLAYER_EVENT_TRACK_CHANGED,
    PLAYER_EVENT_SESSION_CONNECTED,
    PLAYER_EVENT_SESSION_DISCONNECTED,
    PLAYER_EVENT_SESSION_CLIENT_CHANGED,
    PLAYER_EVENT_SHUFFLE_CHANGED,
    PLAYER_EVENT_REPEAT_CHANGED,
}

#[repr(C)]
pub struct PlayerEvent {
    pub event: PlayerEventType,
    pub data: PlayerEventData,
}

#[no_mangle]
pub extern "C" fn player_event_new() -> *mut PlayerEvent {
    ffi_guard(ptr::null_mut(), || {
        clear_error();
        Box::into_raw(Box::new(PlayerEvent {
            event: PlayerEventType::PLAYER_EVENT_NONE,
            data: unsafe { std::mem::zeroed() },
        }))
    })
}

#[no_mangle]
pub extern "C" fn player_event_free(player_event: *mut PlayerEvent) {
    ffi_guard((), || {
        if player_event.is_null() {
            return;
        }

        unsafe {
            drop(Box::from_raw(player_event));
        }
    });
}
