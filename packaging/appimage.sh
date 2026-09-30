#!/usr/bin/env bash
# Builds packaging/Default_Apps-x86_64.AppImage with GTK 4 and libadwaita bundled.
# Needs: cargo, curl, and the gtk4 + libadwaita development files on this machine.
# linuxdeploy-plugin-gtk is deliberately not used: it expects a /usr/lib/gtk-4.0
# module dir that Arch does not ship, and forces X11 and the Adwaita theme.
set -euo pipefail

id=dev.loseardes77.DefaultApps
here=$(cd "$(dirname "$0")" && pwd)
root=$(dirname "$here")
tools=$here/.appimage-tools
appdir=$here/AppDir

mkdir -p "$tools"
fetch() { # fetch <url>: download once into $tools and make it executable
  local file=$tools/${1##*/}
  [ -x "$file" ] || { curl -fL --output "$file" "$1" && chmod +x "$file"; }
}
fetch https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage

cd "$root"
cargo build --release --locked

rm -rf "$appdir"
cd "$here"
APPIMAGE_EXTRACT_AND_RUN=1 OUTPUT=Default_Apps-x86_64.AppImage \
  "$tools/linuxdeploy-x86_64.AppImage" --appdir "$appdir" \
  --executable "$root/target/release/default-apps" \
  --desktop-file "$root/data/$id.desktop" \
  --icon-file "$root/data/$id.svg" \
  --output appimage
echo "Built $here/Default_Apps-x86_64.AppImage"
