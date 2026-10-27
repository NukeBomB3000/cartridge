#!/bin/bash
# Screenshots flow.html's Bluetooth screen into ref/, for bluetooth.rs (and the frontend's own
# render test) to hold what Rust composes to: the modal alone, cropped from the screen at
# modal_rect() and taken back to one pixel per art pixel (each art pixel is a 2x2 block on the
# screen; the script checks every block is one colour). Run it again after changing Bluetooth in
# assets/ui/flow.html or its components.
#
#   crates/cartridge-ui/tests/suite/bluetooth/make-refs.sh
#
# Each ref is named flow-bluetooth-<state>-<canvas>.png, in Night, <state> being flow.html's #bt=;
# the radio's row is lit in each but pairing:
#   on        the screen as it opens, before any look: the radio on and the two paired devices,
#             8BitDo SN30 Pro CONNECTED (gamepad) and WH-1000XM4 PAIRED (audio), then the note
#             "Press X to look for devices."
#   off       the radio off, and the note "Bluetooth is off."
#   empty     the radio on, nothing paired, before any look: "Press X to look for devices." alone
#   looking   a look under way: the paired devices, Keychron K3 (keyboard) found so far, and the
#             note "Looking for devices…"
#   found     after the look: the paired devices, then Keychron K3 and Mi Smart Band 4 (other),
#             found only, and no note
#   pairing   after the look, with Keychron K3 PAIRING, the cursor on it
# Needs python3 with Pillow.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
flow="$(cd "$here/../../../../../assets/ui" && pwd)/flow.html"
chrome="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
mkdir -p "$here/ref"
shot="$(mktemp -d)"
trap 'rm -rf "$shot"' EXIT

# canvas, state
shoot() {
  local w="${1%x*}" h="${1#*x}" name="flow-bluetooth-$2-$1"
  "$chrome" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
    --virtual-time-budget=3000 --window-size="$w,$h" --screenshot="$shot/$name.png" \
    "file://$flow#screen=bluetooth&size=$1&bt=$2&shot=1" 2>/dev/null
  # modal_rect(), in art pixels: 16 in at the sides, 14 from the top, 5 above the 15 pixel bar
  python3 -I - "$shot/$name.png" "$here/ref/$name.png" "$w" "$h" <<'EOF'
import sys
from PIL import Image
src, out, w, h = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
x, y = 16, 14
mw, mh = w // 2 - 2 * 16, h // 2 - 15 - 14 - 5
im = Image.open(src).convert('RGB')
art = Image.new('RGB', (mw, mh))
for ay in range(mh):
    for ax in range(mw):
        block = {im.getpixel((2 * (x + ax) + dx, 2 * (y + ay) + dy)) for dx in (0, 1) for dy in (0, 1)}
        if len(block) != 1:
            sys.exit(f'{src}: art pixel ({ax}, {ay}) is not one colour: {block}')
        art.putpixel((ax, ay), block.pop())
art.save(out)
EOF
}

for canvas in 640x480 720x480 720x720; do
  for state in on off empty looking found pairing; do
    shoot "$canvas" "$state"
  done
done
