#!/bin/sh
set -u

SD="${AGS_CARD:-${CARTRIDGE_ROOT:-/mnt/sdcard}}"
ETC="${AGS_ETC:-/etc}"
VENDOR="${AGS_VENDOR:-/mnt/vendor}"
SYSFS="${AGS_SYSFS:-/sys}"
PROC="${AGS_PROC:-/proc}"
DEBUGFS="${AGS_DEBUGFS:-/sys/kernel/debug}"
DEV="${AGS_DEV:-/dev}"
NET_WAIT_S="${AGS_PROBE_NET_WAIT_S:-0}"

section() {
	printf '\n## %s\n' "$1"
}

value() {
	[ -f "$1" ] && [ -r "$1" ] || return 0
	printf '%s = %s\n' "$1" "$(head -c 512 "$1" 2>/dev/null | tr '\n' ' ' | sed 's/ *$//')"
}

dump() {
	[ -f "$1" ] && [ -r "$1" ] || return 0
	printf '%s\n' "--- $1"
	cat "$1" 2>/dev/null
}

cmd() {
	if command -v "$1" > /dev/null 2>&1; then
		printf '%s\n' "\$ $*"
		"$@" 2>&1
	else
		printf '%s\n' "\$ $1 (not found)"
	fi
}

# Where each tool is, or that it is missing.
where_are() {
	for b in "$@"; do
		if command -v "$b" > /dev/null 2>&1; then
			printf '%s = %s\n' "$b" "$(command -v "$b")"
		else
			printf '%s = not found\n' "$b"
		fi
	done
}

values_in() {
	[ -d "$1" ] || return 0
	for f in "$1"/*; do
		value "$f"
	done
}

target() {
	t="$(sed -n 's/^BASEOS_TARGET=//p' "$ETC/baseos-release" 2>/dev/null | tr -d '"' | head -n 1)"
	echo "${t:-unknown}"
}

identity() {
	section identity
	dump "$ETC/baseos-release"
	if [ -r "$VENDOR/bin/dmenu.bin" ]; then
		printf 'dmenu.bin model = %s\n' \
			"$(tr -c '[:print:]' '\n' < "$VENDOR/bin/dmenu.bin" | grep -m 1 '^RG')"
	fi
	value "$PROC/device-tree/model"
	for f in "$PROC"/device-tree/soc*/lcd0*/lcd_driver_name; do
		value "$f"
	done
	cmd uname -a
	value "$PROC/meminfo"
}

display() {
	section display
	for f in mode modes virtual_size bits_per_pixel stride rotate blank; do
		value "$SYSFS/class/graphics/fb0/$f"
	done
	for b in "$SYSFS"/class/backlight/*; do
		values_in "$b"
	done
	values_in "$SYSFS/class/disp/disp/attr"
	[ -e "$DEV/disp" ] && echo "$DEV/disp present"
	[ -e "$DEV/mali0" ] && echo "$DEV/mali0 present"
	return 0
}

input() {
	section input
	dump "$PROC/bus/input/devices"
	for e in "$SYSFS"/class/input/event*; do
		[ -d "$e" ] || continue
		printf '%s\n' "--- ${e##*/}"
		value "$e/device/name"
		for c in ev key abs sw ff; do
			value "$e/device/capabilities/$c"
		done
		value "$e/device/poll"
	done
}

power() {
	section power
	for p in "$SYSFS"/class/power_supply/*; do
		[ -d "$p" ] || continue
		printf '%s\n' "--- ${p##*/}"
		values_in "$p"
	done
	section leds
	for l in "$SYSFS"/class/leds/*; do
		[ -d "$l" ] || continue
		printf '%s\n' "--- ${l##*/}"
		for f in brightness max_brightness multi_index multi_intensity trigger; do
			value "$l/$f"
		done
	done
	section rtc
	for r in "$SYSFS"/class/rtc/*; do
		value "$r/name"
	done
}

gpio() {
	section gpio
	dump "$DEBUGFS/gpio"
}

audio() {
	section audio
	dump "$PROC/asound/cards"
	cmd aplay -l
	cmd amixer -c 0 scontrols
	dump "$ETC/asound.conf"
}

network() {
	section network
	w=0
	while [ ! -d "$SYSFS/class/net/wlan0" ] && [ "$w" -lt "$NET_WAIT_S" ]; do
		sleep 1
		w=$((w + 1))
	done
	ls "$SYSFS/class/net" 2>/dev/null
	cmd ip link
	cmd iw dev
	cmd iw list
	cmd rfkill list
	where_are wpa_supplicant wpa_cli udhcpc ntpd timedatectl dropbear
	cmd wpa_supplicant -v
	printf 'hostname = %s\n' "$(cat "$PROC/sys/kernel/hostname" 2>/dev/null)"
	dump "$PROC/modules"
}

bluetooth() {
	section bluetooth
	where_are rtk_hciattach hciattach bluetoothd bluetoothctl bluealsa bluealsa-aplay dbus-daemon
	for d in uhid uinput; do
		[ -e "$DEV/$d" ] && echo "$DEV/$d present"
	done
	grep -E '^(hidp|uhid|rtl_btlpm|bluetooth|rfcomm) ' "$PROC/modules" 2>/dev/null
	return 0
}

card() {
	section card
	printf 'SD = %s\n' "$SD"
	ls -la "$SD/System" 2>&1
}

report() {
	printf '# cartridge probe %s\n' "$(date '+%F %T' 2>/dev/null)"
	identity
	display
	input
	power
	gpio
	audio
	network
	bluetooth
	card
}

out="${1:-}"
if [ "$out" = - ]; then
	report
	exit 0
fi
if [ -z "$out" ]; then
	out="$SD/probe-$(target)-$(date '+%Y%m%d-%H%M%S' 2>/dev/null || echo now).txt"
fi
report > "$out.tmp" 2>&1 && mv -f "$out.tmp" "$out"
sync 2>/dev/null
echo "$out"
