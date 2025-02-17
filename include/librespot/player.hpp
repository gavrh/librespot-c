#ifndef LIBRESPOT_PLAYER_H
#define LIBRESPOT_PLAYER_H

#include <string>

class Librespot {
public:
    Librespot(const std::string &lib_path);
    ~Librespot();

    void play_track(const std::string &track_id);

private:
    void* handle;
    typedef void (*PlayTrackFunc)(const char*);
    PlayTrackFunc play_track_func;
};

#endif // LIBRESPOT_PLAYER_H
