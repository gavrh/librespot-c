#ifndef PLAYER_RUST_H
#define PLAYER_RUST_H

#ifdef __cplusplus
extern "C" {
#endif

extern Player* player_new_rust();
extern void player_free_rust(Player*);
extern void player_play_rust(Player*);
extern void player_pause_rust(Player*);
extern void player_set_volume_rust(Player*, int);
extern int player_get_volume_rust(const Player*);

#ifdef __cplusplus
}
#endif

#endif // PLAYER_RUST_H
