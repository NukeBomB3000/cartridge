#!/usr/bin/env bash
# Renders cartridge-logo.html into the logo the README and the site show: the boot logo's
# `pixel` style (see bootlogo.sh), on nothing instead of the shelf's grid, cut to the logo.
#
#   sitelogo.sh OUT [LOGO_WIDTH] [SCALE]
#
# The logo is drawn as pixel art LOGO_WIDTH / 2 pixels wide (414 by default, the boot logo's
# width on a 640x480 panel), in the logo's own colours, with the one-pixel white border round
# it, then blown up SCALE times without smoothing: 2 (the default) is what the boot screen
# shows, each pixel 2x2, and 1 is the pixel art itself. OUT is a png with a transparent
# background, cropped to the border. Needs Google Chrome (CHROME overrides where) and
# ImageMagick.
#
#   sitelogo.sh cartridge-logo.png              the site's, 420x232
#   sitelogo.sh cartridge-logo-small.png 320    the site's on a phone, 324x180
#   sitelogo.sh cartridge-logo@2x.png 414 4     the README's, shown at 420x232 so it stays
#                                                sharp on a 2x screen, where GitHub smooths it
set -euo pipefail

out=$1 width=${2:-414} scale=${3:-2}
here=$(cd "$(dirname "$0")" && pwd)
logo="$here/cartridge-logo.html"
chrome=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# laid out as the 640x480 boot logo is, so its pixels are the same ones
w=640 h=480 lw=320 lh=240
"$chrome" --headless=new --disable-gpu --hide-scrollbars --window-size="$((lw + 300)),$((lh + 300))" \
  --force-device-scale-factor=4 --default-background-color=00000000 --screenshot="$tmp/4x.png" \
  ${CHROME_FLAGS:-} "file://$logo?w=$w&h=$h&logo=$width&pixel=2" 2>/dev/null
magick "$tmp/4x.png" -crop "$((lw * 4))x$((lh * 4))+0+0" +repage -filter Box -resize "${lw}x${lh}!" "$tmp/low.png"

# the logo's own colours, as bootlogo.sh has them
palette=()
for c in '#050508' '#1c1c1e' '#000000' '#141519' \
  '#ffffff' '#e6e8ee' '#c3c7d1' '#9399a6' '#62677a' '#383c48' \
  '#0c037f' '#2b2fa8' '#061c55' '#05367e' '#065a99' '#0a7fb4' '#0ca2cc' '#3cc4e4' \
  '#e41c14' '#ee3a16' '#f2621a' '#f68e1a' '#f8b816' '#fadc12' '#fde96a' \
  '#b81204' '#dc2a0c' '#f4480e' '#fb6a12' '#ff8e22' '#ffb63c' '#ffe080' '#fff6d0' '#c86a3a'; do
  palette+=("xc:$c")
done
magick "${palette[@]}" +append "$tmp/palette.png"
magick "$tmp/low.png" -alpha extract -threshold 50% "$tmp/mask.png"
magick "$tmp/low.png" -background black -alpha remove -alpha off +dither -remap "$tmp/palette.png" "$tmp/colour.png"
magick "$tmp/mask.png" -morphology Dilate Diamond:1 "$tmp/ring.png"

# the white border, the logo over it, nothing around them; cut to the border and blown up
magick -size "${lw}x${lh}" xc:none \
  \( -size "${lw}x${lh}" xc:white "$tmp/ring.png" -alpha off -compose CopyOpacity -composite \) -compose over -composite \
  \( "$tmp/colour.png" "$tmp/mask.png" -alpha off -compose CopyOpacity -composite \) -compose over -composite \
  -trim +repage -filter point -resize "$((scale * 100))%" \
  -strip -define png:compression-level=9 -define png:compression-filter=1 -define png:exclude-chunks=all "$out"
echo "wrote $out ($(magick "$out" -format '%wx%h' info:))"
