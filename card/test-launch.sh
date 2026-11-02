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
	for s in led.sh lid.sh coresave.sh audioroute.sh; do
		printf '#!/bin/sh\nexit 0\n' > "$TMP/card/System/$s"
	done
	printf '#!/bin/sh\necho "cartridge ran"\n' > "$TMP/card/System/cartridge"
	# the loader runs the program it is given, as /lib/ld-linux-aarch64.so.1 does
	printf '#!/bin/sh\nexec sh "$@"\n' > "$TMP/loader"
	chmod 755 "$TMP/loader"
}

launch() {
	CARTRIDGE_ROOT="$TMP/card" AGS_RUN="$TMP/run" AGS_LOADER="$TMP/loader" AGS_TMP="$TMP/tmp" \
		sh "$SCRIPT"
}

fail() {
	echo "launch: $*" >&2
	exit 1
}

setup
echo "the run before" > "$TMP/card/cartridge.log"
launch
grep -q "cartridge ran" "$TMP/card/cartridge.log" || fail "cartridge did not log to the card"
grep -q "the run before" "$TMP/card/cartridge.log.1" || fail "the last log was not kept"

setup
echo "the run before" > "$TMP/card/cartridge.log"
chmod a-w "$TMP/card/cartridge.log" "$TMP/card"
launch
grep -q "cartridge ran" "$TMP/tmp/cartridge.log" ||
	fail "on a read-only card cartridge did not start, logging to /tmp"
grep -q "the run before" "$TMP/card/cartridge.log" || fail "the read-only card's log changed"

# a game a program of its own runs: cartridge leaves the request and ends, the program runs on
# the game, and cartridge starts again
setup
cat > "$TMP/card/System/cartridge" <<EOF
#!/bin/sh
if [ -f "$TMP/card/seen" ]; then
	echo "second run"
else
	: > "$TMP/card/seen"
	printf 'psp\\n/mnt/sdcard/Games/PSP/Racing Game.cso\\n' > "$TMP/run/cartridge-handoff"
	echo "first run"
fi
EOF
cat > "$TMP/card/System/psp.sh" <<EOF
#!/bin/sh
echo "psp.sh \$*" > "$TMP/card/psp-ran"
EOF
launch
grep -q "second run" "$TMP/card/cartridge.log" || fail "cartridge did not start again after the program"
grep -q 'psp.sh run /mnt/sdcard/Games/PSP/Racing Game.cso' "$TMP/card/psp-ran" ||
	fail "psp.sh was not run on the game: $(cat "$TMP/card/psp-ran" 2>/dev/null)"
[ ! -f "$TMP/run/cartridge-handoff" ] || fail "the request was left behind"

# with no request cartridge runs once, as it did when it was exec'd
setup
launch
[ "$(grep -c "cartridge ran" "$TMP/card/cartridge.log")" = 1 ] || fail "cartridge ran more than once"

echo "launch: ok"
