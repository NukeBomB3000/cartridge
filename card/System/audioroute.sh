#!/bin/sh
# Where ALSA's default output goes. cartridge plays on ALSA's default, so this decides by writing
# ~/.asoundrc, which cartridge looks at again as it opens the device:
#
#   Bluetooth headphones or a speaker connected (bluealsa has an A2DP stream for it): that
#   an HDMI monitor connected (hdmi.sh active): its sound
#   neither: a file that only says so, which leaves the handheld's own speaker as the default
#
# The file is never removed: ALSA only watches the files it found when it first read its
# configuration, so one that appears later would not be seen by a cartridge already running.
# Only a file this script wrote is ever replaced. Verbs:
#
#   sync   decide once and write the file when it has to change
#   loop   sync every two seconds, for as long as cartridge runs (launch_frontend.sh starts it)
#   where  print bluetooth:<MAC>, hdmi or speaker, and change nothing
set -u

SD="${CARTRIDGE_ROOT:-/mnt/sdcard}"
SYS="${AGS_SYS:-$SD/System}"
ASOUNDRC="${AGS_ASOUNDRC:-${HOME:-/root}/.asoundrc}"
BLUEALSA_APLAY="${AGS_BLUEALSA_APLAY:-bluealsa-aplay}"
TIMEOUT="${AGS_TIMEOUT:-timeout}"
MARK="# written by cartridge: audioroute.sh"
# what hdmi.sh wrote before this script did
OLD_MARK="# cartridge-hdmi"

# The address of a Bluetooth device bluealsa plays to: its PCM list names each connected one as
# `...DEV=<MAC>,PROFILE=a2dp`. Nothing while bluealsa is not running.
bluetooth_sink() {
	$TIMEOUT 3 $BLUEALSA_APLAY -L 2>/dev/null | sed -n 's/.*DEV=\([0-9A-Fa-f:]*\),PROFILE=a2dp.*/\1/p' | head -n 1
}

hdmi_active() {
	[ -f "$SYS/hdmi.sh" ] && sh "$SYS/hdmi.sh" active
}

where() {
	mac="$(bluetooth_sink)"
	if [ -n "$mac" ]; then
		echo "bluetooth:$mac"
	elif hdmi_active; then
		echo hdmi
	else
		echo speaker
	fi
}

# What the file for $1 (a `where` answer) holds, the marker first.
rc_for() {
	echo "$MARK"
	case "$1" in
	bluetooth:*)
		cat <<EOF
pcm.!default {
    type plug
    slave.pcm {
        type bluealsa
        device "${1#bluetooth:}"
        profile "a2dp"
    }
}
EOF
		;;
	hdmi)
		cat <<EOF
pcm.!default {
    type plug
    slave.pcm {
        type hw
        card ahubhdmi
    }
}
ctl.!default {
    type hw
    card ahubhdmi
}
EOF
		;;
	esac
}

sync_route() {
	now="$(where)"
	# a file somebody else wrote is theirs
	if [ -f "$ASOUNDRC" ] && ! head -n 1 "$ASOUNDRC" | grep -q -e "$MARK" -e "$OLD_MARK"; then
		return 0
	fi
	want="$(rc_for "$now")"
	if [ ! -f "$ASOUNDRC" ] || [ "$(cat "$ASOUNDRC")" != "$want" ]; then
		printf '%s\n' "$want" > "$ASOUNDRC"
	fi
}

case "${1:-}" in
sync) sync_route ;;
where) where ;;
loop)
	while :; do
		sync_route
		sleep 2
	done
	;;
*)
	echo "usage: audioroute.sh sync|loop|where" >&2
	exit 2
	;;
esac
