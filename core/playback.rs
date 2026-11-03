use std::ffi::{c_char, c_uchar, CStr, CString};
use std::os::raw::c_int;
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;

use librespot::{
    core,
    metadata::{
        audio::{AudioFileFormat, AudioItem, UniqueFields},
        artist::ArtistRole,
        availability::UnavailabilityReason,
        image::ImageSize,
    },
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

fn ensure_finite(value: f64, name: &str) -> Result<(), c_int> {
    if !value.is_finite() {
        set_error(format!("{name} must be finite"));
        return Err(LIBRESPOT_ERR_INVALID_ARGUMENT);
    }

    Ok(())
}

fn ensure_positive(value: f64, name: &str) -> Result<(), c_int> {
    if !value.is_finite() || value <= 0.0 {
        set_error(format!("{name} must be finite and greater than zero"));
        return Err(LIBRESPOT_ERR_INVALID_ARGUMENT);
    }

    Ok(())
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

#[no_mangle]
pub extern "C" fn player_config_set_bitrate(
    player_config: *mut PlayerConfig,
    bitrate: c_int,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let bitrate = match bitrate {
            96 => config::Bitrate::Bitrate96,
            160 => config::Bitrate::Bitrate160,
            320 => config::Bitrate::Bitrate320,
            _ => {
                set_error(format!("invalid player bitrate: {bitrate}"));
                return LIBRESPOT_ERR_INVALID_ARGUMENT;
            }
        };

        unsafe {
            (*player_config).0.bitrate = bitrate;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_gapless(
    player_config: *mut PlayerConfig,
    gapless: bool,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player_config).0.gapless = gapless;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_passthrough(
    player_config: *mut PlayerConfig,
    passthrough: bool,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player_config).0.passthrough = passthrough;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_normalisation(
    player_config: *mut PlayerConfig,
    normalisation: bool,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player_config).0.normalisation = normalisation;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_normalisation_type(
    player_config: *mut PlayerConfig,
    normalisation_type: c_int,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let normalisation_type = match normalisation_type {
            0 => config::NormalisationType::Album,
            1 => config::NormalisationType::Track,
            2 => config::NormalisationType::Auto,
            _ => {
                set_error(format!(
                    "invalid player normalisation type: {normalisation_type}"
                ));
                return LIBRESPOT_ERR_INVALID_ARGUMENT;
            }
        };

        unsafe {
            (*player_config).0.normalisation_type = normalisation_type;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_normalisation_method(
    player_config: *mut PlayerConfig,
    normalisation_method: c_int,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let normalisation_method = match normalisation_method {
            0 => config::NormalisationMethod::Basic,
            1 => config::NormalisationMethod::Dynamic,
            _ => {
                set_error(format!(
                    "invalid player normalisation method: {normalisation_method}"
                ));
                return LIBRESPOT_ERR_INVALID_ARGUMENT;
            }
        };

        unsafe {
            (*player_config).0.normalisation_method = normalisation_method;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_normalisation_pregain_db(
    player_config: *mut PlayerConfig,
    value: f64,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        if let Err(status) = ensure_finite(value, "normalisation_pregain_db") {
            return status;
        }

        unsafe {
            (*player_config).0.normalisation_pregain_db = value;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_normalisation_threshold_dbfs(
    player_config: *mut PlayerConfig,
    value: f64,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        if let Err(status) = ensure_finite(value, "normalisation_threshold_dbfs") {
            return status;
        }

        unsafe {
            (*player_config).0.normalisation_threshold_dbfs = value;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_normalisation_attack_cf(
    player_config: *mut PlayerConfig,
    value: f64,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        if let Err(status) = ensure_positive(value, "normalisation_attack_cf") {
            return status;
        }

        unsafe {
            (*player_config).0.normalisation_attack_cf = value;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_normalisation_release_cf(
    player_config: *mut PlayerConfig,
    value: f64,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        if let Err(status) = ensure_positive(value, "normalisation_release_cf") {
            return status;
        }

        unsafe {
            (*player_config).0.normalisation_release_cf = value;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_normalisation_knee_db(
    player_config: *mut PlayerConfig,
    value: f64,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        if let Err(status) = ensure_finite(value, "normalisation_knee_db") {
            return status;
        }

        unsafe {
            (*player_config).0.normalisation_knee_db = value;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_position_update_interval(
    player_config: *mut PlayerConfig,
    interval_ms: u32,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player_config).0.position_update_interval =
                Some(Duration::from_millis(interval_ms as u64));
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_clear_position_update_interval(
    player_config: *mut PlayerConfig,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player_config).0.position_update_interval = None;
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_add_local_file_directory(
    player_config: *mut PlayerConfig,
    path: *const c_char,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let path = match string_from_c(path, "path") {
            Ok(path) => path,
            Err(status) => return status,
        };

        if path.is_empty() {
            set_error("path must not be empty");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player_config)
                .0
                .local_file_directories
                .push(PathBuf::from(path));
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_clear_local_file_directories(
    player_config: *mut PlayerConfig,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        unsafe {
            (*player_config).0.local_file_directories.clear();
        }

        LIBRESPOT_OK
    })
}

#[no_mangle]
pub extern "C" fn player_config_set_ditherer(
    player_config: *mut PlayerConfig,
    ditherer: c_int,
) -> c_int {
    ffi_status(|| {
        clear_error();

        if player_config.is_null() {
            set_error("player_config must not be NULL");
            return LIBRESPOT_ERR_INVALID_ARGUMENT;
        }

        let ditherer: Option<config::DithererBuilder> = match ditherer {
            0 => None,
            1 => Some(config::mk_ditherer::<config::TriangularDitherer>),
            _ => {
                set_error(format!("invalid player ditherer: {ditherer}"));
                return LIBRESPOT_ERR_INVALID_ARGUMENT;
            }
        };

        unsafe {
            (*player_config).0.ditherer = ditherer;
        }

        LIBRESPOT_OK
    })
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
    strings: Vec<CString>,
    cover_images: Vec<PlayerCoverImage>,
    artists: Vec<PlayerArtist>,
    audio_files: Vec<PlayerAudioFile>,
    alternatives: Vec<*mut SpotifyUri>,
    album_artists: Vec<*const c_char>,
    language: Vec<*const c_char>,
    next_tracks: Vec<PlayerQueueTrack>,
    previous_tracks: Vec<PlayerQueueTrack>,
}

fn vec_ptr<T>(values: &mut Vec<T>) -> *mut T {
    if values.is_empty() {
        ptr::null_mut()
    } else {
        values.as_mut_ptr()
    }
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

fn channel_cstring(strings: &mut Vec<CString>, value: String) -> Option<*const c_char> {
    let value = CString::new(value).ok()?;
    let pointer = value.as_ptr();
    strings.push(value);
    Some(pointer)
}

fn channel_queue_track(
    strings: &mut Vec<CString>,
    queue_track: player::QueueTrack,
) -> Option<PlayerQueueTrack> {
    Some(PlayerQueueTrack {
        uri: channel_cstring(strings, queue_track.uri)?,
        provider: channel_cstring(strings, queue_track.provider)?,
    })
}

fn player_image_size(size: ImageSize) -> PlayerImageSize {
    match size {
        ImageSize::DEFAULT => PlayerImageSize::DEFAULT,
        ImageSize::SMALL => PlayerImageSize::SMALL,
        ImageSize::LARGE => PlayerImageSize::LARGE,
        ImageSize::XLARGE => PlayerImageSize::XLARGE,
    }
}

fn player_artist_role(role: ArtistRole) -> PlayerArtistRole {
    match role {
        ArtistRole::ARTIST_ROLE_UNKNOWN => PlayerArtistRole::ARTIST_ROLE_UNKNOWN,
        ArtistRole::ARTIST_ROLE_MAIN_ARTIST => PlayerArtistRole::ARTIST_ROLE_MAIN_ARTIST,
        ArtistRole::ARTIST_ROLE_FEATURED_ARTIST => PlayerArtistRole::ARTIST_ROLE_FEATURED_ARTIST,
        ArtistRole::ARTIST_ROLE_REMIXER => PlayerArtistRole::ARTIST_ROLE_REMIXER,
        ArtistRole::ARTIST_ROLE_ACTOR => PlayerArtistRole::ARTIST_ROLE_ACTOR,
        ArtistRole::ARTIST_ROLE_COMPOSER => PlayerArtistRole::ARTIST_ROLE_COMPOSER,
        ArtistRole::ARTIST_ROLE_CONDUCTOR => PlayerArtistRole::ARTIST_ROLE_CONDUCTOR,
        ArtistRole::ARTIST_ROLE_ORCHESTRA => PlayerArtistRole::ARTIST_ROLE_ORCHESTRA,
    }
}

fn player_audio_file_format(format: AudioFileFormat) -> PlayerAudioFileFormat {
    match format {
        AudioFileFormat::OGG_VORBIS_96 => PlayerAudioFileFormat::OGG_VORBIS_96,
        AudioFileFormat::OGG_VORBIS_160 => PlayerAudioFileFormat::OGG_VORBIS_160,
        AudioFileFormat::OGG_VORBIS_320 => PlayerAudioFileFormat::OGG_VORBIS_320,
        AudioFileFormat::MP3_256 => PlayerAudioFileFormat::MP3_256,
        AudioFileFormat::MP3_320 => PlayerAudioFileFormat::MP3_320,
        AudioFileFormat::MP3_160 => PlayerAudioFileFormat::MP3_160,
        AudioFileFormat::MP3_96 => PlayerAudioFileFormat::MP3_96,
        AudioFileFormat::MP3_160_ENC => PlayerAudioFileFormat::MP3_160_ENC,
        AudioFileFormat::AAC_24 => PlayerAudioFileFormat::AAC_24,
        AudioFileFormat::AAC_48 => PlayerAudioFileFormat::AAC_48,
        AudioFileFormat::FLAC_FLAC => PlayerAudioFileFormat::FLAC_FLAC,
        AudioFileFormat::XHE_AAC_24 => PlayerAudioFileFormat::XHE_AAC_24,
        AudioFileFormat::XHE_AAC_16 => PlayerAudioFileFormat::XHE_AAC_16,
        AudioFileFormat::XHE_AAC_12 => PlayerAudioFileFormat::XHE_AAC_12,
        AudioFileFormat::FLAC_FLAC_24BIT => PlayerAudioFileFormat::FLAC_FLAC_24BIT,
        AudioFileFormat::AAC_160 => PlayerAudioFileFormat::AAC_160,
        AudioFileFormat::AAC_320 => PlayerAudioFileFormat::AAC_320,
        AudioFileFormat::MP4_128 => PlayerAudioFileFormat::MP4_128,
        AudioFileFormat::OTHER5 => PlayerAudioFileFormat::OTHER5,
    }
}

fn player_unavailability_reason(
    availability: Result<(), UnavailabilityReason>,
) -> PlayerUnavailabilityReason {
    match availability {
        Ok(()) => PlayerUnavailabilityReason::NONE,
        Err(UnavailabilityReason::Blacklisted) => PlayerUnavailabilityReason::BLACKLISTED,
        Err(UnavailabilityReason::Embargo) => PlayerUnavailabilityReason::EMBARGO,
        Err(UnavailabilityReason::NoData) => PlayerUnavailabilityReason::NO_DATA,
        Err(UnavailabilityReason::NotWhitelisted) => PlayerUnavailabilityReason::NOT_WHITELISTED,
    }
}

fn channel_unique_fields(
    channel: &mut PlayerChannel,
    unique_fields: UniqueFields,
) -> Option<PlayerUniqueFields> {
    match unique_fields {
        UniqueFields::Track {
            artists,
            album,
            album_artists,
            popularity,
            number,
            disc_number,
        } => {
            let album = channel_cstring(&mut channel.strings, album)?;

            for artist in artists.0 {
                let id = channel_uri(&mut channel.spotify_uris, artist.id)?;
                let name = channel_cstring(&mut channel.strings, artist.name)?;
                let role = player_artist_role(artist.role);
                channel.artists.push(PlayerArtist { id, name, role });
            }

            for album_artist in album_artists {
                let album_artist = channel_cstring(&mut channel.strings, album_artist)?;
                channel.album_artists.push(album_artist);
            }

            let artists = vec_ptr(&mut channel.artists);
            let artists_count = channel.artists.len();
            let album_artists = vec_ptr(&mut channel.album_artists);
            let album_artists_count = channel.album_artists.len();

            Some(PlayerUniqueFields {
                kind: PlayerUniqueFieldsKind::TRACK,
                data: PlayerUniqueFieldsData {
                    track: PlayerTrackFields {
                        album,
                        artists,
                        artists_count,
                        album_artists,
                        album_artists_count,
                        popularity,
                        number,
                        disc_number,
                    },
                },
            })
        }
        UniqueFields::Local {
            artists,
            album,
            album_artists,
            number,
            disc_number,
            path,
        } => {
            let artists = match artists {
                Some(artists) => channel_cstring(&mut channel.strings, artists)?,
                None => ptr::null(),
            };
            let album = match album {
                Some(album) => channel_cstring(&mut channel.strings, album)?,
                None => ptr::null(),
            };
            let album_artists = match album_artists {
                Some(album_artists) => channel_cstring(&mut channel.strings, album_artists)?,
                None => ptr::null(),
            };
            let (has_number, number) = match number {
                Some(number) => (true, number),
                None => (false, 0),
            };
            let (has_disc_number, disc_number) = match disc_number {
                Some(disc_number) => (true, disc_number),
                None => (false, 0),
            };
            let path = channel_cstring(&mut channel.strings, path.to_string_lossy().into_owned())?;

            Some(PlayerUniqueFields {
                kind: PlayerUniqueFieldsKind::LOCAL,
                data: PlayerUniqueFieldsData {
                    local: PlayerLocalFields {
                        artists,
                        album,
                        album_artists,
                        has_number,
                        number,
                        has_disc_number,
                        disc_number,
                        path,
                    },
                },
            })
        }
        UniqueFields::Episode {
            description,
            publish_time,
            show_name,
        } => {
            let description = channel_cstring(&mut channel.strings, description)?;
            let publish_time_ms = publish_time.as_timestamp_ms();
            let show_name = channel_cstring(&mut channel.strings, show_name)?;

            Some(PlayerUniqueFields {
                kind: PlayerUniqueFieldsKind::EPISODE,
                data: PlayerUniqueFieldsData {
                    episode: PlayerEpisodeFields {
                        description,
                        publish_time_ms,
                        show_name,
                    },
                },
            })
        }
    }
}

fn channel_audio_item(channel: &mut PlayerChannel, audio_item: AudioItem) -> Option<PlayerAudioItem> {
    let track_id = channel_uri(&mut channel.spotify_uris, audio_item.track_id)?;
    let uri = channel_cstring(&mut channel.strings, audio_item.uri)?;
    let name = channel_cstring(&mut channel.strings, audio_item.name)?;

    for cover in audio_item.covers {
        let url = channel_cstring(&mut channel.strings, cover.url)?;
        channel.cover_images.push(PlayerCoverImage {
            url,
            size: player_image_size(cover.size),
            width: cover.width,
            height: cover.height,
        });
    }

    for language in audio_item.language {
        let language = channel_cstring(&mut channel.strings, language)?;
        channel.language.push(language);
    }

    for (format, file_id) in audio_item.files.0 {
        channel.audio_files.push(PlayerAudioFile {
            format: player_audio_file_format(format),
            file_id: file_id.0,
        });
    }

    if let Some(alternatives) = audio_item.alternatives {
        for alternative in alternatives.0 {
            let alternative = channel_uri(&mut channel.spotify_uris, alternative)?;
            channel.alternatives.push(alternative);
        }
    }

    let availability = player_unavailability_reason(audio_item.availability);
    let unique_fields = channel_unique_fields(channel, audio_item.unique_fields)?;

    let covers = vec_ptr(&mut channel.cover_images);
    let covers_count = channel.cover_images.len();
    let language = vec_ptr(&mut channel.language);
    let language_count = channel.language.len();
    let files = vec_ptr(&mut channel.audio_files);
    let files_count = channel.audio_files.len();
    let alternatives = vec_ptr(&mut channel.alternatives);
    let alternatives_count = channel.alternatives.len();

    Some(PlayerAudioItem {
        track_id,
        uri,
        name,
        duration_ms: audio_item.duration_ms,
        is_explicit: audio_item.is_explicit,
        covers,
        covers_count,
        language,
        language_count,
        files,
        files_count,
        alternatives,
        alternatives_count,
        availability,
        unique_fields,
    })
}

fn channel_clear(channel: &mut PlayerChannel) {
    for spotify_uri in channel.spotify_uris.drain(..) {
        spotify_uri_free(spotify_uri);
    }

    channel.strings.clear();
    channel.cover_images.clear();
    channel.artists.clear();
    channel.audio_files.clear();
    channel.alternatives.clear();
    channel.album_artists.clear();
    channel.language.clear();
    channel.next_tracks.clear();
    channel.previous_tracks.clear();
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
            strings: Vec::new(),
            cover_images: Vec::new(),
            artists: Vec::new(),
            audio_files: Vec::new(),
            alternatives: Vec::new(),
            album_artists: Vec::new(),
            language: Vec::new(),
            next_tracks: Vec::new(),
            previous_tracks: Vec::new(),
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
            let mut player_channel = Box::from_raw(player_channel);
            channel_clear(&mut player_channel);
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
        channel_clear(player_channel);

        if player_event.is_null() {
            set_error("player_event must not be NULL");
            return false;
        }

        'events: loop {
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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
                    let Some(track_id) =
                        channel_uri(&mut player_channel.spotify_uris, track_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
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

                player::PlayerEvent::TrackChanged { audio_item } => {
                    let Some(audio_item) = channel_audio_item(player_channel, *audio_item) else {
                        channel_clear(player_channel);
                        continue 'events;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_TRACK_CHANGED;
                        (*player_event).data.track_changed = PlayerEventTrackChanged { audio_item };
                    }
                    true
                }

                player::PlayerEvent::SessionConnected {
                    connection_id,
                    user_name,
                } => {
                    let Some(connection_id) =
                        channel_cstring(&mut player_channel.strings, connection_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };
                    let Some(user_name) =
                        channel_cstring(&mut player_channel.strings, user_name)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SESSION_CONNECTED;
                        (*player_event).data.session_connected = PlayerEventSessionConnected {
                            connection_id,
                            user_name,
                        };
                    }
                    true
                }

                player::PlayerEvent::SessionDisconnected {
                    connection_id,
                    user_name,
                } => {
                    let Some(connection_id) =
                        channel_cstring(&mut player_channel.strings, connection_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };
                    let Some(user_name) =
                        channel_cstring(&mut player_channel.strings, user_name)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SESSION_DISCONNECTED;
                        (*player_event).data.session_disconnected = PlayerEventSessionDisconnected {
                            connection_id,
                            user_name,
                        };
                    }
                    true
                }

                player::PlayerEvent::SessionClientChanged {
                    client_id,
                    client_name,
                    client_brand_name,
                    client_model_name,
                } => {
                    let Some(client_id) =
                        channel_cstring(&mut player_channel.strings, client_id)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };
                    let Some(client_name) =
                        channel_cstring(&mut player_channel.strings, client_name)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };
                    let Some(client_brand_name) =
                        channel_cstring(&mut player_channel.strings, client_brand_name)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };
                    let Some(client_model_name) =
                        channel_cstring(&mut player_channel.strings, client_model_name)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SESSION_CLIENT_CHANGED;
                        (*player_event).data.session_client_changed =
                            PlayerEventSessionClientChanged {
                                client_id,
                                client_name,
                                client_brand_name,
                                client_model_name,
                            };
                    }
                    true
                }

                player::PlayerEvent::ShuffleChanged { shuffle } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SHUFFLE_CHANGED;
                        (*player_event).data.shuffle_changed = PlayerEventShuffleChanged { shuffle };
                    }
                    true
                }

                player::PlayerEvent::RepeatChanged { context, track } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_REPEAT_CHANGED;
                        (*player_event).data.repeat_changed = PlayerEventRepeatChanged {
                            context,
                            track,
                        };
                    }
                    true
                }

                player::PlayerEvent::AutoPlayChanged { auto_play } => {
                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_AUTO_PLAY_CHANGED;
                        (*player_event).data.auto_play_changed = PlayerEventAutoPlayChanged {
                            auto_play,
                        };
                    }
                    true
                }

                player::PlayerEvent::FilterExplicitContentChanged { filter } => {
                    unsafe {
                        (*player_event).event =
                            PlayerEventType::PLAYER_EVENT_FILTER_EXPLICIT_CONTENT_CHANGED;
                        (*player_event).data.filter_explicit_content_changed =
                            PlayerEventFilterExplicitContentChanged { filter };
                    }
                    true
                }

                player::PlayerEvent::SetQueue {
                    context_uri,
                    current_track,
                    next_tracks,
                    prev_tracks,
                } => {
                    let Some(context_uri) =
                        channel_cstring(&mut player_channel.strings, context_uri)
                    else {
                        channel_clear(player_channel);
                        continue 'events;
                    };
                    let (has_current_track, current_track) = match current_track {
                        Some(current_track) => {
                            let Some(current_track) =
                                channel_queue_track(&mut player_channel.strings, current_track)
                            else {
                                channel_clear(player_channel);
                                continue 'events;
                            };
                            (true, current_track)
                        }
                        None => (
                            false,
                            PlayerQueueTrack {
                                uri: ptr::null(),
                                provider: ptr::null(),
                            },
                        ),
                    };

                    for next_track in next_tracks {
                        let Some(next_track) =
                            channel_queue_track(&mut player_channel.strings, next_track)
                        else {
                            channel_clear(player_channel);
                            continue 'events;
                        };
                        player_channel.next_tracks.push(next_track);
                    }

                    for previous_track in prev_tracks {
                        let Some(previous_track) =
                            channel_queue_track(&mut player_channel.strings, previous_track)
                        else {
                            channel_clear(player_channel);
                            continue 'events;
                        };
                        player_channel.previous_tracks.push(previous_track);
                    }

                    let next_tracks = vec_ptr(&mut player_channel.next_tracks);
                    let next_tracks_count = player_channel.next_tracks.len();
                    let previous_tracks = vec_ptr(&mut player_channel.previous_tracks);
                    let previous_tracks_count = player_channel.previous_tracks.len();

                    unsafe {
                        (*player_event).event = PlayerEventType::PLAYER_EVENT_SET_QUEUE;
                        (*player_event).data.set_queue = PlayerEventSetQueue {
                            context_uri,
                            has_current_track,
                            current_track,
                            next_tracks,
                            next_tracks_count,
                            previous_tracks,
                            previous_tracks_count,
                        };
                    }
                    true
                }
            };
        }
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum PlayerImageSize {
    DEFAULT = 0,
    SMALL = 1,
    LARGE = 2,
    XLARGE = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerCoverImage {
    pub url: *const c_char,
    pub size: PlayerImageSize,
    pub width: i32,
    pub height: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum PlayerArtistRole {
    ARTIST_ROLE_UNKNOWN = 0,
    ARTIST_ROLE_MAIN_ARTIST = 1,
    ARTIST_ROLE_FEATURED_ARTIST = 2,
    ARTIST_ROLE_REMIXER = 3,
    ARTIST_ROLE_ACTOR = 4,
    ARTIST_ROLE_COMPOSER = 5,
    ARTIST_ROLE_CONDUCTOR = 6,
    ARTIST_ROLE_ORCHESTRA = 7,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerArtist {
    pub id: *mut SpotifyUri,
    pub name: *const c_char,
    pub role: PlayerArtistRole,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum PlayerAudioFileFormat {
    OGG_VORBIS_96 = 0,
    OGG_VORBIS_160 = 1,
    OGG_VORBIS_320 = 2,
    MP3_256 = 3,
    MP3_320 = 4,
    MP3_160 = 5,
    MP3_96 = 6,
    MP3_160_ENC = 7,
    AAC_24 = 8,
    AAC_48 = 9,
    FLAC_FLAC = 10,
    XHE_AAC_24 = 11,
    XHE_AAC_16 = 12,
    XHE_AAC_12 = 13,
    FLAC_FLAC_24BIT = 14,
    AAC_160 = 15,
    AAC_320 = 16,
    MP4_128 = 17,
    OTHER5 = 18,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerAudioFile {
    pub format: PlayerAudioFileFormat,
    pub file_id: [u8; 20],
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum PlayerUnavailabilityReason {
    NONE = -1,
    BLACKLISTED = 0,
    EMBARGO = 1,
    NO_DATA = 2,
    NOT_WHITELISTED = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum PlayerUniqueFieldsKind {
    TRACK = 0,
    LOCAL = 1,
    EPISODE = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerTrackFields {
    pub album: *const c_char,
    pub artists: *mut PlayerArtist,
    pub artists_count: usize,
    pub album_artists: *mut *const c_char,
    pub album_artists_count: usize,
    pub popularity: u8,
    pub number: u32,
    pub disc_number: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerLocalFields {
    pub artists: *const c_char,
    pub album: *const c_char,
    pub album_artists: *const c_char,
    pub has_number: bool,
    pub number: u32,
    pub has_disc_number: bool,
    pub disc_number: u32,
    pub path: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEpisodeFields {
    pub description: *const c_char,
    pub publish_time_ms: i64,
    pub show_name: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union PlayerUniqueFieldsData {
    pub track: PlayerTrackFields,
    pub local: PlayerLocalFields,
    pub episode: PlayerEpisodeFields,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerUniqueFields {
    pub kind: PlayerUniqueFieldsKind,
    pub data: PlayerUniqueFieldsData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerAudioItem {
    pub track_id: *mut SpotifyUri,
    pub uri: *const c_char,
    pub name: *const c_char,
    pub duration_ms: u32,
    pub is_explicit: bool,
    pub covers: *mut PlayerCoverImage,
    pub covers_count: usize,
    pub language: *mut *const c_char,
    pub language_count: usize,
    pub files: *mut PlayerAudioFile,
    pub files_count: usize,
    pub alternatives: *mut *mut SpotifyUri,
    pub alternatives_count: usize,
    pub availability: PlayerUnavailabilityReason,
    pub unique_fields: PlayerUniqueFields,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerQueueTrack {
    pub uri: *const c_char,
    pub provider: *const c_char,
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
#[derive(Clone, Copy)]
pub struct PlayerEventTrackChanged {
    pub audio_item: PlayerAudioItem,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventSessionConnected {
    pub connection_id: *const c_char,
    pub user_name: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventSessionDisconnected {
    pub connection_id: *const c_char,
    pub user_name: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventSessionClientChanged {
    pub client_id: *const c_char,
    pub client_name: *const c_char,
    pub client_brand_name: *const c_char,
    pub client_model_name: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventShuffleChanged {
    pub shuffle: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventRepeatChanged {
    pub context: bool,
    pub track: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventAutoPlayChanged {
    pub auto_play: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventFilterExplicitContentChanged {
    pub filter: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerEventSetQueue {
    pub context_uri: *const c_char,
    pub has_current_track: bool,
    pub current_track: PlayerQueueTrack,
    pub next_tracks: *mut PlayerQueueTrack,
    pub next_tracks_count: usize,
    pub previous_tracks: *mut PlayerQueueTrack,
    pub previous_tracks_count: usize,
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
    pub track_changed: PlayerEventTrackChanged,
    pub session_connected: PlayerEventSessionConnected,
    pub session_disconnected: PlayerEventSessionDisconnected,
    pub session_client_changed: PlayerEventSessionClientChanged,
    pub shuffle_changed: PlayerEventShuffleChanged,
    pub repeat_changed: PlayerEventRepeatChanged,
    pub auto_play_changed: PlayerEventAutoPlayChanged,
    pub filter_explicit_content_changed: PlayerEventFilterExplicitContentChanged,
    pub set_queue: PlayerEventSetQueue,
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
    PLAYER_EVENT_AUTO_PLAY_CHANGED,
    PLAYER_EVENT_FILTER_EXPLICIT_CONTENT_CHANGED,
    PLAYER_EVENT_SET_QUEUE,
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
