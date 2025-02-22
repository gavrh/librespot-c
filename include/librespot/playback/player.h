#ifndef LIBRESPOT_PLAYBACK_PLAYER_H
#define LIBRESPOT_PLAYBACK_PLAYER_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Player Player;
extern Player* player_new();
extern void player_free(Player*);
extern void player_play(Player*);
extern void player_pause(Player*);
extern void player_set_volume(Player*, int);
extern int player_get_volume(const Player*);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_PLAYBACK_PLAYER_H
