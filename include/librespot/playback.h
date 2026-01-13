#ifndef LIBRESPOT_PLAYBACK_H
#define LIBRESPOT_PLAYBACK_H

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
extern void player_play(Player* player);
extern void player_pause(Player* player);
extern void player_seek(Player* player, uint32_t position_ms);

typedef enum {
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
    PLAYER_EVENT_REPEAT_CHANGED
} PlayerEventType;

typedef struct {
    uint64_t play_request_id;
} PlayerEventPlayRequestIdChanged;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
} PlayerEventStopped;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
    uint32_t position_ms;
} PlayerEventLoading;

typedef struct {
    SpotifyUri* track_id;
} PlayerEventPreloading;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
    uint32_t position_ms;
} PlayerEventPlaying;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
    uint32_t position_ms;
} PlayerEventPaused;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
} PlayerEventTimeToPreloadNextTrack;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
} PlayerEventEndOfTrack;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
} PlayerEventUnavailable;

typedef struct {
    uint16_t volume;
} PlayerEventVolumeChanged;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
    uint32_t position_ms;
} PlayerEventPositionCorrection;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
    uint32_t position_ms;
} PlayerEventPositionChanged;

typedef struct {
    uint64_t play_request_id;
    SpotifyUri* track_id;
    uint32_t position_ms;
} PlayerEventSeeked;

typedef struct {} PlayerEventTrackChanged;

typedef struct {} PlayerEventSessionConnected;

typedef struct {} PlayerEventSessionDisconnected;

typedef struct {} PlayerEventSessionClientChanged;

typedef struct {} PlayerEventShuffleChanged;

typedef struct {} PlayerEventRepeatChanged;

typedef union {
    PlayerEventPlayRequestIdChanged play_request_id_changed;
    PlayerEventStopped stopped;
    PlayerEventLoading loading;
    PlayerEventPreloading preloading;
    PlayerEventPlaying playing;
    PlayerEventPaused paused;
    PlayerEventTimeToPreloadNextTrack time_to_preload_next_track;
    PlayerEventEndOfTrack end_of_track;
    PlayerEventUnavailable unavailable;
    PlayerEventVolumeChanged volume_changed;
    PlayerEventPositionCorrection position_correction;
    PlayerEventPositionChanged position_changed;
    PlayerEventSeeked seeked;
    PlayerEventTrackChanged track_changed;
    PlayerEventSessionConnected session_connected;
    PlayerEventSessionDisconnected session_disconnected;
    PlayerEventSessionClientChanged session_client_changed;
    PlayerEventShuffleChanged shuffle_changed;
    PlayerEventRepeatChanged repeat_changed;
} PlayerEventData;

typedef struct {
    PlayerEventType event;
    PlayerEventData data;
} PlayerEvent;
extern PlayerEvent* player_event_new();
extern void player_event_free(PlayerEvent* player_event);

typedef struct PlayerChannel PlayerChannel;
extern PlayerChannel* player_channel_get(Player* player);
extern void player_channel_free(PlayerChannel* player_channel);
extern bool player_channel_poll(PlayerChannel* player_channel, PlayerEvent* player_event);


#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_PLAYBACK_H
