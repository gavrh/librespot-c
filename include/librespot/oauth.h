#ifndef LIBRESPOT_OAUTH_H
#define LIBRESPOT_OAUTH_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Authenticator Authenticator;
typedef enum AuthScope {
    APP_REMOTE_CONTROL,
    PLAYLIST_MODIFY,
    PLAYLIST_MODIFY_PRIVATE,
    PLAYLIST_MODIFY_PUBLIC,
    PLAYLIST_READ,
    PLAYLIST_READ_COLLABORATIVE,
    PLAYLIST_READ_PRIVATE,
    STREAMING,
    UGC_IMAGE_UPLOAD,
    USER_FOLLOW_MODIFY,
    USER_FOLLOW_READ,
    USER_LIBRARY_MODIFY,
    USER_LIBRARY_READ,
    USER_MODIFY,
    USER_MODIFY_PLAYBACK_STATE,
    USER_MODIFY_PRIVATE,
    USER_PERSONALIZED,
    USER_READ_BIRTHDATE,
    USER_READ_CURRENTLY_PLAYING,
    USER_READ_EMAIL,
    USER_READ_PLAY_HISTORY,
    USER_READ_PLAYBACK_POSITION,
    USER_READ_PLAYBACK_STATE,
    USER_READ_PRIVATE,
    USER_READ_RECENTLY_PLAYED,
    USER_TOP_READ
} AuthScope;

extern Authenticator* authenticator_new(const char* client_id, const char* redirect_uri);
extern void authenticator_free(Authenticator* auther);
extern void authenticator_custom_message(Authenticator* auther, const char* message);
extern void authenticator_open(Authenticator* auther);

typedef struct OAuth OAuth;
extern OAuth* authenticator_build(Authenticator* auther);
extern const char* oauth_access_token(OAuth* auth);
extern const char* oauth_refresh_token(OAuth* auth);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_OAUTH_H
