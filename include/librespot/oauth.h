#ifndef LIBRESPOT_OAUTH_H
#define LIBRESPOT_OAUTH_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Authenticator Authenticator;
typedef struct OAuth OAuth;

extern Authenticator* authenticator_new(const char* client_id, const char* redirect_uri);
extern void authenticator_free(Authenticator* auther);
extern void authenticator_custom_message(Authenticator* auther, const char* message);
extern void authenticator_open(Authenticator* auther);

extern OAuth* authenticator_build(Authenticator* auther);
extern const char* oauth_access_token(OAuth* auth);
extern const char* oauth_refresh_token(OAuth* auth);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_OAUTH_H
