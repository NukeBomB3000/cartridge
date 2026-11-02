#!/bin/sh
# Runs PPSSPP, the stand-alone PSP emulator, on a game, and gives it what cartridge gives a
# libretro core: the volume keys, with the gauge on the screen, and a clean framebuffer
# afterwards. Verbs:
#
#   run GAME [OPTION...]   run PPSSPP from System/psp on GAME, with options for it, and wait
#                          for it to end
#   volume     follow the volume keys until killed (run starts it; the loop is its own verb so it
#              can be tried without the emulator)
#
# cartridge's own volume is software gain on the samples it plays, which a program of its own does
# not go through. The SDL that System/psp ships (its Mali video driver and its audio thread were
# changed for this, see scripts/build-psp.sh) scales what the program plays by the square of the
# level in $VOLUME, as cartridge does, and draws the volume gauge for a moment after $HUD is
# written: the level and the icon, in the faces `cartridge --hud-faces` drew in the colours of the
# card's backdrop, so the gauge looks as it does over a game of cartridge's own.
set -u

SD="${CARTRIDGE_ROOT:-/mnt/sdcard}"
SYS="${AGS_SYS:-$SD/System}"
PSP="${AGS_PSP:-$SYS/psp}"
STATE="${AGS_STATE:-$SD/Config/cartridge.state}"
RUN="${AGS_RUN:-/run}"
HUD="${AGS_HUD:-$RUN/cartridge-hud}"
VOLUME="${AGS_VOLUME:-$RUN/cartridge-volume}"
EVTEST="${AGS_EVTEST:-evtest}"
KEYS="${AGS_KEYS:-/dev/input/event1}"
GPIO="${AGS_GPIO:-/sys/kernel/debug/gpio}"
FB="${AGS_FB:-/dev/fb0}"
FACES="${AGS_FACES:-$RUN/cartridge-hud-faces}"
CARTRIDGE="${AGS_CARTRIDGE:-$SYS/cartridge}"
LOADER="${AGS_LOADER:-/lib/ld-linux-aarch64.so.1}"
STEP=5

# The level of the volume, 0..100, the way cartridge keeps it: one for the speaker and one for
# headphones in the jack.
state_key() {
	if grep "|Headphone detection" "$GPIO" 2>/dev/null | sed 's/.*)//' | grep -q ' hi'; then
		echo volume_hp
	else
		echo volume
	fi
}

level() {
	sed -n "s/^$1=//p" "$STATE" 2>/dev/null | head -n 1
}

volume_loop() {
	key="$(state_key)"
	now="$(level "$key")"
	[ -n "$now" ] || now=50
	echo "$now" > "$VOLUME"
	$EVTEST "$KEYS" 2>/dev/null | while read -r line; do
		case "$line" in
		*"code 115 "*"value 1" | *"code 115 "*"value 2") delta=$STEP ;;
		*"code 114 "*"value 1" | *"code 114 "*"value 2") delta=-$STEP ;;
		*) continue ;;
		esac
		key="$(state_key)"
		now="$(level "$key")"
		[ -n "$now" ] || now=50
		now=$((now + delta))
		[ "$now" -lt 0 ] && now=0
		[ "$now" -gt 100 ] && now=100
		echo "$now" > "$VOLUME"
		if [ -f "$STATE" ] && grep -q "^$key=" "$STATE"; then
			sed -i "s/^$key=.*/$key=$now/" "$STATE"
		fi
		case "$key" in
		volume_hp) icon=headphones ;;
		*) icon=volume ;;
		esac
		echo "$now $icon" > "$HUD"
	done
}

# PPSSPP's own settings, set once on the handheld they are for: the picture as big as the panel
# lets it be at the PSP's own shape (the pak's value is for the Trimui Brick's 4:3 panel), its
# menus smaller, and no frame counter.
configure() {
	ini="$PSP/.config/ppsspp/PSP/SYSTEM/ppsspp.ini"
	[ -f "$ini" ] && [ ! -f "$PSP/.configured" ] || return 0
	sed -i \
		-e 's/^DisplayAspectRatio = .*/DisplayAspectRatio = 1.000000/' \
		-e 's/^UIScaleFactor = .*/UIScaleFactor = -4/' \
		-e 's/^iShowStatusFlags = .*/iShowStatusFlags = 0/' \
		"$ini"
	: > "$PSP/.configured"
}

# Its saves and save states belong with the card's other games', not in the emulator's folder,
# which is what a sync of System/ replaces.
bind_saves() {
	for pair in "Saves/PSP:SAVEDATA" "States/PSP:PPSSPP_STATE"; do
		from="$SD/${pair%%:*}"
		to="$PSP/.config/ppsspp/PSP/${pair##*:}"
		mkdir -p "$from" "$to"
		umount "$to" 2>/dev/null
		mount --bind "$from" "$to" 2>/dev/null || echo "psp.sh: $from is not the saves' place" >&2
	done
}

unbind_saves() {
	for dir in SAVEDATA PPSSPP_STATE; do
		umount "$PSP/.config/ppsspp/PSP/$dir" 2>/dev/null
	done
}

# What SDL needs to take the handheld's buttons for a pad's.
pad_mapping() {
	[ -x "$PSP/padmap" ] && [ -e /dev/input/js0 ] || return 0
	LD_LIBRARY_PATH="$PSP:/usr/lib" "$PSP/padmap" /dev/input/js0 2>/dev/null
}

run_game() {
	game="$1"
	shift
	[ -f "$game" ] || { echo "psp.sh: no game $game" >&2; exit 1; }
	[ -x "$PSP/PPSSPPSDL" ] || { echo "psp.sh: no emulator in $PSP" >&2; exit 1; }
	configure
	bind_saves
	# the frontend's own gauge, drawn once for the driver to read
	if [ -f "$CARTRIDGE" ]; then
		CARTRIDGE_ROOT="$SD" "$LOADER" "$CARTRIDGE" --hud-faces "$FACES" 2>/dev/null
	fi
	sh "$0" volume &
	volume=$!
	(
		cd "$PSP" || exit 1
		# HOME stays what it is, so ALSA finds the ~/.asoundrc audioroute.sh keeps; the emulator's
		# own folder is found by XDG_CONFIG_HOME
		export XDG_CONFIG_HOME="$PSP/.config" SDL_VIDEODRIVER=mali LD_LIBRARY_PATH="$PSP:/usr/lib"
		export SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1 SDL_JOYSTICK_DEVICE=/dev/input/js0
		mapping="$(pad_mapping)"
		[ -n "$mapping" ] && export SDL_GAMECONTROLLERCONFIG="$mapping"
		./PPSSPPSDL "$game" --fullscreen --pause-menu-exit "$@"
	)
	status=$?
	kill "$volume" 2>/dev/null
	pkill -f "$EVTEST $KEYS" 2>/dev/null
	unbind_saves
	rm -f "$HUD" "$VOLUME" "$FACES"
	# what the emulator last drew would stay on the screen until cartridge's first frame
	head -c 33554432 /dev/zero > "$FB" 2>/dev/null
	return $status
}

case "${1:-}" in
run)
	game="${2:-}"
	[ "$#" -ge 2 ] && shift 2
	run_game "$game" "$@"
	;;
volume) volume_loop ;;
*)
	echo "usage: psp.sh run GAME [OPTION...] | volume" >&2
	exit 2
	;;
esac
