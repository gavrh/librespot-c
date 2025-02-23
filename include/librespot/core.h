#ifndef LIBRESPOT_CORE_H
#define LIBRESPOT_CORE_H

#include <librespot/discovery.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SessionConfig SessionConfig;
extern SessionConfig* session_config_default();
extern void session_config_free(SessionConfig* session_config);

typedef struct Session Session;
extern Session* session_new(SessionConfig* session_config);
extern void session_free(Session* session);
extern void session_connect(Session* session, Credentials* credentials);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_CORE_H
