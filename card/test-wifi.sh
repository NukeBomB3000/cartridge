#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/System/wifi.sh"
TMP="$(mktemp -d)"
trap 'stray; rm -rf "$TMP"' EXIT HUP INT TERM
FAILS=0

fail() {
	echo "FAIL: $*"
	FAILS=$((FAILS + 1))
}

# The fake daemons are sleeps standing in for real processes, so the script can signal and
# stop them. Any left at the end are stopped here.
stray() {
	for p in "$TMP"/run/*.pid; do
		[ -s "$p" ] && kill "$(cat "$p")" 2>/dev/null
	done
	return 0
}

fake() {
	name="$1"
	shift
	{
		echo '#!/bin/sh'
		echo "printf '%s\\n' \"\$*\" >> \"$TMP/$name.log\""
		for line in "$@"; do echo "$line"; done
		echo 'exit 0'
	} > "$TMP/bin/$name"
	chmod 755 "$TMP/bin/$name"
}

reset() {
	stray
	rm -rf "$TMP/net" "$TMP/run" "$TMP/ctrl" "$TMP/card" "$TMP"/*.log "$TMP/bin"
	mkdir -p "$TMP/net/wlan0" "$TMP/run" "$TMP/ctrl" "$TMP/card/Config" "$TMP/bin"
	echo rg35xxsp > "$TMP/hostname"
	fake wpa_cli \
		'for a in "$@"; do last="$a"; done' \
		'case " $* " in' \
		"*' ping '*) [ -e \"$TMP/ctrl/wlan0\" ] && echo PONG ;;" \
		"*' terminate '*) rm -f \"$TMP/ctrl/wlan0\"; echo OK ;;" \
		"*' status '*) echo \"wpa_state=\${FAKE_STATE:-COMPLETED}\"; printf '%s\\n' 'ssid=Home \\xc3\\xa9'; echo 'ip_address=192.168.1.20' ;;" \
		"*' signal_poll '*) echo 'RSSI=-52'; echo 'LINKSPEED=72' ;;" \
		"*' scan_results '*) printf 'bssid / frequency / signal level / flags / ssid\\n02:00:00:00:00:01\\t2437\\t-48\\t[WPA2-PSK-CCMP][ESS]\\tHome\\n' ;;" \
		"*' list_networks '*) printf 'network id / ssid / bssid / flags\\n1\\tHome\\tany\\t[CURRENT]\\n' ;;" \
		"*' add_network '*) echo 3 ;;" \
		"*' psk '*) [ -n \"\${FAKE_PSK_FAIL:-}\" ] && echo FAIL || echo OK ;;" \
		"*' -a '*) sleep 30 & echo \$! > \"\$last\" ;;" \
		'*) echo OK ;;' \
		'esac'
	fake wpa_supplicant ": > \"$TMP/ctrl/wlan0\""
	fake udhcpc \
		'pid=' \
		'while [ $# -gt 0 ]; do [ "$1" = -p ] && pid="$2"; shift; done' \
		'echo $$ > "$pid"' \
		"trap 'echo renew >> \"$TMP/udhcpc.log\"' USR1" \
		"trap 'echo release >> \"$TMP/udhcpc.log\"' USR2" \
		'while :; do sleep 0.1; done'
	fake ip
	fake rfkill
	fake timedatectl 'case "$1" in show) echo yes ;; esac'
}

run() {
	PATH="$TMP/bin:$PATH" \
		AGS_CARD="$TMP/card" AGS_NET_SYS="$TMP/net" AGS_RUN="$TMP/run" AGS_CTRL_DIR="$TMP/ctrl" \
		AGS_HOSTNAME="$TMP/hostname" AGS_WIFI_WAIT_S="${WAIT_S:-1}" AGS_WIFI_SCAN_S=0 \
		AGS_WPA_SUPPLICANT="$TMP/bin/wpa_supplicant" AGS_WPA_CLI="$TMP/bin/wpa_cli" \
		AGS_UDHCPC="$TMP/bin/udhcpc" AGS_IP="$TMP/bin/ip" AGS_TIMEDATECTL="$TMP/bin/timedatectl" \
		sh "$SCRIPT" "$@"
}

# The lease client is a background process: give it a moment to write its pid or log a signal.
settle() {
	n=0
	while [ "$n" -lt 20 ]; do
		[ -s "$1" ] && grep -q "${2:-.}" "$1" 2>/dev/null && return 0
		sleep 0.1
		n=$((n + 1))
	done
	return 1
}

conf="$TMP/card/Config/wifi.conf"

reset
run up && rc=0 || rc=$?
[ "$rc" = 0 ] || fail "up exited $rc"
[ -f "$TMP/run/cartridge-wifi.up" ] || fail "up left no mark that Wi-Fi is wanted"
grep -q -- "-i wlan0 -c $conf -Dnl80211" "$TMP/wpa_supplicant.log" || fail "the supplicant did not start on the card's networks"
grep -q "^ctrl_interface=$TMP/ctrl" "$conf" || fail "the networks file has no control socket"
grep -q '^update_config=1' "$conf" || fail "the networks file may not be saved to"
grep -q 'enable_network all' "$TMP/wpa_cli.log" || fail "up did not let every saved network be joined"
grep -q -- "-a $TMP/run/cartridge-wifi-hook -B -P $TMP/run/cartridge-wifi-events.pid" "$TMP/wpa_cli.log" \
	|| fail "up did not listen for the radio joining a network"
[ -x "$TMP/run/cartridge-wifi-hook" ] || fail "the hook is not executable"
grep -q "'$SCRIPT' event" "$TMP/run/cartridge-wifi-hook" || fail "the hook does not call back into wifi.sh"
settle "$TMP/run/cartridge-wifi-udhcpc.pid" || fail "up on a joined network asked for no lease"
grep -q -- '-i wlan0 -R -T 2 -p .* -x hostname:rg35xxsp' "$TMP/udhcpc.log" || fail "the lease was not asked for under the device's name"

run up >/dev/null
[ "$(grep -c -- '-Dnl80211' "$TMP/wpa_supplicant.log")" = 1 ] || fail "a second up started a second supplicant"

reset
printf 'update_config=0\nnetwork={\n\tssid="Cabin"\n\tpsk="hunter22"\n}\n' > "$conf"
run up >/dev/null
settle "$TMP/run/cartridge-wifi-udhcpc.pid"
grep -q 'ssid="Cabin"' "$conf" || fail "up lost a network written on a computer"
grep -q '^update_config=0' "$conf" && fail "up left saving switched off"
[ "$(grep -c '^update_config=1' "$conf")" = 1 ] || fail "up did not switch saving on exactly once"

reset
FAKE_STATE=SCANNING run up >/dev/null
sleep 0.3
[ -e "$TMP/run/cartridge-wifi-udhcpc.pid" ] && fail "up asked for a lease before joining a network"
run event wlan0 CONNECTED
settle "$TMP/run/cartridge-wifi-udhcpc.pid" || fail "joining a network asked for no lease"
run event wlan0 CONNECTED
settle "$TMP/udhcpc.log" release || fail "joining again did not let the old lease go"
settle "$TMP/udhcpc.log" renew || fail "joining again did not ask for a new lease"
[ "$(grep -c -- '-i wlan0 -R' "$TMP/udhcpc.log")" = 1 ] || fail "joining again started a second client"

reset
: > "$TMP/run/cartridge-link.session"
run up && rc=0 || rc=$?
[ "$rc" = 0 ] || fail "up during a link exited $rc"
[ -f "$TMP/run/cartridge-wifi.up" ] || fail "up during a link forgot Wi-Fi is wanted"
[ -e "$TMP/wpa_supplicant.log" ] && fail "up took the radio from a link"

reset
rmdir "$TMP/net/wlan0"
WAIT_S=0 run up && rc=0 || rc=$?
[ "$rc" = 1 ] || fail "up without wlan0 exited $rc"

reset
run up >/dev/null
settle "$TMP/run/cartridge-wifi-udhcpc.pid"
dhcp="$(cat "$TMP/run/cartridge-wifi-udhcpc.pid")"
events="$(cat "$TMP/run/cartridge-wifi-events.pid")"
run pause
[ -f "$TMP/run/cartridge-wifi.up" ] || fail "pause forgot Wi-Fi is wanted"
kill -0 "$dhcp" 2>/dev/null && fail "pause left the lease client running"
kill -0 "$events" 2>/dev/null && fail "pause left the hook listening"
[ -e "$TMP/ctrl/wlan0" ] && fail "pause left the supplicant running"
grep -q 'addr flush dev wlan0' "$TMP/ip.log" || fail "pause left the address on wlan0"
run resume
[ -e "$TMP/ctrl/wlan0" ] || fail "resume did not bring Wi-Fi back"
run down
[ -f "$TMP/run/cartridge-wifi.up" ] && fail "down left the mark"
[ -e "$TMP/ctrl/wlan0" ] && fail "down left the supplicant running"
: > "$TMP/wpa_supplicant.log"
run resume
[ -s "$TMP/wpa_supplicant.log" ] && fail "resume turned on Wi-Fi that was off"

reset
run scan && rc=0 || rc=$?
[ "$rc" = 1 ] || fail "scan with Wi-Fi off exited $rc"
run up >/dev/null
settle "$TMP/run/cartridge-wifi-udhcpc.pid"
out="$(run scan)"
printf '%s\n' "$out" | grep -q "$(printf '\t')Home$" || fail "scan printed no networks"
grep -q ' scan$' "$TMP/wpa_cli.log" || fail "scan did not ask the radio to look"
run saved | grep -q "$(printf '1\tHome')" || fail "saved printed no networks"

reset
run up >/dev/null
settle "$TMP/run/cartridge-wifi-udhcpc.pid"
echo 'correct horse' | run join 486f6d65 && rc=0 || rc=$?
[ "$rc" = 0 ] || fail "join exited $rc"
grep -q 'set_network 3 ssid 486f6d65' "$TMP/wpa_cli.log" || fail "join did not name the network in hex"
grep -q 'set_network 3 psk "correct horse"' "$TMP/wpa_cli.log" || fail "join did not pass the passphrase quoted"
grep -q 'select_network 3' "$TMP/wpa_cli.log" || fail "join did not pick the network"
grep -q 'save_config' "$TMP/wpa_cli.log" || fail "join did not save the network"
grep -q 'remove_network' "$TMP/wpa_cli.log" && fail "join removed a network it was not asked to"

: > "$TMP/wpa_cli.log"
printf '' | run join 486f6d65 1
grep -q 'remove_network 1' "$TMP/wpa_cli.log" || fail "join did not replace the saved network"
grep -q 'set_network 3 key_mgmt NONE' "$TMP/wpa_cli.log" || fail "an open network was given a key"

: > "$TMP/wpa_cli.log"
echo short | FAKE_PSK_FAIL=1 run join 486f6d65 && rc=0 || rc=$?
[ "$rc" = 4 ] || fail "a refused passphrase exited $rc"
grep -q 'remove_network 3' "$TMP/wpa_cli.log" || fail "a refused passphrase left a half-made network"

run join 'Home' </dev/null && rc=0 || rc=$?
[ "$rc" = 2 ] || fail "a name that is not hex exited $rc"
run forget x && rc=0 || rc=$?
[ "$rc" = 2 ] || fail "forgetting a network that is not a number exited $rc"
: > "$TMP/wpa_cli.log"
run forget 1
grep -q 'remove_network 1' "$TMP/wpa_cli.log" || fail "forget did not remove the network"
grep -q 'save_config' "$TMP/wpa_cli.log" || fail "forget did not save"
: > "$TMP/wpa_cli.log"
run select 1
grep -q 'select_network 1' "$TMP/wpa_cli.log" || fail "select did not pick the network"

reset
run up >/dev/null
settle "$TMP/run/cartridge-wifi-udhcpc.pid"
out="$(run status)"
for want in enabled=yes wpa_state=COMPLETED 'ssid=Home \xc3\xa9' ip_address=192.168.1.20 RSSI=-52 \
	hostname=rg35xxsp ntp=yes ntp_synced=yes; do
	printf '%s\n' "$out" | grep -qxF -- "$want" || fail "status lacks $want"
done
printf '%s\n' "$out" | grep -q LINKSPEED && fail "status passed on more than the signal"
run down
run status | grep -qx enabled=no || fail "status after down still says enabled"

reset
run clock on
run clock off
grep -qx 'set-ntp true' "$TMP/timedatectl.log" || fail "clock on did not turn on network time"
grep -qx 'set-ntp false' "$TMP/timedatectl.log" || fail "clock off did not turn off network time"
run clock maybe && rc=0 || rc=$?
[ "$rc" = 2 ] || fail "clock with no on or off exited $rc"

run bogus 2>/dev/null && rc=0 || rc=$?
[ "$rc" = 2 ] || fail "an unknown verb exited $rc"

if [ "$FAILS" -gt 0 ]; then
	echo "wifi: $FAILS failed"
	exit 1
fi
echo "wifi: all passed"
