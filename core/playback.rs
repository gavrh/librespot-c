use librespot::playback::{
    config,
    mixer,
    player,
    audio_backend
};
use std::ffi::{CStr, c_char, c_uchar};
use std::sync::Arc;

use crate::core::{Session, session_box};

#[repr(C)]
pub struct MixerConfig {
    mixer_config: *mut mixer::MixerConfig,
}

#[no_mangle]
pub fn mixer_config_default() -> *mut MixerConfig {
    Box::into_raw(Box::new(
        MixerConfig {
            mixer_config: Box::into_raw(Box::new(
                mixer::MixerConfig::default()
            ))
        }
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
pub struct Mixer {
    mixer: *mut Arc<dyn mixer::Mixer>,
}

#[no_mangle]
pub fn mixer_new(mixer_config: *mut MixerConfig, mixer_name: *const c_char) -> *mut Mixer {

    let new_mixer: *mut Mixer;

    unsafe {
        new_mixer = Box::into_raw(Box::new(
            Mixer {
                mixer: Box::into_raw(Box::new(
                    mixer::find(Some(CStr::from_ptr(mixer_name).to_str().unwrap()))
                        .expect("Failed to find mixer.")
                        (*Box::from_raw((*mixer_config).mixer_config))
                ))
            }
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
        (*(*mixer).mixer).volume()
    }
}

#[repr(C)]
pub struct PlayerConfig {
    player_config: *mut config::PlayerConfig,
}

pub fn player_config_box(player_config: *mut PlayerConfig) -> Box<config::PlayerConfig> {
    unsafe {
        Box::from_raw((*player_config).player_config)
    }
}

#[no_mangle]
pub fn player_config_new(
    gapless: c_uchar,
    passthrough: c_uchar,
    normalisation: c_uchar
) -> *mut PlayerConfig {
    Box::into_raw(Box::new(
        PlayerConfig {
            player_config: Box::into_raw(Box::new(
                config::PlayerConfig {
                    bitrate: config::Bitrate::Bitrate320,
                    ditherer: None,
                    gapless: gapless != 0,
                    passthrough: passthrough != 0,
                    normalisation: normalisation != 0,
                    normalisation_attack_cf: 0.0,
                    normalisation_type: config::NormalisationType::Auto,
                    normalisation_knee_db: 0.0,
                    normalisation_method: config::NormalisationMethod::Basic,
                    normalisation_pregain_db: 0.0,
                    normalisation_release_cf: 0.0,
                    normalisation_threshold_dbfs: 0.0
                }
            ))
        }
    ))
}

#[no_mangle]
pub fn player_config_default() -> *mut PlayerConfig {
    Box::into_raw(Box::new(
        PlayerConfig {
            player_config: Box::into_raw(Box::new(
                config::PlayerConfig::default()
            ))
        }
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
pub struct Player {
    player: *mut Arc<player::Player>,
}

#[no_mangle]
pub fn player_new(player_config: *mut PlayerConfig, session: *mut Session, mixer: *mut Mixer, _audio_backend: *const c_char) -> *mut Player {

    let new_player: *mut Player;
    // let player_config_raw = player_config_raw(player_config);
    let sink_builder: audio_backend::SinkBuilder = move |device: Option<String>, format: config::AudioFormat| {
        audio_backend::find(
            Some("pulseaudio".to_string())
        ).expect("Failed to find audio backend.")(device, format)
    };

    unsafe {

        new_player = Box::into_raw(Box::new(
            Player {
                player: Box::into_raw(Box::new(
                    player::Player::new(
                        *player_config_box(player_config),
                        *session_box(session),
                        (*(*mixer).mixer).get_soft_volume(),
                        move || { sink_builder(None, config::AudioFormat::S16) }
                    )
                ))
            }
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
    if player.is_null() {
        return 0;
    }
    unsafe {
        !Box::from_raw((*player).player).is_invalid() as u8
    }
}
