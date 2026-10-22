#ifndef LIBRESPOT_ERROR_H
#define LIBRESPOT_ERROR_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum LibrespotStatus {
    LIBRESPOT_OK = 0,
    LIBRESPOT_ERR_INVALID_ARGUMENT = -1,
    LIBRESPOT_ERR_INVALID_STRING = -2,
    LIBRESPOT_ERR_FAILED = -3
} LibrespotStatus;

extern const char* librespot_last_error(void);
extern void librespot_clear_error(void);

#ifdef __cplusplus
}
#endif

#endif // LIBRESPOT_ERROR_H
