#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/System/launch_frontend.sh"
TMP="$(mktemp -d)"
trap 'chmod -R u+w "$TMP" 2>/dev/null; rm -rf "$TMP"' EXIT HUP INT TERM

setup() {
	chmod -R u+w "$TMP" 2>/dev/null || true
	rm -rf "$TMP/card" "$TMP/run" "$TMP/tmp"
	mkdir -p "$TMP/card/System" "$TMP/run" "$TMP/tmp"
	for s in led.sh lid.sh coresave.sh; do
		printf '#!/bin/sh\nexit 0\n' > "$TMP/card/System/$s"
	done
	printf '#!/bin/sh\necho "omega-zero ran"\n' > "$TMP/card/System/omega-zero"
	# the loader runs the program it is given, as /lib/ld-linux-aarch64.so.1 does
	printf '#!/bin/sh\nexec sh "$@"\n' > "$TMP/loader"
	chmod 755 "$TMP/loader"
}

launch() {
	OMEGA_ROOT="$TMP/card" AGS_RUN="$TMP/run" AGS_LOADER="$TMP/loader" AGS_TMP="$TMP/tmp" \
		sh "$SCRIPT"
}

fail() {
	echo "launch: $*" >&2
	exit 1
}

setup
echo "the run before" > "$TMP/card/omega-zero.log"
launch
grep -q "omega-zero ran" "$TMP/card/omega-zero.log" || fail "omega-zero did not log to the card"
grep -q "the run before" "$TMP/card/omega-zero.log.1" || fail "the last log was not kept"

setup
echo "the run before" > "$TMP/card/omega-zero.log"
chmod a-w "$TMP/card/omega-zero.log" "$TMP/card"
launch
grep -q "omega-zero ran" "$TMP/tmp/omega-zero.log" ||
	fail "on a read-only card omega-zero did not start, logging to /tmp"
grep -q "the run before" "$TMP/card/omega-zero.log" || fail "the read-only card's log changed"

echo "launch: ok"
