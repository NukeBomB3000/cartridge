#!/bin/sh
# The Cartridge logo as the boot screen's BIOS logo, where an Award BIOS put its Energy Star:
# 176 px wide, in 16 colours with no dither and its edge cut clean, the way a BIOS's VGA logo
# was drawn. Needs ImageMagick.
#
#   bioslogo.sh OUT.png
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
magick "$here/cartridge-logo.png" -filter Lanczos -resize 176x \
  -channel A -threshold 50% +channel -dither None -colors 16 "PNG32:$1"
