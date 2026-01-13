# librespot-c
C wrapper for the popular Rust-based [librespot](https://github.com/librespot-org/librespot) library,
allowing you to integrate Spotify functionality into your
C and C++ projects. This library enables easy access to
Spotify’s streaming capabilities, including playback, authentication, and more.

## Requirements
- [CMake](https://cmake.org/download/) 3.12 or higher
- [Rust & Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- A C/C++ compiler (e.g., [GCC](https://gcc.gnu.org/install/) or [Clang](https://releases.llvm.org/download.html))

## Installation
```sh
# clone library
git clone https://github.com/gavrh/librespot-c.git
cd librespot-c

# create make files
mkdir build
cd build
cmake ..

# install locally
make
# install globally
sudo make install
```

#### Copy and Paste
> Local Install
> ```sh
> git clone https:://github.com/gavrh/librspot-c.git && cd librespot-c && mkdir build && cd build && cmake .. && > > make
> ```
> Global Install
> ```sh
> git clone https:://github.com/gavrh/librspot-c.git && cd librespot-c && mkdir build && cd build && cmake .. && > > sudo make install
> ```

## Get Started

Quick Start Variables:
| Variable | Value |
| - | - |
| **SPOTIFY_API_CLIENT_ID:** | [ncspot client id](https://github.com/hrkfdn/ncspot/blob/master/src/authentication.rs) |
| **REDIRECT_URI:** | http://127.0.0.1:8888/login |
| **MIXER**: | softvol |
| **BACKEND**: | rodio |

```c
#include <librespot/oauth.h>
#include <librespot/core.h>
#include <librespot/discovery.h>
#include <librespot/playback.h>
#include <stdbool.h>

int main() {

    // authenticate with spotify
    OAuthBuilder* oauth = oauth_builder_new("SPOTIFY_API_CLIENT_ID", "REDIRECT_URI");
    oauth_builder_auto_open(oauth);
    // make sure you're api client is allowed the scopes you add
    oauth_builder_add_scope(oauth, OAUTH_SCOPE_ALL);
    OAuth* auth = oauth_build(oauth);

    // create credentials and session, then connect
    const char* access = oauth_access_token(auth);
    Credentials* creds = credentials_new(access);
    Session* session = session_new(session_config_default());
    session_connect(session, creds);

    // create mixer and player
    Mixer* mixer = mixer_new(mixer_config_default(), "MIXER");
    Player* player = player_new(player_config_default(), session, mixer, "BACKEND");

    // play song
    player_load(player, "spotify:track:<TRACK_ID>", true, 0);

    // player channel and event loop
    PlayerChannel* channel = player_channel_get(player);
    PlayerEvent event;
    while (true) {
        if (player_channel_poll(channel, &event)) {
            switch (event.event) {
                case PLAYER_EVENT_PLAYING:
                    // event.data.playing to access union fields
                    break;
                case PLAYER_EVENT_LOADING:
                    // event.data.laoding to access union fields
                    break;

                // and so on for the other events...

                default:
                    break;
            }
        }
    }

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
