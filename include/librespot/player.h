#ifndef LIBRESPOT_PLAYER_H
#define LIBRESPOT_PLAYER_H

extern "C" {

struct Player;
Player *player_new();
void player_free(Player*);
void player_set_volume(Player*, int);
int player_get_volume(const Player*);

}

#endif // LIBRESPOT_PLAYER_H
