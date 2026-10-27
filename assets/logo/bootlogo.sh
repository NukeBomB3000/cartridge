#!/usr/bin/env bash
# Renders cartridge-logo.html into a boot logo for a W x H panel.
#
#   bootlogo.sh STYLE W H OUT [LOGO_WIDTH]
#
# STYLE is `pixel`, the one cartridge ships: pixel art on a half-size canvas, each pixel blown
# up to 2x2, in a palette of the logo's own colours, with a one-pixel white border and the
# shelf's grid behind it. Or `smooth`: the logo drawn straight at the panel's pixels, lightly
# sharpened. LOGO_WIDTH is how wide the logo is on the panel, in pixels (414 by default). OUT
# is written as a 24-bit bmp when it ends in .bmp, the only kind the bootloader draws, and as a
# png otherwise. Needs Google Chrome (CHROME overrides where) and ImageMagick.
set -euo pipefail

style=$1 w=$2 h=$3 out=$4 width=${5:-414}
here=$(cd "$(dirname "$0")" && pwd)
logo="$here/cartridge-logo.html"
chrome=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# shot W H FILE QUERY [flags]: the page's top-left W x H at 4x, for a box filter to bring down.
# The window is bigger than that because Chrome won't go below 500px wide; the logo is pinned
# to the top left, so the extra room is only cropped off.
shot() {
  "$chrome" --headless=new --disable-gpu --hide-scrollbars --window-size="$(($1 + 300)),$(($2 + 300))" \
    --force-device-scale-factor=4 --screenshot="$3" "${@:5}" "file://$logo?$4" 2>/dev/null
  magick "$3" -crop "$(($1 * 4))x$(($2 * 4))+0+0" +repage "$3"
}

case $style in
smooth)
  shot "$w" "$h" "$tmp/4x.png" "w=$w&h=$h&logo=$width"
  # the threshold keeps the sharpening off the grid, whose lines are fainter than that
  magick "$tmp/4x.png" -filter Box -resize "${w}x${h}!" -alpha off -unsharp 0x0.5+0.5+0.1 "$tmp/out.png"
  ;;
pixel)
  lw=$((w / 2)) lh=$((h / 2))
  shot "$lw" "$lh" "$tmp/4x.png" "w=$w&h=$h&logo=$width&pixel=2" --default-background-color=00000000
  magick "$tmp/4x.png" -filter Box -resize "${lw}x${lh}!" "$tmp/low.png"
  # the logo's own colours: grid, plate, chrome, navy, the blue and fire faces, the emblem
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
  # the shelf's grid at half size: a line every 20 pixels from the top left corner
  grid=()
  for ((x = 0; x < lw; x += 20)); do grid+=(-draw "line $x,0 $x,$((lh - 1))"); done
  for ((y = 0; y < lh; y += 20)); do grid+=(-draw "line 0,$y $((lw - 1)),$y"); done
  magick -size "${lw}x${lh}" xc:'#050508' -fill '#1c1c1e' -stroke none "${grid[@]}" \
    \( -size "${lw}x${lh}" xc:white "$tmp/ring.png" -alpha off -compose CopyOpacity -composite \) -compose over -composite \
    \( "$tmp/colour.png" "$tmp/mask.png" -alpha off -compose CopyOpacity -composite \) -compose over -composite \
    -alpha off -filter point -resize "${w}x${h}!" "$tmp/out.png"
  ;;
*)
  echo "bootlogo.sh: STYLE is pixel or smooth, not $style" >&2
  exit 2
  ;;
esac

case $out in
*.bmp) magick "$tmp/out.png" -type TrueColor -compress none BMP3:"$out" ;;
*) cp "$tmp/out.png" "$out" ;;
esac
echo "wrote $out"
