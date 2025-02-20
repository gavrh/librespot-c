#ifndef LIBRESPOT_PRIVATE_PLAYBACK_PLAYER_RUST_H
#define LIBRESPOT_PRIVATE_PLAYBACK_PLAYER_RUST_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Player Player;

extern Player* player_new_rust();
extern void player_free_rust(Player*);
extern void player_play_rust(Player*);
extern void player_pause_rust(Player*);
extern void player_set_volume_rust(Player*, int);
extern int player_get_volume_rust(const Player*);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_PRIVATE_PLAYBACK_PLAYER_RUST_H

