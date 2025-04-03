#ifndef LIBRESPOT_PLAYBACK_H
#define LIBRESPOT_PLAYBACK_H

#include <cstdint>
#include <librespot/core.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MixerConfig MixerConfig;
extern MixerConfig* mixer_config_default();
extern void mixer_config_free(MixerConfig* mixer_config);

typedef struct Mixer Mixer;
extern Mixer* mixer_new(MixerConfig* mixer_config, const char* mixer_name);
extern void mixer_free(Mixer* mixer);
extern uint16_t mixer_get_volume(Mixer* mixer);
extern void mixer_set_volume(Mixer* mixer, uint16_t volume);

typedef struct PlayerConfig PlayerConfig;
extern PlayerConfig* player_config_default();
extern void player_config_free(PlayerConfig* player_config);

typedef struct Player Player;
extern Player* player_new(PlayerConfig* player_config, Session* session, Mixer* mixer, const char* audio_backend);
extern void player_free(Player* player);
extern bool player_is_valid(Player* player);
extern void player_load(Player* player, const char* spotify_uri, bool start_playing, uint32_t position_ms);
extern void player_preload(Player* player, const char* spotify_uri);

typedef struct PlayerChannel PlayerChannel;
extern PlayerChannel* player_channel_get(Player* player);
extern void player_channel_free(PlayerChannel* player_channel);
extern bool player_channel_next(PlayerChannel* player_channel);

typedef struct PlayerEvent PlayerEvent;

typedef struct PlayerEventPlayRequestIdChanged PlayerEventPlayRequestIdChanged;
extern PlayerEventPlayRequestIdChanged* player_event_play_request_id_changed_new(PlayerEvent* player_event);
extern void player_event_play_request_id_changed_free(PlayerEventPlayRequestIdChanged* player_event);
extern uint64_t player_event_play_request_id_changed_play_request_id(PlayerEventPlayRequestIdChanged* player_event);

typedef struct PlayerEventStopped PlayerEventStopped;

typedef struct PlayerEventLoading PlayerEventLoading;
extern PlayerEventLoading* player_event_loading_new(PlayerEvent* player_event);
extern void player_event_loading_free(PlayerEventLoading* player_event);
extern uint64_t player_event_loading_play_request_id(PlayerEventLoading* player_event);
extern uint64_t player_event_loading_play_request_id(PlayerEventLoading* player_event);
extern uint32_t player_event_loading_position_ms(PlayerEventLoading* player_event);

typedef struct PlayerEventPreloading PlayerEventPreloading;
typedef struct PlayerEventPlaying PlayerEventPlaying;
typedef struct PlayerEventPaused PlayerEventPaused;
typedef struct PlayerEventTimeToPreloadNextTrack PlayerEventTimeToPreloadNextTrack;
typedef struct PlayerEventEndOfTrack PlayerEventEndOfTrack;
typedef struct PlayerEventUnavailble PlayerEventUnavailable;
typedef struct PlayerEventVolumeChanged PlayerEventVolumeChanged;
typedef struct PlayerEventPositionCorrection PlayerEventPositionCorrection;
typedef struct PlayerEventSeeked PlayerEventSeeked;

typedef struct PlayerEventTrackChanged PlayerEventTrackChanged;
extern PlayerEventTrackChanged* player_event_track_changed_new(PlayerEvent* player_event);
extern void player_event_track_changed_free(PlayerEventTrackChanged* player_event);

typedef struct PlayerEventSessionConnected PlayerEventSessionConnected;
typedef struct PlayerEventSessionDisconnected PlayerEventSessionDisconnected;
typedef struct PlayerEventSessionClientChanged PlayerEventSessionClientChanged;
typedef struct PlayerEventShuffleChanged PlayerEventShuffleChanged;
typedef struct PlayerEventRepeatChanged PlayerEventRepeatChanged;

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_PLAYBACK_H
