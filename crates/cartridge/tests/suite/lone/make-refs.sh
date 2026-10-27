#!/bin/bash
# Screenshots flow.html's lone panels into ref/, the whole 640x480 screen at one pixel to a
# pixel, for render_lone.rs to hold the Rust panels to. Run it again after changing a screen in
# assets/ui/flow.html; the panels must then change to match, pixel for pixel.
#
#   crates/cartridge/tests/suite/lone/make-refs.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
flow="$(cd "$here/../../../../../assets/ui" && pwd)/flow.html"
chrome="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
mkdir -p "$here/ref"

for screen in controls background datetime labels power restarting poweroff; do
  for colour in night amber; do
    "$chrome" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
      --virtual-time-budget=3000 --window-size=640,480 \
      --screenshot="$here/ref/$screen-$colour.png" \
      "file://$flow#screen=$screen&colour=$colour&shot=1" 2>/dev/null
  done
done
