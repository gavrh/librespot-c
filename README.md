# librespot-c
C bindings for the Rust-based [librespot](https://github.com/librespot-org/librespot) library.
It exposes Spotify authentication, session handling, playback, and player events to C and C++ projects.

## Requirements

- [CMake](https://cmake.org/download/) 3.12 or newer
- [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- A C/C++ compiler such as GCC, Clang, or MSVC

## Build

```sh
git clone https://github.com/gavrh/librespot-c.git
cd librespot-c

cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release --parallel
```

The default build uses the rodio backend and the softvol mixer. It needs no extra flags or optional system audio dependencies.

CMake is the supported path for building the C bindings: it generates the public playback header for the selected backend options and passes the selected Cargo features to the Rust build.

## Audio backends

Optional audio backends are OFF by default. Enable one or more when configuring:

| Backend | CMake option | Cargo feature |
| --- | --- | --- |
| ALSA | `-DLIBRESPOT_BACKEND_ALSA=ON` | `backend-alsa` |
| PulseAudio | `-DLIBRESPOT_BACKEND_PULSEAUDIO=ON` | `backend-pulseaudio` |
| JACK | `-DLIBRESPOT_BACKEND_JACK=ON` | `backend-jack` |
| PortAudio | `-DLIBRESPOT_BACKEND_PORTAUDIO=ON` | `backend-portaudio` |
| SDL | `-DLIBRESPOT_BACKEND_SDL=ON` | `backend-sdl` |
| GStreamer | `-DLIBRESPOT_BACKEND_GSTREAMER=ON` | `backend-gstreamer` |
| rodio JACK | `-DLIBRESPOT_BACKEND_RODIOJACK=ON` | `backend-rodiojack` |

For example, to build with ALSA and PulseAudio:

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release \
    -DLIBRESPOT_BACKEND_ALSA=ON \
    -DLIBRESPOT_BACKEND_PULSEAUDIO=ON
cmake --build build --config Release --parallel
```

Backend and mixer enum values are compile-time values. A value that was not enabled is absent from the public `librespot/playback.h`, so referencing it in C is a compile error. `LIBRESPOT_ERR_UNSUPPORTED` only protects against stale headers, mismatched installations, or values cast from arbitrary integers at runtime.

The `pipe` and `subprocess` backends are always available. `pipe` writes raw PCM to standard output. `subprocess` requires a command, which the current API cannot provide, so it may fail at runtime.

## Install

```sh
sudo cmake --install build
```

On Windows:

```powershell
cmake --install build --config Release
```

To install into a custom prefix:

```sh
cmake --install build --prefix "$HOME/.local"
```

## Use from CMake

```cmake
find_package(librespot CONFIG REQUIRED)

add_executable(my_app main.c)
target_link_libraries(my_app PRIVATE librespot)
```

If you installed to a custom prefix, pass it when configuring:

```sh
cmake -S . -B build -DCMAKE_PREFIX_PATH="$HOME/.local"
```

## Get Started

Quick Start Variables:
| Variable | Value |
| --- | --- |
| SPOTIFY_API_CLIENT_ID | 65b708073fc0480ea92a077233ca87bd (Spotify's client id) |
| REDIRECT_URI | http://127.0.0.1:8888/login |
| MIXER | MIXER_DEFAULT |
| BACKEND | BACKEND_DEFAULT |
| TRACK_URI | spotify:track:02JIdsrod3BYucThfUFDUX |

```c
#define _DEFAULT_SOURCE

#include <librespot/core.h>
#include <librespot/discovery.h>
#include <librespot/error.h>
#include <librespot/oauth.h>
#include <librespot/playback.h>

#include <stdio.h>
#include <unistd.h>

int main(void) {
    // authenticate with Spotify
    OAuthBuilder* builder = oauth_builder_new(
        "65b708073fc0480ea92a077233ca87bd",
        "http://127.0.0.1:8888/login"
    );

    oauth_builder_auto_open(builder);
    oauth_builder_add_scope(builder, OAUTH_SCOPE_STREAMING);

    OAuth* auth = oauth_build(builder);
    oauth_builder_free(builder);

    // create credentials from the access token
    Credentials* credentials = credentials_new(oauth_access_token(auth));

    // create and connect a session
    SessionConfig* session_config = session_config_default();
    Session* session = session_new(session_config);
    session_config_free(session_config);

    session_connect(session, credentials);
    credentials_free(credentials);

    // create the mixer and player
    MixerConfig* mixer_config = mixer_config_default();
    Mixer* mixer = mixer_new(mixer_config, MIXER_DEFAULT);
    mixer_config_free(mixer_config);

    PlayerConfig* player_config = player_config_default();
    Player* player = player_new(player_config, session, mixer, BACKEND_DEFAULT);
    player_config_free(player_config);

    // load a track
    player_load(player, "spotify:track:02JIdsrod3BYucThfUFDUX", true, 0);

    // poll for player events
    PlayerChannel* channel = player_channel_get(player);
    PlayerEvent event;

    while (session_is_valid(session)) {
        if (!player_channel_poll(channel, &event)) {
            usleep(10000);
            continue;
        }

        switch (event.event) {
            case PLAYER_EVENT_PLAYING:
                printf("Playing %s\n", spotify_uri_to_string(event.data.playing.track_id));
                break;

            case PLAYER_EVENT_END_OF_TRACK:
                printf("End of track\n");
                break;

            default:
                break;
        }
    }

    // clean up
    player_channel_free(channel);
    player_free(player);
    mixer_free(mixer);
    session_free(session);
    oauth_free(auth);

    return 0;
}
```

## Contributing
Contributions are welcome! If you have suggestions or improvements, feel free to fork the repository, create a pull request, or open an issue.

1. Fork the repository.

2. Create a new branch (`git checkout -b <feature-name>`).

3. Commit your changes (`git commit -am 'Add new feature'`).

4. Push to the branch (`git push origin <feature-name>`.

5. Open a pull request.

A more detailed walkthrough on how to properly contribute to the project will be coming soon...

## License
This project is licensed under the GPL-3.0 License - see the [LICENSE](/LICENSE) file for details.
