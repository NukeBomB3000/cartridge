#!/bin/sh
# hdmi.sh sync|on|off|plugged|active
#
# HDMI out on the H700 handhelds (mini HDMI port). The display engine has one output at a time:
# the LCD or the HDMI transmitter. Switching goes through the dispdbg debugfs of the sunxi disp2
# driver, as NextUI's and the stock firmware's HDMI monitors do, and it must happen while no EGL
# surface exists: the GPU driver latches the framebuffer's size as the frontend starts. So the
# launcher runs `hdmi.sh sync` before it starts cartridge, and cartridge ends itself when a
# cable comes or goes (BaseOS starts it again).
#
#   sync     make the output match the cable: HDMI with a cable in, the LCD without
#   on, off  force one
#   plugged  exit 0 with a cable in
#   active   exit 0 while the HDMI transmitter has the output
#
# The signal is 1080p60 whatever the monitor is; a 4K monitor scales it itself. The framebuffer is
# half of that, 960x540, which the display engine doubles exactly, and cartridge draws a 16:9
# canvas of the same size 1:1 into it (launch_frontend.sh names it), so the pixels stay sharp and a
# game stretched fills the monitor. A bigger framebuffer does not fit the GPU's window surfaces.

DBG=/sys/kernel/debug/dispdbg
FB=/sys/class/graphics/fb0
HDMI_PARAM="4 10 0 0 0x4 0x101 0 0 0 8"
LCD_PARAM="1 0"
LCD_SIZE=/run/cartridge-lcd-size
ASOUNDRC="${HOME:-/root}/.asoundrc"
MARK="# cartridge-hdmi"

target="$(sed -n 's/^BASEOS_TARGET=//p' /etc/baseos-release 2>/dev/null)"

plugged() {
	[ "$(sed -n 's/^HDMI=//p' /sys/class/extcon/hdmi/state 2>/dev/null)" = 1 ]
}

active() {
	grep -q 'hdmi output' /sys/class/disp/disp/attr/sys 2>/dev/null
}

# the LCD's size, as the framebuffer had it before the first switch to HDMI
panel_size() {
	if [ -s "$LCD_SIZE" ]; then
		cat "$LCD_SIZE"
		return
	fi
	case "$target" in
	rgcubexx) echo "720 720" ;;
	rg34xx | rg34xxsp) echo "720 480" ;;
	rg28xx) echo "480 640" ;;
	*) echo "640 480" ;;
	esac
}

fb_size() {
	# $1 width, $2 height: zero first so the driver drops the old allocation
	fbset -fb /dev/fb0 -g 0 0 0 0 32 2>/dev/null
	sleep 0.25
	fbset -fb /dev/fb0 -g "$1" "$2" "$1" "$(($2 * 2))" 32 2>/dev/null
	# the pan offset of the old size (the LCD's 480) is not a page of the new one
	echo 0,0 > "$FB/pan" 2>/dev/null
}

wait_for() {
	# $1 = hdmi | lcd: the attach is asynchronous and slow, up to 3 s
	i=0
	while [ "$i" -lt 60 ]; do
		if [ "$1" = hdmi ]; then active && break; else active || break; fi
		sleep 0.05
		i=$((i + 1))
	done
	sleep 0.15
}

audio() {
	# ALSA's default to the HDMI transmitter while it has the output; only a file we wrote
	if [ "$1" = on ]; then
		if [ -f "$ASOUNDRC" ] && ! head -n1 "$ASOUNDRC" | grep -q "$MARK"; then
			return
		fi
		cat > "$ASOUNDRC" <<EOF
$MARK
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
	elif [ -f "$ASOUNDRC" ] && head -n1 "$ASOUNDRC" | grep -q "$MARK"; then
		rm -f "$ASOUNDRC"
	fi
}

switch() {
	# $1 = hdmi | lcd
	[ -w "$DBG/start" ] || exit 1
	if [ "$1" = hdmi ]; then
		active && { audio on; return; }
		# the LCD's size, once, before the framebuffer is resized for the TV
		if [ ! -s "$LCD_SIZE" ]; then
			vs="$(cat "$FB/virtual_size")"
			echo "${vs%,*} $((${vs#*,} / 2))" > "$LCD_SIZE"
		fi
		fbw=960
		fbh=540
		set -- hdmi "$HDMI_PARAM" "$fbw" "$fbh"
	else
		active || { audio off; return; }
		set -- lcd "$LCD_PARAM" $(panel_size)
	fi
	echo 4 > "$FB/blank"
	cat /dev/zero 2>/dev/null | dd of=/dev/fb0 bs=4096 count=900 2>/dev/null
	echo disp0 > "$DBG/name"
	echo switch > "$DBG/command"
	echo "$2" > "$DBG/param"
	echo 1 > "$DBG/start"
	wait_for "$1"
	fb_size "$3" "$4"
	sleep 0.25
	echo 0 > "$FB/blank"
	if [ "$1" = hdmi ]; then audio on; else audio off; fi
}

case "$1" in
plugged) plugged ;;
active) active ;;
on) switch hdmi ;;
off) switch lcd ;;
sync)
	if plugged; then switch hdmi; else switch lcd; fi
	;;
*)
	echo "usage: hdmi.sh sync|on|off|plugged|active" >&2
	exit 2
	;;
esac
