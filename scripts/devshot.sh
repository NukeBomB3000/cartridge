#!/bin/sh
# Takes a screenshot of the handheld's screen over adb, as a PNG.
#
#   scripts/devshot.sh NAME [DIR]    writes DIR/NAME.png (DIR defaults to screenshots/)
#
# Reads the framebuffer of the visible page (720x480 panels; the pan offset says which of the two
# pages is on screen). Needs adb, a device with root and python3 with Pillow.
set -e
name=${1:?usage: devshot.sh NAME [DIR]}
dir=${2:-screenshots}
mkdir -p "$dir"
raw=$(mktemp)
trap 'rm -f "$raw"' EXIT
page=$(adb shell 'cat /sys/class/graphics/fb0/pan' | tr -d '\r' | cut -d, -f2)
adb shell 'cat /dev/fb0 > /tmp/fb.raw'
adb pull /tmp/fb.raw "$raw" >/dev/null 2>&1
python3 -I -c "
import sys
from PIL import Image
raw = open(sys.argv[1], 'rb').read()
w, h = 720, 480
o = int(sys.argv[3]) * w * 4
Image.frombuffer('RGBA', (w, h), raw[o:o + w * h * 4], 'raw', 'BGRA', 0, 1).convert('RGB').save(sys.argv[2])
" "$raw" "$dir/$name.png" "$page"
echo "$dir/$name.png"
