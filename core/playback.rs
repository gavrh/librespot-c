use librespot::{
    core,
    playback::{
        config,
        mixer,
        player,
        audio_backend
    }
};
use tokio::sync::mpsc::UnboundedReceiver;
use std::ffi::{c_char, c_uchar, CStr};
use std::sync::Arc;
use std::time::Duration;

use crate::core::{session_box, spotify_id_new_internal, Session, SpotifyId};
use crate::runtime::runtime;

#[repr(C)]
pub struct MixerConfig(*mut mixer::MixerConfig);

#[no_mangle]
pub fn mixer_config_default() -> *mut MixerConfig {
    Box::into_raw(Box::new(
        MixerConfig(Box::into_raw(Box::new(
            mixer::MixerConfig::default()
        )))
    ))
}

#[no_mangle]
pub fn mixer_config_free(mixer_config: *mut MixerConfig) {
    if mixer_config.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(mixer_config));
    }
}

#[repr(C)]
pub struct Mixer(*mut Arc<dyn mixer::Mixer>);

#[no_mangle]
pub fn mixer_new(mixer_config: *mut MixerConfig, mixer_name: *const c_char) -> *mut Mixer {

    let new_mixer: *mut Mixer;

    unsafe {
        new_mixer = Box::into_raw(Box::new(
            Mixer(Box::into_raw(Box::new(
                mixer::find(Some(
                    CStr::from_ptr(mixer_name).to_str().unwrap()
                )).expect("Failed to find mixer.")(*Box::from_raw((*mixer_config).0))
            )))
        ));
    }

    mixer_config_free(mixer_config);
    return  new_mixer;
}

#[no_mangle]
pub fn mixer_free(mixer: *mut Mixer) {
    if mixer.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(mixer));
    }
}

#[no_mangle]
pub fn mixer_get_volume(mixer: *mut Mixer) -> u16 {
    unsafe {
        (*(*mixer).0).volume()
    }
}

#[no_mangle]
pub fn mixer_set_volume(mixer: *mut Mixer, volume: u16) {
    unsafe {
        (*(*mixer).0).set_volume(volume);
    }
}

#[repr(C)]
pub struct PlayerConfig(*mut config::PlayerConfig);

pub fn player_config_box(player_config: *mut PlayerConfig) -> Box<config::PlayerConfig> {
    unsafe {
        Box::from_raw((*player_config).0)
    }
}

#[no_mangle]
pub fn player_config_default() -> *mut PlayerConfig {
    Box::into_raw(Box::new(
        PlayerConfig(Box::into_raw(Box::new(
            config::PlayerConfig {
                bitrate: config::Bitrate::default(),
                gapless: true,
                normalisation: true,
                normalisation_type: config::NormalisationType::default(),
                normalisation_method: config::NormalisationMethod::default(),
                normalisation_pregain_db: 0.0,
                normalisation_threshold_dbfs: -2.0,
                normalisation_attack_cf: 0.005,
                normalisation_release_cf: 0.1,
                normalisation_knee_db: 5.0,
                passthrough: false,
                ditherer: Some(config::mk_ditherer::<config::TriangularDitherer>),
                position_update_interval: Some(Duration::from_millis(500))
            }
        )))
    ))
}

#[no_mangle]
pub fn player_config_free(player_config: *mut PlayerConfig) {
    if player_config.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(player_config));
    }
}

#[repr(C)]
pub struct Player(*mut Arc<player::Player>);

#[no_mangle]
pub fn player_new(player_config: *mut PlayerConfig, session: *mut Session, mixer: *mut Mixer, _audio_backend: *const c_char) -> *mut Player {

    let new_player: *mut Player;
    let sink_builder: audio_backend::SinkBuilder = move |device: Option<String>, format: config::AudioFormat| {
        audio_backend::find(
            Some("pulseaudio".to_string())
        ).expect("Failed to find audio backend.")(device, format)
    };

    unsafe {
        new_player = Box::into_raw(Box::new(
            Player(Box::into_raw(Box::new(
                player::Player::new(
                    *player_config_box(player_config),
                    *session_box(session),
                    (*(*mixer).0).get_soft_volume(),
                    move || { sink_builder(None, config::AudioFormat::S16) }
                )
            )))
        ));
    }

    player_config_free(player_config);
    return new_player;
}

#[no_mangle]
pub fn player_free(player: *mut Player) {
    if player.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(player));
    }
}

#[no_mangle]
pub fn player_is_valid(player: *mut Player) -> u8 {
    unsafe {
        !(*(*player).0).is_invalid() as u8
    }
}

#[no_mangle]
pub fn player_load(player: *mut Player, spotify_uri: *const c_char, start_playing: c_uchar, position_ms: u32) {
    unsafe {
        match core::SpotifyId::from_uri(CStr::from_ptr(spotify_uri).to_str().unwrap()) {
            Ok(id) => {
                if !id.is_playable() {
                    eprintln!("Track is not playable.");
                } else {
                    runtime().block_on(async {
                        // println!("loading {:#?}", spotify_uri);
                        (*(*player).0).load(id, start_playing != 0, position_ms);
                    });
                }
            },
            Err(e) => {
                eprintln!("Failed to load spotify uri: {}", e)
            }
        }
    }
}

#[no_mangle]
pub fn player_preload(player: *mut Player, spotify_uri: *const c_char) {
    unsafe {
        match core::SpotifyId::from_uri(CStr::from_ptr(spotify_uri).to_str().unwrap()) {
            Ok(id) => {
                if !id.is_playable() {
                    eprintln!("Track is not playable.");
                } else {
                    runtime().block_on(async {
                        // println!("preloading {:#?}", spotify_uri);
                        (*(*player).0).preload(id);
                    });
                }
            },
            Err(e) => {
                eprintln!("Failed to load spotify uri: {}", e)
            }
        }
    }
}

#[no_mangle]
pub fn player_play(player: *mut Player) {
    unsafe {
        (*(*player).0).play();
    }
}

#[no_mangle]
pub fn player_pause(player: *mut Player) {
    unsafe {
        (*(*player).0).pause();
    }
}

#[no_mangle]
pub fn player_seek(player: *mut Player, position_ms: u32) {
    unsafe {
        (*(*player).0).seek(position_ms);
    }
}

#[repr(C)]
pub struct PlayerChannel(*mut UnboundedReceiver<player::PlayerEvent>);

#[no_mangle]
pub fn player_channel_get(player: *mut Player) -> *mut PlayerChannel {
    unsafe {
        Box::into_raw(Box::new(
            PlayerChannel(Box::into_raw(Box::new(
                (*(*player).0).get_player_event_channel()
            )))
        ))
    }
}

#[no_mangle]
pub fn player_channel_free(player_channel: *mut PlayerChannel) {
    if player_channel.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(player_channel));
    }
}

#[no_mangle]
pub fn player_channel_poll(player_channel: *mut PlayerChannel, player_event: *mut PlayerEvent) -> u8 {
    unsafe {
        match (*(*player_channel).0).try_recv() {
            Ok(event) => {
                // println!("{:#?}", event);
                match event {

                    player::PlayerEvent::PlayRequestIdChanged {
                        play_request_id
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_PLAYER_REQUEST_ID_CHANGED;
                        (*player_event).data.play_request_id_changed = PlayerEventPlayRequestIdChanged {
                            play_request_id
                        };
                        return true as u8;
                    }

                    player::PlayerEvent::Stopped {
                        play_request_id,
                        track_id
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_STOPPED;
                        (*player_event).data.stopped = PlayerEventStopped {
                            play_request_id,
                            track_id: spotify_id_new_internal(track_id)
                        };
                        return true as u8;
                    }

                    player::PlayerEvent::Loading {
                        play_request_id,
                        track_id,
                        position_ms
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_LOADING;
                        (*player_event).data.loading = PlayerEventLoading {
                            play_request_id,
                            track_id: spotify_id_new_internal(track_id),
                            position_ms
                        };
                        return true as u8;
                    }

                    player::PlayerEvent::Preloading {
                        track_id
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_PRELOADING;
                        (*player_event).data.preloading = PlayerEventPreloading {
                            track_id: spotify_id_new_internal(track_id)
                        };
                        return true as u8;
                    }

                    player::PlayerEvent::Playing {
                        play_request_id,
                        track_id,
                        position_ms
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_PLAYING;
                        (*player_event).data.playing = PlayerEventPlaying {
                            play_request_id,
                            track_id: spotify_id_new_internal(track_id),
                            position_ms
                        };
                        return true as u8;
                    }

                    player::PlayerEvent::Paused {
                        play_request_id,
                        track_id,
                        position_ms
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_PAUSED;
                        (*player_event).data.paused = PlayerEventPaused {
                            play_request_id,
                            track_id: spotify_id_new_internal(track_id),
                            position_ms
                        };
                        return true as u8;
                    }

                    player::PlayerEvent::EndOfTrack {
                        play_request_id,
                        track_id
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_END_OF_TRACK;
                        (*player_event).data.end_of_track = PlayerEventEndOfTrack {
                            play_request_id, 
                            track_id: spotify_id_new_internal(track_id)
                        };
                        return true as u8;
                    }

                    player::PlayerEvent::TimeToPreloadNextTrack {
                        play_request_id,
                        track_id
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_TIME_TO_PRELOAD_NEXT_TRACK;
                        (*player_event).data.time_to_preload_next_track = PlayerEventTimeToPreloadNextTrack {
                            play_request_id, 
                            track_id: spotify_id_new_internal(track_id)
                        };
                        return true as u8;
                    }

                    player::PlayerEvent::PositionChanged {
                        play_request_id,
                        track_id,
                        position_ms
                    } => {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_POSITION_CHANGED;
                        (*player_event).data.positon_changed = PlayerEventPositionChanged {
                            play_request_id,
                            track_id: spotify_id_new_internal(track_id),
                            position_ms
                        };
                        return true as u8;
                    }

                    _ => {
                        return false as u8;
                    }
                }
            },
            Err(_) => {
                (*player_event).event = PlayerEventType::PLAYER_EVENT_NONE;
                return false as u8;
            }
        }
    }   
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPlayRequestIdChanged {
    pub play_request_id: u64
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventStopped {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventLoading {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId,
    pub position_ms: u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPreloading {
    pub track_id: *mut SpotifyId
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPlaying {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId,
    pub position_ms: u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPaused {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId,
    pub position_ms: u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventTimeToPreloadNextTrack {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventEndOfTrack {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventUnavailable {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventVolumeChanged {
    pub volume: u64
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPositionCorrection {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId,
    pub position_ms: u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventPositionChanged {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId,
    pub position_ms: u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventSeeked {
    pub play_request_id: u64,
    pub track_id: *mut SpotifyId,
    pub position_ms: u32
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
    pub positon_correction: PlayerEventPositionCorrection,
    pub positon_changed: PlayerEventPositionChanged,
    pub seeked: PlayerEventSeeked
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum PlayerEventType {
    PLAYER_EVENT_NONE,
    PLAYER_EVENT_PLAYER_REQUEST_ID_CHANGED,
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
    PLAYER_EVENT_REPEAT_CHANGED
}

#[repr(C)]
pub struct PlayerEvent {
    pub event: PlayerEventType,
    pub data: PlayerEventData
}

#[no_mangle]
pub fn player_event_new() -> *mut PlayerEvent {
    let new_player_event: *mut PlayerEvent;

    unsafe {
        new_player_event = Box::into_raw(Box::new(
            PlayerEvent {
                event: PlayerEventType::PLAYER_EVENT_NONE,
                data: std::mem::zeroed()
            }
        ));
    }

    return new_player_event;
}

#[no_mangle]
pub fn player_event_free(player_event: *mut PlayerEvent) {
    if player_event.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(player_event));
    }
}
