#ifndef LIBRESPOT_PLAYBACK_H
#define LIBRESPOT_PLAYBACK_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct PlayerConfig PlayerConfig;
extern PlayerConfig* player_config_new(bool gapless, bool passthrough, bool normalisation);
extern PlayerConfig* player_config_default();
extern void player_config_free(PlayerConfig* player_config);

typedef struct Player Player;
extern Player* player_new(PlayerConfig* player_config);

typedef struct Mixer Mixer;

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_PLAYBACK_H
