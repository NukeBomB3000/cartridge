#!/bin/sh
SD="${CARTRIDGE_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
RUN="${AGS_RUN:-/run}"
LOADER="${AGS_LOADER:-/lib/ld-linux-aarch64.so.1}"
TMPDIR_RO="${AGS_TMP:-/tmp}"
export CARTRIDGE_ROOT="$SD"
# a file TRACE at the card's root has cartridge say what the audio and the pacing are doing, once
# a second or so, in cartridge.log
[ -f "$SD/TRACE" ] && export CARTRIDGE_TRACE=1

log() {
	(echo "$(date '+%H:%M:%S') $*" >> "$SD/cartridge-session.log") 2>/dev/null
}

pid="$(cat "$RUN/cartridge-led.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/led.sh" once
	sh "$SYS/led.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/cartridge-led.pid"
fi

pid="$(cat "$RUN/cartridge-lid.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/lid.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/cartridge-lid.pid"
fi

# where the sound goes follows Bluetooth headphones and the HDMI cable; the file it writes is there
# before cartridge starts, as ALSA only watches the files it found
sh "$SYS/audioroute.sh" sync
pid="$(cat "$RUN/cartridge-audioroute.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/audioroute.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/cartridge-audioroute.pid"
fi

(echo "|/bin/sh $SYS/coresave.sh %p %s %e %t" > /proc/sys/kernel/core_pattern) 2>/dev/null

if [ -f "$SD/RUN_PROBE" ]; then
	rm -f "$SD/RUN_PROBE"
	log "probe $(AGS_PROBE_NET_WAIT_S=15 sh "$SYS/probe.sh" 2>&1)"
fi

# a card the kernel made read-only after an error takes no log, and the shell would refuse to
# start cartridge over a log it cannot open: cartridge then logs to /tmp instead and still
# starts, so it can say the card is read-only and offer to repair it
out="$SD/cartridge.log"
[ -f "$out" ] && mv -f "$out" "$out.1" 2>/dev/null
if ! (: > "$out") 2>/dev/null; then
	out="$TMPDIR_RO/cartridge.log"
fi
# the output follows the cable, HDMI with one in and the LCD without; it is switched here, with
# no EGL surface up, as the GPU driver latches the framebuffer's size as cartridge starts
if [ -f "$SYS/hdmi.sh" ]; then
	sh "$SYS/hdmi.sh" sync > /dev/null 2>&1
	if sh "$SYS/hdmi.sh" active; then
		log "output hdmi"
		# a 16:9 canvas, drawn 1:1 into the 960x540 framebuffer the engine doubles
		export CARTRIDGE_CANVAS=960x540
	else
		log "output lcd"
	fi
fi
log "exec $SYS/cartridge"

# cartridge ends when a game is run by a program of its own (PSP, by PPSSPP), leaving the
# program's name and then the game's in $RUN/cartridge-handoff, one to a line; the program runs
# on the game and cartridge starts again, on the shelf where it left off. Ending otherwise ends
# the session, which init starts afresh.
request="$RUN/cartridge-handoff"
rm -f "$request"
"$LOADER" "$SYS/cartridge" > "$out" 2>&1
while [ -f "$request" ]; do
	{
		read -r program
		read -r game
	} < "$request"
	rm -f "$request"
	case "$program" in
	psp)
		log "psp $game"
		sh "$SYS/psp.sh" run "$game" > "$SD/psp.log" 2>&1
		log "psp ended: $?"
		;;
	*) log "handoff to $program: no such program" ;;
	esac
	"$LOADER" "$SYS/cartridge" >> "$out" 2>&1
done
