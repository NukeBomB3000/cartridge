#!/bin/sh
SD="${OMEGA_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
RUN="${AGS_RUN:-/run}"
LOADER="${AGS_LOADER:-/lib/ld-linux-aarch64.so.1}"
TMPDIR_RO="${AGS_TMP:-/tmp}"
export OMEGA_ROOT="$SD"

log() {
	(echo "$(date '+%H:%M:%S') $*" >> "$SD/omega-zero-session.log") 2>/dev/null
}

pid="$(cat "$RUN/omega-led.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/led.sh" once
	sh "$SYS/led.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/omega-led.pid"
fi

pid="$(cat "$RUN/omega-lid.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/lid.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/omega-lid.pid"
fi

(echo "|/bin/sh $SYS/coresave.sh %p %s %e %t" > /proc/sys/kernel/core_pattern) 2>/dev/null

if [ -f "$SD/RUN_PROBE" ]; then
	rm -f "$SD/RUN_PROBE"
	log "probe $(AGS_PROBE_NET_WAIT_S=15 sh "$SYS/probe.sh" 2>&1)"
fi

# a card the kernel made read-only after an error takes no log, and the shell would refuse to
# start omega-zero over a log it cannot open: omega-zero then logs to /tmp instead and still
# starts, so it can say the card is read-only and offer to repair it
out="$SD/omega-zero.log"
[ -f "$out" ] && mv -f "$out" "$out.1" 2>/dev/null
if ! (: > "$out") 2>/dev/null; then
	out="$TMPDIR_RO/omega-zero.log"
fi
log "exec $SYS/omega-zero"

exec "$LOADER" "$SYS/omega-zero" > "$out" 2>&1
