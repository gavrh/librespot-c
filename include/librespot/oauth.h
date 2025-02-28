#ifndef LIBRESPOT_OAUTH_H
#define LIBRESPOT_OAUTH_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    OAUTH_SCOPE_APP_REMOTE_CONTROL = 1,
    OAUTH_SCOPE_PLAYLIST_MODIFY = 2,
    OAUTH_SCOPE_PLAYLIST_MODIFY_PRIVATE = 3,
    OAUTH_SCOPE_PLAYLIST_MODIFY_PUBLIC = 4,
    OAUTH_SCOPE_PLAYLIST_READ = 5,
    OAUTH_SCOPE_PLAYLIST_READ_COLLABORATIVE = 6,
    OAUTH_SCOPE_PLAYLIST_READ_PRIVATE = 7,
    OAUTH_SCOPE_STREAMING = 8,
    OAUTH_SCOPE_UGC_IMAGE_UPLOAD = 9,
    OAUTH_SCOPE_USER_FOLLOW_MODIFY = 10,
    OAUTH_SCOPE_USER_FOLLOW_READ = 11,
    OAUTH_SCOPE_USER_LIBRARY_MODIFY = 12,
    OAUTH_SCOPE_USER_LIBRARY_READ = 13,
    OAUTH_SCOPE_USER_MODIFY = 14,
    OAUTH_SCOPE_USER_MODIFY_PLAYBACK_STATE = 15,
    OAUTH_SCOPE_USER_MODIFY_PRIVATE = 16,
    OAUTH_SCOPE_USER_PERSONALIZED = 17,
    OAUTH_SCOPE_USER_READ_BIRTHDATE = 18,
    OAUTH_SCOPE_USER_READ_CURRENTLY_PLAYING = 19,
    OAUTH_SCOPE_USER_READ_EMAIL = 20,
    OAUTH_SCOPE_USER_READ_PLAY_HISTORY = 21,
    OAUTH_SCOPE_USER_READ_PLAYBACK_POSITION = 22,
    OAUTH_SCOPE_USER_READ_PLAYBACK_STATE = 23,
    OAUTH_SCOPE_USER_READ_PRIVATE = 24,
    OAUTH_SCOPE_USER_READ_RECENTLY_PLAYED = 25,
    OAUTH_SCOPE_USER_TOP_READ = 26,
    OAUTH_SCOPE_ALL = 27
} OAuthScope;

typedef struct OAuthBuilder OAuthBuilder;
extern OAuthBuilder* oauth_builder_new(const char* client_id, const char* redirect_uri);
extern void oauth_builder_free(OAuthBuilder* oauth_builder);
extern void oauth_builder_custom_message(OAuthBuilder* oauth_builder, const char* message);
extern void oauth_builder_auto_open(OAuthBuilder* oauth_builder);
extern void oauth_builder_add_scope(OAuthBuilder* oauth_builder, OAuthScope scope);

typedef struct OAuth OAuth;
extern OAuth* oauth_build(OAuthBuilder* oauth_builder);
extern void oauth_free(OAuth* oauth);
extern const char* oauth_access_token(OAuth* oauth);
extern const char* oauth_refresh_token(OAuth* oauth);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_OAUTH_H
