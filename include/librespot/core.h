#ifndef LIBRESPOT_CORE_H
#define LIBRESPOT_CORE_H

#include <stdbool.h>
#include <stdint.h>

#include <librespot/discovery.h>
#include <librespot/error.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SessionConfig SessionConfig;
extern SessionConfig* session_config_default(void);
extern void session_config_free(SessionConfig* session_config);

extern LibrespotStatus session_config_set_client_id(SessionConfig* session_config, const char* client_id);
extern LibrespotStatus session_config_set_device_id(SessionConfig* session_config, const char* device_id);
extern LibrespotStatus session_config_set_proxy(SessionConfig* session_config, const char* proxy);
extern LibrespotStatus session_config_clear_proxy(SessionConfig* session_config);
extern LibrespotStatus session_config_set_ap_port(SessionConfig* session_config, uint16_t ap_port);
extern LibrespotStatus session_config_clear_ap_port(SessionConfig* session_config);
extern LibrespotStatus session_config_set_tmp_dir(SessionConfig* session_config, const char* tmp_dir);
extern LibrespotStatus session_config_set_autoplay(SessionConfig* session_config, bool autoplay);
extern LibrespotStatus session_config_clear_autoplay(SessionConfig* session_config);

typedef struct Session Session;
extern Session* session_new(SessionConfig* session_config);
extern void session_free(Session* session);
extern LibrespotStatus session_connect(Session* session, Credentials* credentials);
extern bool session_is_valid(Session* session);
extern void session_shutdown(Session* session);

typedef struct SpotifyUri SpotifyUri;
extern SpotifyUri* spotify_uri_new(const char* uri);
extern void spotify_uri_free(SpotifyUri* spotify_uri);
extern const char* spotify_uri_to_string(SpotifyUri* spotify_uri);
extern const char* spotify_uri_item_type(SpotifyUri* spotify_uri);
extern bool spotify_uri_is_playable(SpotifyUri* spotify_uri);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_CORE_H
