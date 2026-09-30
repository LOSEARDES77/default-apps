<img src="data/dev.loseardes77.DefaultApps.svg" width="96" align="right" alt="">

# Default Apps

A small GTK4/libadwaita app for choosing which applications open your files and
links on Linux. It is a graphical front end for what `xdg-mime default` does.

- **Default applications**: one-click choices for web browser, file manager,
  terminal, code editor, text editor, PDF, email, images, video, music and
  archives.
- **All file types**: every MIME type that an installed app declares, with
  search.

## Install

Tagged versions publish a `.deb`, an `.rpm`, an Arch package and an AppImage on the
[releases page](https://github.com/LOSEARDES77/default-apps/releases).

| System                    | Command                                              |
| ------------------------- | ---------------------------------------------------- |
| Debian 13+, Ubuntu 24.04+ | `sudo apt install ./default-apps_*.deb`              |
| Fedora                    | `sudo dnf install ./default-apps-*.rpm`              |
| Arch Linux                | `sudo pacman -U ./default-apps-*.pkg.tar.zst`        |
| Anything else             | `chmod +x Default_Apps-x86_64.AppImage`, then run it |

The AppImage bundles GTK 4 and libadwaita and needs glibc 2.39 or newer.

## Building the packages yourself

Each package has to be built on the distro it targets.

```sh
cargo install cargo-deb && cargo deb                                        # Debian, Ubuntu
cargo install cargo-generate-rpm && cargo build --release && cargo generate-rpm   # Fedora
cd packaging && makepkg -si                                                 # Arch
packaging/appimage.sh                                                       # AppImage
```

Run the PKGBUILD from `packaging/`, not from the project root. An AppImage only
runs on systems whose glibc is at least as new as the build machine's, which is
why releases are built on Ubuntu 24.04.

## Build and run from source

Requires Rust, GTK 4 and libadwaita 1.5 or newer.

```sh
cargo run --release
```

## How it works

Choices are written through GIO to `~/.config/mimeapps.list`, the same file
`xdg-mime default` writes. You can check any result from a shell:

```sh
xdg-mime query default application/pdf
```

Grouped entries set several MIME types at once. For example, _Web browser_
sets `x-scheme-handler/http`, `x-scheme-handler/https` and `text/html`. The
groups are defined in the `SPECIALS` table at the top of `src/main.rs`.

### Terminal

Linux has no standard "default terminal" setting. The choice is stored under
`x-scheme-handler/terminal`, which most desktops do not read by themselves. To
use it, launch your terminal through a lookup, for example in Hyprland:

```
bind = SUPER, Return, exec, gtk-launch $(xdg-mime query default x-scheme-handler/terminal)
```

### Files without their own type

`.ini`, `.conf`, `.env` and similar files are plain text as far as the system
is concerned, so they follow _Text editor_ rather than _Code editor_.

## License

MIT, see [LICENSE](LICENSE).
