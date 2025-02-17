#ifndef LIBRESPOT_PLAYER_HPP
#define LIBRESPOT_PLAYER_HPP

#include <string>

namespace librespot {

class Player {
public:
    Player(const std::string &lib_path);
    ~Player();

    void play_track(const std::string &track_id);

private:
    void* handle;
    typedef void (*PlayTrackFunc)(const char*);
    PlayTrackFunc play_track_func;
};

}

#endif // LIBRESPOT_PLAYER_HPP
