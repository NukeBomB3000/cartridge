#!/usr/bin/env bash
# Renders bootcart.html into the boot logo for a W x H panel: the Game Boy Advance cart with
# the Cartridge logo on its label, over the shelf's grid.
#
#   bootcart.sh W H OUT
#
# OUT is written as a 24-bit bmp when it ends in .bmp, the only kind the bootloader draws, and
# as a png otherwise. Needs Google Chrome (CHROME overrides where) and ImageMagick.
set -euo pipefail

w=$1 h=$2 out=$3
here=$(cd "$(dirname "$0")" && pwd)
chrome=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# drawn at 4x and brought down with a box filter; the window is bigger than the panel because
# Chrome won't go below 500px wide, and the extra room is cropped off
"$chrome" --headless=new --disable-gpu --hide-scrollbars --allow-file-access-from-files \
  --window-size="$((w * 4 + 300)),$((h * 4 + 300))" --force-device-scale-factor=1 \
  --screenshot="$tmp/4x.png" "file://$here/bootcart.html?w=$((w * 4))&h=$((h * 4))&scale=4" 2>/dev/null
magick "$tmp/4x.png" -crop "$((w * 4))x$((h * 4))+0+0" +repage -filter Box -resize "${w}x${h}!" \
  -alpha off "$tmp/out.png"
case $out in
*.bmp) magick "$tmp/out.png" -type TrueColor "BMP3:$out" ;;
*) cp "$tmp/out.png" "$out" ;;
esac
echo "wrote $out"
