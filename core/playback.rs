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

use crate::core::{Session, session_box, session_free};
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
            config::PlayerConfig::default()
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
    session_free(session);
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
pub fn player_get_event_channel(player: *mut Player) -> *mut PlayerChannel {
    unsafe {
        Box::into_raw(Box::new(
            PlayerChannel(Box::into_raw(Box::new(
                (*(*player).0).get_player_event_channel()
            )))
        ))
    }
}

#[repr(C)]
pub struct PlayerChannel(*mut UnboundedReceiver<player::PlayerEvent>);


#[repr(C)]
#[allow(dead_code)]
pub struct PlayerEvent;
