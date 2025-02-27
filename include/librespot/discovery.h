#ifndef LIBRESPOT_DISCOVERY_H
#define LIBRESPOT_DISCOVERY_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Credentials Credentials;
extern Credentials* credentials_new(const char* access_token);
extern void credentials_free(Credentials* credentials);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_DISCOVERY_H
