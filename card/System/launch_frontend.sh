#!/bin/sh
SD="${OMEGA_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
RUN="${AGS_RUN:-/run}"
export OMEGA_ROOT="$SD"

log() {
	echo "$(date '+%H:%M:%S') $*" >> "$SD/omega-zero-session.log" 2>/dev/null
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

echo "|/bin/sh $SYS/coresave.sh %p %s %e %t" > /proc/sys/kernel/core_pattern 2>/dev/null

if [ -f "$SD/RUN_PROBE" ]; then
	rm -f "$SD/RUN_PROBE"
	log "probe $(AGS_PROBE_NET_WAIT_S=15 sh "$SYS/probe.sh" 2>&1)"
fi

[ -f "$SD/omega-zero.log" ] && mv -f "$SD/omega-zero.log" "$SD/omega-zero.log.1"
log "exec $SYS/omega-zero"

exec /lib/ld-linux-aarch64.so.1 "$SYS/omega-zero" > "$SD/omega-zero.log" 2>&1
