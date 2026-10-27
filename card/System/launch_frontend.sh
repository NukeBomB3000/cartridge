#!/bin/sh
SD="${CARTRIDGE_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
RUN="${AGS_RUN:-/run}"
LOADER="${AGS_LOADER:-/lib/ld-linux-aarch64.so.1}"
TMPDIR_RO="${AGS_TMP:-/tmp}"
export CARTRIDGE_ROOT="$SD"

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
log "exec $SYS/cartridge"

exec "$LOADER" "$SYS/cartridge" > "$out" 2>&1
