#!/usr/bin/env bash
# Draws the README's section strips with the UI kit: each an <cartridge-panel>'s header, active,
# 380 art pixels wide, in Night, cut out on nothing at 2x so GitHub can show it at 760x28 and
# keep it sharp on a 2x screen. Needs Google Chrome (CHROME overrides where) and ImageMagick.
#
#   strips.sh
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
kit=$(cd "$here/../../../assets/ui" && pwd)
chrome=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# file name, then the title in the strip
strips=(
  "supported-devices|SUPPORTED DEVICES"
  "highlights|HIGHLIGHTS"
  "screens|SCREENS"
  "quick-start|QUICK START"
  "credits|CREDITS"
)

# one panel to a strip, every 80 pixels down the page
panels=""
for i in "${!strips[@]}"; do
  panels+="<cartridge-panel label=\"${strips[$i]#*|}\" active style=\"top: $((i * 80))px\"></cartridge-panel>"
done
cat > "$tmp/strips.html" <<HTML
<!doctype html>
<meta charset="utf-8">
<base href="file://$kit/">
<link rel="stylesheet" href="foundation/fonts.css">
<link rel="stylesheet" href="foundation/tokens.css">
<script src="foundation/pixel.js"></script>
<script src="foundation/backdrop.js"></script>
<script src="foundation/palette.js"></script>
<script src="components/cartridge-panel.js"></script>
<style>
  html, body { margin: 0; background: transparent; }
  cartridge-panel { position: absolute; left: 0; width: calc(380 * var(--u)); height: calc(30 * var(--u)); }
</style>
$panels
HTML

"$chrome" --headless=new --disable-gpu --hide-scrollbars --window-size=800,$((${#strips[@]} * 80)) \
  --force-device-scale-factor=2 --default-background-color=00000000 --virtual-time-budget=2000 \
  --allow-file-access-from-files --screenshot="$tmp/strips.png" "file://$tmp/strips.html" 2>/dev/null

# the panel's top edge and its 13-pixel header, 14 art pixels, 56 pixels at 2x
for i in "${!strips[@]}"; do
  out="$here/${strips[$i]%%|*}.png"
  magick "$tmp/strips.png" -crop "1520x56+0+$((i * 160))" +repage -strip \
    -define png:compression-level=9 -define png:exclude-chunks=all "$out"
  echo "wrote $out"
done
