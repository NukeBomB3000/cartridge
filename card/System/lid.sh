#!/bin/sh
# A watcher apart from omega-zero: if the lid stays closed a minute past the 3 minutes after which
# omega-zero powers off by itself, omega-zero has hung, and this powers off in its place so the
# battery is not run flat in a bag. slot's (#117), its lid time fixed at omega-zero's.
set -u

SD="${OMEGA_ROOT:-/mnt/sdcard}"
PSY="${AGS_PSY:-}"
TRACE="${AGS_TRACE:-$SD/omega-zero-session.log}"
CLOSED="${AGS_LID_CLOSED:-/run/omega-lid.closed}"
UPTIME="${AGS_UPTIME:-/proc/uptime}"
INTERVAL="${AGS_LID_INTERVAL:-5}"
MINUTES="${AGS_LID_MINUTES:-3}"
GRACE="${AGS_LID_GRACE:-60}"
RMMOD="${AGS_RMMOD:-rmmod}"
SYNC="${AGS_SYNC:-sync}"
POWEROFF="${AGS_POWEROFF:-poweroff}"

now() {
	read -r up _ < "$UPTIME" 2>/dev/null || up=0
	echo "${up%%.*}"
}

# the power supply that reports the lid, as omega-zero finds it
hallkey() {
	if [ -n "$PSY" ]; then
		echo "$PSY/hallkey"
		return
	fi
	for f in /sys/class/power_supply/*/hallkey; do
		[ -f "$f" ] && echo "$f" && return
	done
}

power_off() {
	echo "$(date '+%H:%M:%S') lid: closed ${1}s and omega-zero has not powered off, powering off" >> "$TRACE" 2>/dev/null
	$RMMOD 8821cs 2>/dev/null
	$SYNC
	$POWEROFF
}

check() {
	key="$(hallkey)"
	if [ -z "$key" ] || [ "$(cat "$key" 2>/dev/null)" != 0 ]; then
		rm -f "$CLOSED"
		return 1
	fi
	t="$(now)"
	if [ ! -s "$CLOSED" ]; then
		echo "$t" > "$CLOSED"
		return 1
	fi
	shut=$((t - $(cat "$CLOSED")))
	[ "$shut" -ge $((MINUTES * 60 + GRACE)) ] || return 1
	rm -f "$CLOSED"
	power_off "$shut"
}

case "${1:-loop}" in
once)
	check || true
	;;
loop)
	while :; do
		check && exit 0
		sleep "$INTERVAL"
	done
	;;
*)
	echo "usage: lid.sh [once|loop]" >&2
	exit 2
	;;
esac
