#ifndef LIBRESPOT_PLAYBACK_H
#define LIBRESPOT_PLAYBACK_H

#include <librespot/core.h>
#include <stdint.h>

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

typedef struct PlayerConfig PlayerConfig;
extern PlayerConfig* player_config_default();
extern void player_config_free(PlayerConfig* player_config);

typedef struct Player Player;
extern Player* player_new(PlayerConfig* player_config, Session* session, Mixer* mixer, const char* audio_backend);
// extern void player_free(Player* player);
extern bool player_is_valid(Player* player);
extern void player_load(Player* player, const char* spotify_uri, bool start_playing, uint32_t position_ms);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_PLAYBACK_H
