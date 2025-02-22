#ifndef LIBRESPOT_OAUTH_H
#define LIBRESPOT_OAUTH_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Authenticator Authenticator;
typedef enum {
    AUTH_SCOPE_APP_REMOTE_CONTROL = 1,
    AUTH_SCOPE_PLAYLIST_MODIFY = 2,
    AUTH_SCOPE_PLAYLIST_MODIFY_PRIVATE = 3,
    AUTH_SCOPE_PLAYLIST_MODIFY_PUBLIC = 4,
    AUTH_SCOPE_PLAYLIST_READ = 5,
    AUTH_SCOPE_PLAYLIST_READ_COLLABORATIVE = 6,
    AUTH_SCOPE_PLAYLIST_READ_PRIVATE = 7,
    AUTH_SCOPE_STREAMING = 8,
    AUTH_SCOPE_UGC_IMAGE_UPLOAD = 9,
    AUTH_SCOPE_USER_FOLLOW_MODIFY = 10,
    AUTH_SCOPE_USER_FOLLOW_READ = 11,
    AUTH_SCOPE_USER_LIBRARY_MODIFY = 12,
    AUTH_SCOPE_USER_LIBRARY_READ = 13,
    AUTH_SCOPE_USER_MODIFY = 14,
    AUTH_SCOPE_USER_MODIFY_PLAYBACK_STATE = 15,
    AUTH_SCOPE_USER_MODIFY_PRIVATE = 16,
    AUTH_SCOPE_USER_PERSONALIZED = 17,
    AUTH_SCOPE_USER_READ_BIRTHDATE = 18,
    AUTH_SCOPE_USER_READ_CURRENTLY_PLAYING = 19,
    AUTH_SCOPE_USER_READ_EMAIL = 20,
    AUTH_SCOPE_USER_READ_PLAY_HISTORY = 21,
    AUTH_SCOPE_USER_READ_PLAYBACK_POSITION = 22,
    AUTH_SCOPE_USER_READ_PLAYBACK_STATE = 23,
    AUTH_SCOPE_USER_READ_PRIVATE = 24,
    AUTH_SCOPE_USER_READ_RECENTLY_PLAYED = 25,
    AUTH_SCOPE_USER_TOP_READ = 26,
    AUTH_SCOPE_ALL = 27
} AuthScope;

extern Authenticator* authenticator_new(const char* client_id, const char* redirect_uri);
extern void authenticator_free(Authenticator* auther);
extern void authenticator_custom_message(Authenticator* auther, const char* message);
extern void authenticator_auto_open(Authenticator* auther);
extern void authenticator_add_scope(Authenticator* auther, AuthScope scope);

typedef struct OAuth OAuth;
extern OAuth* authenticator_build(Authenticator* auther);
extern void oauth_free(OAuth* auth);
extern const char* oauth_access_token(OAuth* auth);
extern const char* oauth_refresh_token(OAuth* auth);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_OAUTH_H
