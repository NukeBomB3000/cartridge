#!/bin/bash
# Screenshots assets/ui/flow.html's keyboard into ref/, the modal alone at one pixel to an art
# pixel, for keyboard.rs to hold the keyboard's faces to: a canvas, a backdrop colour and the
# key flow.html's `row=` puts the cursor on. Run it again after changing the keyboard in flow/;
# the faces must then change to match, pixel for pixel.
#
#   crates/cartridge-ui/tests/suite/keyboard/make-refs.sh
#
# flow.html draws at two screen pixels to an art pixel, every art pixel one colour, so ImageMagick
# takes one pixel of each two by two after cutting the modal out (its rect: 16 and 14 art pixels
# in, kit/layout.rs's margins, above the bar's 15).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
flow="$(cd "$here/../../../../../assets/ui" && pwd)/flow.html"
chrome="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
shot="$(mktemp -t keyboard-ref).png"
mkdir -p "$here/ref"

# canvas, backdrop colour, then flow.html's row=: the key's row and its column of ten
shoot() {
  local w="${1%x*}" h="${1#*x}"
  "$chrome" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
    --virtual-time-budget=3000 --window-size="$w,$h" --screenshot="$shot" \
    "file://$flow#screen=keyboard&size=$1&colour=$2&row=$3&shot=1" 2>/dev/null
  magick "$shot" -crop "$((w - 64))x$((h - 68))+32+28" +repage -sample 50% \
    "$here/ref/keyboard-$1-$2-${3/,/-}.png"
}

shoot 640x480 night 0,0
shoot 640x480 amber 4,3
shoot 720x480 night 2,5
shoot 720x480 amber 3,9
shoot 720x720 night 1,0
shoot 720x720 amber 4,8
rm -f "$shot"
