<img src="data/dev.loseardes77.DefaultApps.svg" width="96" align="right" alt="">

# Default Apps

A small GTK4/libadwaita app for choosing which applications open your files and
links on Linux. It is a graphical front end for what `xdg-mime default` does.

- **Default applications**: one-click choices for web browser, file manager,
  terminal, code editor, text editor, PDF, email, images, video, music and
  archives.
- **All file types**: every MIME type that an installed app declares, with
  search.

## Install (Arch Linux)

```sh
cd packaging
makepkg -si
```

This installs the binary, a launcher entry and the icon. Run the PKGBUILD from
`packaging/`, not from the project root.

## AppImage

```sh
packaging/appimage.sh
```

This downloads `linuxdeploy` on first run and writes
`packaging/Default_Apps-x86_64.AppImage` with GTK 4 and libadwaita bundled.
glibc is not bundled, so the result only runs on systems whose glibc is at
least as new as the one on the build machine.

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

Grouped entries set several MIME types at once. For example, *Web browser*
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
is concerned, so they follow *Text editor* rather than *Code editor*.
