#ifndef LIBRESPOT_PLAYER_H
#define LIBRESPOT_PLAYER_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Player Player;
Player* player_new();
void player_free(Player*);
void player_play(Player*);
void player_pause(Player*);
void player_set_volume(Player*, int);
int player_get_volume(const Player*);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_PLAYER_H
