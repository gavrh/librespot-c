#include <librespot/player.h>

extern Player* player_new_rust();
extern void player_free_rust(Player*);
extern void player_play_rust(Player*);
extern void player_pause_rust(Player*);
extern void player_set_volume_rust(Player*, int);
extern int player_get_volume_rust(const Player*);

Player* player_new() {
    return player_new_rust();
}

void player_free(Player* player) {
    player_free_rust(player);
}

void player_play(Player* player) {
    player_play_rust(player);
}

void player_pause(Player* player) {
    player_pause_rust(player);
}

void player_set_volume(Player* player, int volume) {
    player_set_volume_rust(player, volume);
}

int player_get_volume(const Player* player) {
    return player_get_volume_rust(player);
}
