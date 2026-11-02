#!/bin/sh
# Runs PPSSPP, the stand-alone PSP emulator, on a game, and gives it what cartridge gives a
# libretro core: the volume keys, with the gauge on the screen, MENU held to leave the game, the
# game saved as it is left and loaded as it is come back to, the power button, the battery and
# the player's RetroAchievements. Verbs:
#
#   run GAME [OPTION...]   run PPSSPP from System/psp on GAME, with options for it, and wait
#                          for it to end
#   keys                   follow the handheld's keys until killed (run starts it; the loop is its
#                          own verb so it can be tried without the emulator)
#   power                  follow the power button and the battery the same way
#
# cartridge's own volume is software gain on the samples it plays, which a program of its own does
# not go through. The SDL that System/psp ships (its Mali video driver and its audio thread were
# changed for this, see scripts/build-psp.sh) scales what the program plays by the square of the
# level in $VOLUME, as cartridge does, and draws the volume gauge for a moment after $HUD is
# written: the level and the icon, in the faces `cartridge --hud-faces` drew in the colours of the
# card's backdrop, so the gauge looks as it does over a game of cartridge's own.
#
# PPSSPP takes its hotkeys from a keyboard only, so `hotkeys` makes one that is not there and
# types on it what this script asks: a tap of MENU the pause menu (Escape), a hold of it a state
# saved to the last slot (F2) and then PPSSPP left (F10), and the next start loads the newest
# state by itself. Holding the power button or a battery that runs out saves and leaves the same
# way, and has cartridge power the handheld off when it is back, with the game still in the
# slot, so that the next start goes into it.
set -u

SD="${CARTRIDGE_ROOT:-/mnt/sdcard}"
SYS="${AGS_SYS:-$SD/System}"
PSP="${AGS_PSP:-$SYS/psp}"
STATE="${AGS_STATE:-$SD/Config/cartridge.state}"
RUN="${AGS_RUN:-/run}"
HUD="${AGS_HUD:-$RUN/cartridge-hud}"
VOLUME="${AGS_VOLUME:-$RUN/cartridge-volume}"
FIFO="${AGS_FIFO:-$RUN/cartridge-hotkeys}"
POWEROFF="${AGS_POWEROFF:-$RUN/cartridge-poweroff}"
HELD="${AGS_HELD:-$RUN/cartridge-psp-held}"
EVTEST="${AGS_EVTEST:-evtest}"
HOTKEYS="${AGS_HOTKEYS:-$PSP/hotkeys}"
DEVICES="${AGS_DEVICES:-/proc/bus/input/devices}"
GPIO="${AGS_GPIO:-/sys/kernel/debug/gpio}"
POWER_SUPPLY="${AGS_POWER_SUPPLY:-/sys/class/power_supply}"
FB="${AGS_FB:-/dev/fb0}"
FACES="${AGS_FACES:-$RUN/cartridge-hud-faces}"
CARTRIDGE="${AGS_CARTRIDGE:-$SYS/cartridge}"
LOADER="${AGS_LOADER:-/lib/ld-linux-aarch64.so.1}"
STATES="${AGS_STATES:-$SD/States/PSP}"
INI="$PSP/.config/ppsspp/PSP/SYSTEM/ppsspp.ini"
CONTROLS="$PSP/.config/ppsspp/PSP/SYSTEM/controls.ini"
STEP=5
# how long a key is held to count as held: cartridge's own (MENU_HOLD_MS, POWER_HOLD_MS)
HOLD_S="${AGS_HOLD_S:-1}"
# the battery's level at which the handheld saves and goes off, as cartridge does
BATTERY_CRITICAL="${AGS_BATTERY_CRITICAL:-5}"
BATTERY_EVERY="${AGS_BATTERY_EVERY:-20}"

# The /dev/input node of the input device that has the key 115 (volume up) and the handheld's A
# button, for "keys", or the key 116 (power) and no A button, for "power": the handheld's own
# keys and its power button, whatever the handheld's driver calls them and wherever it puts them;
# and, for "hotkeys", the keyboard `hotkeys` made.
find_node() {
	awk -v want="$1" '
	function bit(words, n,    h, p, c, v) {
		h = words[int(n / 64) + 1]
		p = int((n % 64) / 4)
		if (p >= length(h)) return 0
		c = substr(h, length(h) - p, 1)
		v = index("0123456789abcdef", tolower(c)) - 1
		return int(v / (2 ^ (n % 4))) % 2
	}
	function done_block(    i, f, words, a) {
		if (node != "" && want == "hotkeys" && name == "cartridge-hotkeys") { print node; found = 1; exit }
		if (node != "" && nkeys > 0) {
			split(keys, f, " ")
			for (i = 1; i <= nkeys; i++) words[i] = f[nkeys - i + 1]
			a = bit(words, 304)
			if (want == "keys" && bit(words, 115) && a) { print node; found = 1; exit }
			if (want == "power" && bit(words, 116) && !a) { print node; found = 1; exit }
		}
		node = ""; keys = ""; nkeys = 0; name = ""
	}
	/^$/ { done_block(); next }
	/^N: Name=/ { name = $0; sub(/^N: Name="/, "", name); sub(/"$/, "", name) }
	/^H: Handlers=/ { for (i = 1; i <= NF; i++) if ($i ~ /^event[0-9]+$/) node = "/dev/input/" $i }
	/^B: KEY=/ { sub(/^B: KEY=/, ""); keys = $0; nkeys = split($0, g, " ") }
	END { if (!found) done_block() }
	' "$DEVICES" 2>/dev/null
}

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

# a key typed on the keyboard hotkeys made
hotkey() {
	[ -p "$FIFO" ] && echo "$1" > "$FIFO"
}

# The game saved to the slot PPSSPP is set to save to (the last of its five): waits until a
# state file is written, up to eight seconds.
autosave() {
	marker="$RUN/cartridge-psp-save"
	: > "$marker"
	hotkey save
	waited=0
	while [ "$waited" -lt 80 ]; do
		if [ -n "$(find "$STATES" -name '*.ppst' -newer "$marker" 2>/dev/null | head -n 1)" ]; then
			# written through, not begun
			sleep 1
			break
		fi
		sleep 0.1
		waited=$((waited + 1))
	done
	rm -f "$marker"
}

# Leaves the game: saved first, with `save`, and with `poweroff` leaving a note for cartridge to
# power the handheld off once it is back. PPSSPP is asked to leave, and ended when it does not.
leave() {
	[ "${1:-}" = save ] && autosave
	[ "${2:-}" = poweroff ] && : > "$POWEROFF"
	hotkey exit
	tries=0
	while [ "$tries" -lt 40 ] && pgrep PPSSPPSDL > /dev/null 2>&1; do
		sleep 0.25
		tries=$((tries + 1))
	done
	pkill PPSSPPSDL 2>/dev/null
	return 0
}

# The volume keys, and MENU: a tap is the pause menu, a hold, for HOLD_S seconds, leaves the game.
keys_loop() {
	node="${AGS_KEYS:-$(find_node keys)}"
	[ -n "$node" ] || { echo "psp.sh: no keys to follow" >&2; return 1; }
	key="$(state_key)"
	now="$(level "$key")"
	[ -n "$now" ] || now=50
	echo "$now" > "$VOLUME"
	rm -f "$HELD"
	timer=""
	$EVTEST "$node" 2>/dev/null | while read -r line; do
		case "$line" in
		*"code 115 "*"value 1" | *"code 115 "*"value 2") delta=$STEP ;;
		*"code 114 "*"value 1" | *"code 114 "*"value 2") delta=-$STEP ;;
		*"code 312 "*"value 1")
			rm -f "$HELD"
			( sleep "$HOLD_S"; : > "$HELD"; leave save ) &
			timer=$!
			continue
			;;
		*"code 312 "*"value 0")
			if [ -f "$HELD" ]; then
				rm -f "$HELD"
			else
				[ -n "$timer" ] && kill "$timer" 2>/dev/null
				hotkey pause
			fi
			timer=""
			continue
			;;
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

# The power button, held, saves and has the handheld power off; and the battery, as it runs out.
power_loop() {
	node="${AGS_POWER_KEYS:-$(find_node power)}"
	if [ -n "$node" ]; then
		(
			timer=""
			$EVTEST "$node" 2>/dev/null | while read -r line; do
				case "$line" in
				*"code 116 "*"value 1")
					( sleep "$HOLD_S"; leave save poweroff ) &
					timer=$!
					;;
				*"code 116 "*"value 0")
					[ -n "$timer" ] && kill "$timer" 2>/dev/null
					timer=""
					;;
				esac
			done
		) &
	fi
	while :; do
		sleep "$BATTERY_EVERY"
		for dir in "$POWER_SUPPLY"/*; do
			[ "$(cat "$dir/type" 2>/dev/null)" = Battery ] || continue
			percent="$(cat "$dir/capacity" 2>/dev/null)"
			status="$(cat "$dir/status" 2>/dev/null)"
			if [ "$status" = Discharging ] && [ -n "$percent" ] && [ "$percent" -le "$BATTERY_CRITICAL" ]; then
				leave save poweroff
				return 0
			fi
		done
	done
}

# Sets `$2` to `$3` in the settings file `$1`, adding it after the line `$4` where it is not
# there.
set_key() {
	file="$1"
	key="$2"
	value="$3"
	after="$4"
	[ -f "$file" ] || return 0
	if grep -q "^$key = " "$file"; then
		sed -i "s|^$key = .*|$key = $value|" "$file"
	else
		sed -i "/$after/a $key = $value" "$file"
	fi
}

# PPSSPP's own settings, set once on the handheld they are for: the picture as big as the panel
# lets it be at the PSP's own shape (the pak's value is for the Trimui Brick's 4:3 panel), its
# menus smaller, and no frame counter.
configure() {
	[ -f "$INI" ] && [ ! -f "$PSP/.configured" ] || return 0
	sed -i \
		-e 's/^DisplayAspectRatio = .*/DisplayAspectRatio = 1.000000/' \
		-e 's/^UIScaleFactor = .*/UIScaleFactor = -4/' \
		-e 's/^iShowStatusFlags = .*/iShowStatusFlags = 0/' \
		"$INI"
	: > "$PSP/.configured"
}

# What is set on every run: the slot a state is saved to, and that the newest state is loaded
# as a game starts, so that a game left saved comes back as it was left; the keys that take
# what `hotkeys` types; and the player's RetroAchievements, softcore as everywhere in cartridge.
every_run() {
	set_key "$INI" StateSlot 4 '\[General\]'
	set_key "$INI" AutoLoadSaveState 2 '\[General\]'
	if [ -f "$CONTROLS" ]; then
		for pair in "Pause:1-111" "Save State:1-132" "Load State:1-134" "Exit App:1-140"; do
			set_key "$CONTROLS" "${pair%%:*}" "${pair##*:}" 'ControlMapping'
		done
	fi
	creds="$SD/Config/retroachievements.txt"
	user=""
	token=""
	if [ "$(level cheevos)" != 0 ] && [ -f "$creds" ]; then
		user="$(sed -n 's/^username=//p' "$creds" | head -n 1 | tr -d '\r')"
		token="$(sed -n 's/^token=//p' "$creds" | head -n 1 | tr -d '\r')"
	fi
	if [ -n "$user" ] && [ -n "$token" ]; then
		set_key "$INI" AchievementsEnable True '\[Achievements\]'
		set_key "$INI" AchievementsChallengeMode False '\[Achievements\]'
		set_key "$INI" AchievementsUserName "$user" '\[Achievements\]'
		set_key "$INI" AchievementsToken "$token" '\[Achievements\]'
	else
		set_key "$INI" AchievementsEnable False '\[Achievements\]'
		set_key "$INI" AchievementsToken "" '\[Achievements\]'
	fi
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

# The cart is out of the slot, as it is when a game is ejected: cartridge starts on the shelf, at
# the game it left. A game left through a power-off stays in it, so that the next start goes
# into the game.
eject() {
	[ -f "$STATE" ] || return 0
	sed -i -e 's/^cart=.*/cart=/' -e 's/^cart_platform=.*/cart_platform=/' "$STATE"
}

run_game() {
	game="$1"
	shift
	[ -f "$game" ] || { echo "psp.sh: no game $game" >&2; exit 1; }
	[ -x "$PSP/PPSSPPSDL" ] || { echo "psp.sh: no emulator in $PSP" >&2; exit 1; }
	rm -f "$POWEROFF"
	configure
	every_run
	bind_saves
	# the frontend's own gauge, drawn once for the driver to read
	if [ -f "$CARTRIDGE" ]; then
		CARTRIDGE_ROOT="$SD" "$LOADER" "$CARTRIDGE" --hud-faces "$FACES" 2>/dev/null
	fi
	# the keyboard for the hotkeys, made before PPSSPP starts as SDL looks for devices only
	# once; this script holds the pipe open for it, so it never reads an end of its input
	hotkeys=""
	if [ -x "$HOTKEYS" ]; then
		rm -f "$FIFO"
		mkfifo "$FIFO"
		exec 9<> "$FIFO"
		"$HOTKEYS" < "$FIFO" > /dev/null 2>&1 &
		hotkeys=$!
		sleep 1
	fi
	sh "$0" keys &
	keys=$!
	sh "$0" power &
	power=$!
	(
		cd "$PSP" || exit 1
		# HOME stays what it is, so ALSA finds the ~/.asoundrc audioroute.sh keeps; the emulator's
		# own folder is found by XDG_CONFIG_HOME
		export XDG_CONFIG_HOME="$PSP/.config" SDL_VIDEODRIVER=mali LD_LIBRARY_PATH="$PSP:/usr/lib"
		export SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1 SDL_JOYSTICK_DEVICE=/dev/input/js0
		# this SDL has no udev to find the keyboard with: it is told which it is (2 is a keyboard)
		kbd="$(find_node hotkeys)"
		[ -n "$kbd" ] && export SDL_EVDEV_DEVICES="2:$kbd"
		mapping="$(pad_mapping)"
		[ -n "$mapping" ] && export SDL_GAMECONTROLLERCONFIG="$mapping"
		./PPSSPPSDL "$game" --fullscreen --pause-menu-exit "$@"
	)
	status=$?
	kill "$keys" "$power" $hotkeys 2>/dev/null
	pkill -f "$EVTEST" 2>/dev/null
	exec 9>&-
	rm -f "$FIFO" "$HELD"
	unbind_saves
	rm -f "$HUD" "$VOLUME" "$FACES"
	[ -f "$POWEROFF" ] || eject
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
keys) keys_loop ;;
power) power_loop ;;
*)
	echo "usage: psp.sh run GAME [OPTION...] | keys | power" >&2
	exit 2
	;;
esac
