use librespot::playback::{
    config,
    mixer,
    player,
};
use std::os::raw::c_uchar;

#[repr(C)]
pub struct PlayerConfig {
    player_config: *mut config::PlayerConfig,
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
    player: *mut player::Player,
}

#[repr(C)]
pub struct Mixer {
    mixer: *mut dyn mixer::Mixer,
}
