#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/System/psp.sh"
TMP="$(mktemp -d)"
trap 'pkill -f "${TMP:?}" 2>/dev/null || true; rm -rf "${TMP:?}"' EXIT HUP INT TERM

fail() {
	echo "psp: $*" >&2
	exit 1
}

CARD="$TMP/card"
PSPDIR="$CARD/System/psp"
INI="$PSPDIR/.config/ppsspp/PSP/SYSTEM/ppsspp.ini"
CONTROLS="$PSPDIR/.config/ppsspp/PSP/SYSTEM/controls.ini"

setup() {
	rm -rf "${CARD:?}" "${TMP:?}/run" "${TMP:?}/bin" "${TMP:?}/battery"
	mkdir -p "$CARD/Config" "$CARD/Games/PSP" "$PSPDIR/.config/ppsspp/PSP/SYSTEM" "$TMP/run" "$TMP/bin" \
		"$TMP/battery/BAT0"
	printf 'cart=Racing Game\ncart_platform=psp\nvolume=15\nvolume_hp=60\ncheevos=1\n' > "$CARD/Config/cartridge.state"
	printf 'username=tester\ntoken=sekrit-token\n' > "$CARD/Config/retroachievements.txt"
	: > "$TMP/gpio"
	: > "$CARD/Games/PSP/Racing Game.cso"
	printf '[General]\nStateSlot = 0\nAutoLoadSaveState = 0\n[Graphics]\nDisplayAspectRatio = 0.848000\nUIScaleFactor = 0\niShowStatusFlags = 3\nOther = 1\n[Achievements]\nAchievementsEnable = True\nAchievementsChallengeMode = True\nAchievementsUserName = \n' > "$INI"
	printf '[ControlMapping]\nUp = 10-19\nPause = 10-109\n' > "$CONTROLS"
	: > "$TMP/hotkeys.log"
	# the emulator writes down what it was run with, and stays until it is told to leave
	cat > "$PSPDIR/PPSSPPSDL" <<EOT
#!/bin/sh
echo "\$*" > "$TMP/emulator-args"
echo "\${XDG_CONFIG_HOME:-}" > "$TMP/emulator-home"
echo "\${SDL_EVDEV_DEVICES:-}" > "$TMP/emulator-evdev"
sleep \${EMULATOR_SECONDS:-1}
EOT
	# the keyboard: every word it is told is noted, a save writes a state, a leave ends the emulator
	cat > "$PSPDIR/hotkeys" <<EOT
#!/bin/sh
while read -r word; do
	echo "\$word" >> "$TMP/hotkeys.log"
	case "\$word" in
	save) mkdir -p "$CARD/States/PSP"; : > "$CARD/States/PSP/ULUS10311_1.00_4.ppst" ;;
	exit) pkill PPSSPPSDL ;;
	esac
done
EOT
	for c in mount umount; do
		printf '#!/bin/sh\necho "%s $*" >> "%s"\n' "$c" "$TMP/mounts" > "$TMP/bin/$c"
	done
	chmod 755 "$TMP/bin/mount" "$TMP/bin/umount" "$PSPDIR/PPSSPPSDL" "$PSPDIR/hotkeys"
	: > "$TMP/mounts"
	printf '#!/bin/sh\n' > "$TMP/evtest"
	chmod 755 "$TMP/evtest"
	printf 'Battery\n' > "$TMP/battery/BAT0/type"
	printf '80\n' > "$TMP/battery/BAT0/capacity"
	printf 'Discharging\n' > "$TMP/battery/BAT0/status"
}

# `evtest_for NODE LINES...` makes the evtest that stands in for the real one print LINES, each
# `<delay> <code> <name> <value>`, when it is asked about NODE, and nothing for any other
evtest_for() {
	node="$1"
	shift
	{
		echo '#!/bin/sh'
		echo "[ \"\$1\" = $node ] || exit 0"
		for line in "$@"; do
			set -- $line
			echo "sleep $1"
			echo "echo 'Event: time 1.1, type 1 (EV_KEY), code $2 ($3), value $4'"
		done
		echo "sleep 6"
	} > "$TMP/evtest"
	chmod 755 "$TMP/evtest"
}

psp() {
	PATH="$TMP/bin:$PATH" CARTRIDGE_ROOT="$CARD" AGS_RUN="$TMP/run" AGS_GPIO="$TMP/gpio" \
		AGS_EVTEST="$TMP/evtest" AGS_FB=/dev/null AGS_KEYS=keys-node AGS_POWER_KEYS=power-node \
		AGS_CARTRIDGE=/nonexistent AGS_HOLD_S=1 AGS_BATTERY_EVERY=1 AGS_POWER_SUPPLY="$TMP/battery" \
		sh "$SCRIPT" "$@"
}

# the stand-in emulator is ended with a signal when it is told to leave: that is not a failure
emu() {
	psp run "$CARD/Games/PSP/Racing Game.cso" "$@" || true
}

words() {
	tr '\n' ' ' < "$TMP/hotkeys.log"
}

# the volume keys: each press moves the speaker's level five, and says so
setup
evtest_for keys-node "0 115 KEY_VOLUMEUP 1" "0 115 KEY_VOLUMEUP 0" "0 115 KEY_VOLUMEUP 1" "0 114 KEY_VOLUMEDOWN 1"
sed -i 's/^sleep 6$//' "$TMP/evtest"
psp keys
[ "$(cat "$TMP/run/cartridge-volume")" = 20 ] || fail "the level is $(cat "$TMP/run/cartridge-volume"), not 20"
grep -q '^volume=20$' "$CARD/Config/cartridge.state" || fail "the speaker's level was not kept"
grep -q '^volume_hp=60$' "$CARD/Config/cartridge.state" || fail "the headphones' level changed"
[ "$(cat "$TMP/run/cartridge-hud")" = "20 volume" ] || fail "the gauge was told $(cat "$TMP/run/cartridge-hud")"

# with headphones in the jack the keys move the headphones' level, and the gauge wears them
setup
echo ' gpio-259 (                    |Headphone detection ) in  hi ' > "$TMP/gpio"
evtest_for keys-node "0 115 KEY_VOLUMEUP 1"
sed -i 's/^sleep 6$//' "$TMP/evtest"
psp keys
grep -q '^volume_hp=65$' "$CARD/Config/cartridge.state" || fail "the headphones' level was not moved"
grep -q '^volume=15$' "$CARD/Config/cartridge.state" || fail "the speaker's level changed"
[ "$(cat "$TMP/run/cartridge-hud")" = "65 headphones" ] || fail "the gauge was told $(cat "$TMP/run/cartridge-hud")"

# the level never leaves 0..100
setup
printf 'volume=98\nvolume_hp=60\n' > "$CARD/Config/cartridge.state"
evtest_for keys-node "0 115 KEY_VOLUMEUP 1" "0 115 KEY_VOLUMEUP 1" "0 115 KEY_VOLUMEUP 1"
sed -i 's/^sleep 6$//' "$TMP/evtest"
psp keys
[ "$(cat "$TMP/run/cartridge-volume")" = 100 ] || fail "the level went past 100"

# running a game: the emulator gets the game, its menu's exit ends it, its folder is found
# through XDG_CONFIG_HOME, its settings are set once, its saves are the card's, and the game
# is out of the slot when it is left
setup
emu
grep -q -- "Racing Game.cso --fullscreen --pause-menu-exit" "$TMP/emulator-args" ||
	fail "the emulator was run with: $(cat "$TMP/emulator-args")"
[ "$(cat "$TMP/emulator-home")" = "$PSPDIR/.config" ] || fail "XDG_CONFIG_HOME is wrong"
grep -q '^DisplayAspectRatio = 1.000000$' "$INI" || fail "the picture's shape was not set"
grep -q '^UIScaleFactor = -4$' "$INI" || fail "the menus were not made smaller"
grep -q '^iShowStatusFlags = 0$' "$INI" || fail "the frame counter was left on"
grep -q '^Other = 1$' "$INI" || fail "a setting that was not ours changed"
grep -q "mount --bind $CARD/Saves/PSP $PSPDIR/.config/ppsspp/PSP/SAVEDATA" "$TMP/mounts" ||
	fail "the saves were not put in the card's Saves/PSP"
grep -q "mount --bind $CARD/States/PSP " "$TMP/mounts" || fail "the states were not put in States/PSP"
[ "$(grep -c '^umount' "$TMP/mounts")" -ge 4 ] || fail "the saves were not let go of"
[ ! -e "$TMP/run/cartridge-volume" ] || fail "the volume file was left behind"
grep -q '^cart=$' "$CARD/Config/cartridge.state" || fail "the game was left in the slot after a plain exit"
grep -q '^cart_platform=$' "$CARD/Config/cartridge.state" || fail "the system was left in the slot"
[ ! -e "$TMP/run/cartridge-poweroff" ] || fail "a plain exit asked for a power-off"

# the settings every run: the slot a state is saved to, the newest state loaded as a game
# starts, and the keys the hotkeys keyboard types on
grep -q '^StateSlot = 4$' "$INI" || fail "the state slot was not set"
grep -q '^AutoLoadSaveState = 2$' "$INI" || fail "the newest state is not loaded"
grep -q '^Pause = 1-111$' "$CONTROLS" || fail "the pause menu is not on Escape (and MENU no longer on it)"
grep -q '^Save State = 1-132$' "$CONTROLS" || fail "no key saves a state"
grep -q '^Load State = 1-134$' "$CONTROLS" || fail "no key loads a state"
grep -q '^Exit App = 1-140$' "$CONTROLS" || fail "no key leaves"
grep -q '^Up = 10-19$' "$CONTROLS" || fail "a mapping that was not ours changed"

# RetroAchievements: the player's own sign-in, softcore, when cartridge has them on
grep -q '^AchievementsEnable = True$' "$INI" || fail "the achievements were not turned on"
grep -q '^AchievementsChallengeMode = False$' "$INI" || fail "hardcore was left on"
grep -q '^AchievementsUserName = tester$' "$INI" || fail "the player's name was not set"
grep -q '^AchievementsToken = sekrit-token$' "$INI" || fail "the player's token was not given"

# settings made by hand are kept: the second run does not set them again
sed -i 's/^UIScaleFactor = -4$/UIScaleFactor = -2/' "$INI"
emu
grep -q '^UIScaleFactor = -2$' "$INI" || fail "a setting the player made was set again"

setup
sed -i 's/^cheevos=1$/cheevos=0/' "$CARD/Config/cartridge.state"
emu
grep -q '^AchievementsEnable = False$' "$INI" || fail "the achievements stayed on with cartridge's off"
grep -q '^AchievementsToken = $' "$INI" || fail "the token stayed in the settings"
setup
rm "$CARD/Config/retroachievements.txt"
emu
grep -q '^AchievementsEnable = False$' "$INI" || fail "the achievements were on with no sign-in"

# a tap of MENU opens the pause menu
setup
evtest_for keys-node "0.3 312 BTN_TL2 1" "0 312 BTN_TL2 0"
EMULATOR_SECONDS=2 emu
grep -qx pause "$TMP/hotkeys.log" || fail "a tap of MENU did not open the pause menu: $(words)"
grep -qx save "$TMP/hotkeys.log" && fail "a tap of MENU saved"

# a hold of MENU saves, then leaves, though the key is let go while the save is still under way; the state is on the card by then, and the game is out of
# the slot, and the pause menu stays shut
setup
evtest_for keys-node "0.3 312 BTN_TL2 1" "1.5 312 BTN_TL2 0"
EMULATOR_SECONDS=30 emu
[ "$(words)" = "save exit " ] || fail "a hold of MENU typed: $(words)"
[ -f "$CARD/States/PSP/ULUS10311_1.00_4.ppst" ] || fail "no state was saved"
grep -q '^cart=$' "$CARD/Config/cartridge.state" || fail "the game was left in the slot after leaving"

# the power button, held, saves and leaves with a note to power off, though it is let go while
# the save is still under way; the game stays in the slot
setup
evtest_for power-node "0.3 116 KEY_POWER 1" "1.5 116 KEY_POWER 0"
EMULATOR_SECONDS=30 emu
[ "$(words)" = "save exit " ] || fail "the power button typed: $(words)"
[ -f "$TMP/run/cartridge-poweroff" ] || fail "no note to power off was left"
grep -q '^cart=Racing Game$' "$CARD/Config/cartridge.state" || fail "the game left the slot at a power-off"

# a power button let go of at once is a tap, which does nothing
setup
evtest_for power-node "0.3 116 KEY_POWER 1" "0 116 KEY_POWER 0"
EMULATOR_SECONDS=3 emu
[ ! -s "$TMP/hotkeys.log" ] || fail "a tap of the power button typed: $(words)"
[ ! -e "$TMP/run/cartridge-poweroff" ] || fail "a tap of the power button asked for a power-off"

# an empty battery saves and leaves the same way; one that is charging does not
setup
printf '3\n' > "$TMP/battery/BAT0/capacity"
EMULATOR_SECONDS=30 emu
[ "$(words)" = "save exit " ] || fail "an empty battery typed: $(words)"
[ -f "$TMP/run/cartridge-poweroff" ] || fail "no note to power off at an empty battery"
setup
printf '3\n' > "$TMP/battery/BAT0/capacity"
printf 'Charging\n' > "$TMP/battery/BAT0/status"
EMULATOR_SECONDS=3 emu
[ ! -s "$TMP/hotkeys.log" ] || fail "a charging battery was left"

# the handheld's keys and power button are found by what they have, not by where they are
setup
cat > "$TMP/devices" <<EOT
I: Bus=0019 Vendor=0001 Product=0001 Version=0100
N: Name="power"
H: Handlers=kbd event0
B: KEY=10000000000000 0

I: Bus=0019 Vendor=0001 Product=0001 Version=0100
N: Name="handheld"
H: Handlers=kbd js0 event4
B: KEY=1000000000000 0 0 8000000000000 0

EOT
cat > "$TMP/evtest" <<EOT
#!/bin/sh
echo "\$1" >> "$TMP/nodes-followed"
EOT
: > "$TMP/nodes-followed"
PATH="$TMP/bin:$PATH" CARTRIDGE_ROOT="$CARD" AGS_RUN="$TMP/run" AGS_GPIO="$TMP/gpio" AGS_EVTEST="$TMP/evtest" \
	AGS_DEVICES="$TMP/devices" sh "$SCRIPT" keys
grep -qx /dev/input/event4 "$TMP/nodes-followed" || fail "the handheld's keys were not found: $(cat "$TMP/nodes-followed")"
: > "$TMP/nodes-followed"
(PATH="$TMP/bin:$PATH" CARTRIDGE_ROOT="$CARD" AGS_RUN="$TMP/run" AGS_GPIO="$TMP/gpio" AGS_EVTEST="$TMP/evtest" \
	AGS_DEVICES="$TMP/devices" AGS_BATTERY_EVERY=30 AGS_POWER_SUPPLY="$TMP/battery" timeout 3 sh "$SCRIPT" power || true)
grep -qx /dev/input/event0 "$TMP/nodes-followed" || fail "the power button was not found: $(cat "$TMP/nodes-followed")"

# the keyboard hotkeys made is named to the emulator's SDL, which has no udev to find it with
setup
cat > "$TMP/devices" <<EOT
I: Bus=0019 Vendor=0001 Product=0001 Version=0100
N: Name="handheld"
H: Handlers=kbd js0 event1
B: KEY=1000000000000 0 0 8000000000000 0

I: Bus=0006 Vendor=1209 Product=ca47 Version=0000
N: Name="cartridge-hotkeys"
H: Handlers=kbd event7
B: KEY=40000000000400 0

EOT
PATH="$TMP/bin:$PATH" CARTRIDGE_ROOT="$CARD" AGS_RUN="$TMP/run" AGS_GPIO="$TMP/gpio" AGS_EVTEST="$TMP/evtest" \
	AGS_FB=/dev/null AGS_KEYS=keys-node AGS_POWER_KEYS=power-node AGS_CARTRIDGE=/nonexistent \
	AGS_POWER_SUPPLY="$TMP/battery" AGS_DEVICES="$TMP/devices" \
	sh "$SCRIPT" run "$CARD/Games/PSP/Racing Game.cso" || true
[ "$(cat "$TMP/emulator-evdev")" = "2:/dev/input/event7" ] ||
	fail "the emulator was told the keyboard is $(cat "$TMP/emulator-evdev")"

# no game, no run
setup
if psp run "$CARD/Games/PSP/Missing.cso" 2>/dev/null; then
	fail "a game that is not there was run"
fi

echo "psp: ok"
