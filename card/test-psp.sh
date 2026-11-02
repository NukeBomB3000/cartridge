#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/System/psp.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT HUP INT TERM

fail() {
	echo "psp: $*" >&2
	exit 1
}

setup() {
	rm -rf "$TMP/card" "$TMP/run" "$TMP/bin"
	mkdir -p "$TMP/card/Config" "$TMP/card/Games/PSP" "$TMP/card/System/psp/.config/ppsspp/PSP/SYSTEM" \
		"$TMP/run" "$TMP/bin"
	printf 'volume=15\nvolume_hp=60\n' > "$TMP/card/Config/cartridge.state"
	: > "$TMP/gpio"
	: > "$TMP/card/Games/PSP/Racing Game.cso"
	printf 'DisplayAspectRatio = 0.848000\nUIScaleFactor = 0\niShowStatusFlags = 3\nOther = 1\n' \
		> "$TMP/card/System/psp/.config/ppsspp/PSP/SYSTEM/ppsspp.ini"
	# the emulator writes down what it was run with
	cat > "$TMP/card/System/psp/PPSSPPSDL" <<EOT
#!/bin/sh
echo "\$*" > "$TMP/emulator-args"
echo "\${XDG_CONFIG_HOME:-}" > "$TMP/emulator-home"
sleep 1
EOT
	# mount and umount are only noted: the test may not mount anything
	for c in mount umount; do
		printf '#!/bin/sh\necho "%s $*" >> "%s"\n' "$c" "$TMP/mounts" > "$TMP/bin/$c"
	done
	chmod 755 "$TMP/bin/mount" "$TMP/bin/umount" "$TMP/card/System/psp/PPSSPPSDL"
	: > "$TMP/mounts"
}

psp() {
	PATH="$TMP/bin:$PATH" CARTRIDGE_ROOT="$TMP/card" AGS_RUN="$TMP/run" AGS_GPIO="$TMP/gpio" \
		AGS_EVTEST="$TMP/evtest" AGS_FB=/dev/null AGS_KEYS=/dev/null AGS_CARTRIDGE=/nonexistent \
		sh "$SCRIPT" "$@"
}

key() {
	echo "Event: time 1.1, type 1 (EV_KEY), code $1 ($2), value $3"
}

# the volume keys: each press moves the speaker's level five, and says so
setup
{
	echo '#!/bin/sh'
	echo "echo 'Input driver version is 1.0.1'"
	key 115 KEY_VOLUMEUP 1 | sed 's/^/echo "/; s/$/"/'
	key 115 KEY_VOLUMEUP 0 | sed 's/^/echo "/; s/$/"/'
	key 115 KEY_VOLUMEUP 1 | sed 's/^/echo "/; s/$/"/'
	key 114 KEY_VOLUMEDOWN 1 | sed 's/^/echo "/; s/$/"/'
} > "$TMP/evtest"
chmod 755 "$TMP/evtest"
psp volume
[ "$(cat "$TMP/run/cartridge-volume")" = 20 ] || fail "the level is $(cat "$TMP/run/cartridge-volume"), not 20"
grep -q '^volume=20$' "$TMP/card/Config/cartridge.state" || fail "the speaker's level was not kept"
grep -q '^volume_hp=60$' "$TMP/card/Config/cartridge.state" || fail "the headphones' level changed"
[ "$(cat "$TMP/run/cartridge-hud")" = "20 volume" ] || fail "the gauge was told $(cat "$TMP/run/cartridge-hud")"

# with headphones in the jack the keys move the headphones' level, and the gauge wears them
setup
echo ' gpio-259 (                    |Headphone detection ) in  hi ' > "$TMP/gpio"
cp "$TMP/evtest" "$TMP/evtest.keep" 2>/dev/null || true
{
	echo '#!/bin/sh'
	key 115 KEY_VOLUMEUP 1 | sed 's/^/echo "/; s/$/"/'
} > "$TMP/evtest"
chmod 755 "$TMP/evtest"
psp volume
grep -q '^volume_hp=65$' "$TMP/card/Config/cartridge.state" || fail "the headphones' level was not moved"
grep -q '^volume=15$' "$TMP/card/Config/cartridge.state" || fail "the speaker's level changed"
[ "$(cat "$TMP/run/cartridge-hud")" = "65 headphones" ] || fail "the gauge was told $(cat "$TMP/run/cartridge-hud")"

# the level never leaves 0..100
setup
printf 'volume=98\nvolume_hp=60\n' > "$TMP/card/Config/cartridge.state"
{
	echo '#!/bin/sh'
	for _ in 1 2 3; do key 115 KEY_VOLUMEUP 1 | sed 's/^/echo "/; s/$/"/'; done
} > "$TMP/evtest"
chmod 755 "$TMP/evtest"
psp volume
[ "$(cat "$TMP/run/cartridge-volume")" = 100 ] || fail "the level went past 100"

# running a game: the emulator gets the game, its menu's exit ends it, its folder is found
# through XDG_CONFIG_HOME, its settings are set once, and its saves are the card's
setup
printf '#!/bin/sh\n' > "$TMP/evtest"
chmod 755 "$TMP/evtest"
psp run "$TMP/card/Games/PSP/Racing Game.cso"
grep -q -- "Racing Game.cso --fullscreen --pause-menu-exit" "$TMP/emulator-args" ||
	fail "the emulator was run with: $(cat "$TMP/emulator-args")"
[ "$(cat "$TMP/emulator-home")" = "$TMP/card/System/psp/.config" ] || fail "XDG_CONFIG_HOME is wrong"
ini="$TMP/card/System/psp/.config/ppsspp/PSP/SYSTEM/ppsspp.ini"
grep -q '^DisplayAspectRatio = 1.000000$' "$ini" || fail "the picture's shape was not set"
grep -q '^UIScaleFactor = -4$' "$ini" || fail "the menus were not made smaller"
grep -q '^iShowStatusFlags = 0$' "$ini" || fail "the frame counter was left on"
grep -q '^Other = 1$' "$ini" || fail "a setting that was not ours changed"
grep -q "mount --bind $TMP/card/Saves/PSP $TMP/card/System/psp/.config/ppsspp/PSP/SAVEDATA" "$TMP/mounts" ||
	fail "the saves were not put in the card's Saves/PSP"
grep -q "mount --bind $TMP/card/States/PSP " "$TMP/mounts" || fail "the states were not put in States/PSP"
[ "$(grep -c '^umount' "$TMP/mounts")" -ge 4 ] || fail "the saves were not let go of"
[ ! -e "$TMP/run/cartridge-volume" ] || fail "the volume file was left behind"

# settings made by hand are kept: the second run does not set them again
sed -i 's/^UIScaleFactor = -4$/UIScaleFactor = -2/' "$ini"
psp run "$TMP/card/Games/PSP/Racing Game.cso"
grep -q '^UIScaleFactor = -2$' "$ini" || fail "a setting the player made was set again"

# no game, no run
setup
if psp run "$TMP/card/Games/PSP/Missing.cso" 2>/dev/null; then
	fail "a game that is not there was run"
fi

echo "psp: ok"
