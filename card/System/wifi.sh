#!/bin/sh
# Home Wi-Fi. BaseOS loads the driver and brings wlan0 up; association, DHCP and the saved
# networks are the frontend's. The networks live on the card in Config/wifi.conf, in
# wpa_supplicant's own format, so one can be added from a computer as well as from cartridge.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SELF="$HERE/$(basename "$0")"
CARD="${AGS_CARD:-$HERE/..}"
NET="${AGS_NET_SYS:-/sys/class/net}"
RUN="${AGS_RUN:-/run}"
CTRL_DIR="${AGS_CTRL_DIR:-/var/run/wpa_supplicant}"
CONF="${AGS_WIFI_CONF:-$CARD/Config/wifi.conf}"
WAIT_S="${AGS_WIFI_WAIT_S:-25}"
SCAN_S="${AGS_WIFI_SCAN_S:-3}"
HOSTNAME_FILE="${AGS_HOSTNAME:-/proc/sys/kernel/hostname}"

WPA_SUPPLICANT="${AGS_WPA_SUPPLICANT:-wpa_supplicant}"
WPA_CLI="${AGS_WPA_CLI:-wpa_cli}"
UDHCPC="${AGS_UDHCPC:-udhcpc}"
IP="${AGS_IP:-ip}"
TIMEDATECTL="${AGS_TIMEDATECTL:-timedatectl}"

IFACE=wlan0
UP_MARK="$RUN/cartridge-wifi.up"
DHCP_PID="$RUN/cartridge-wifi-udhcpc.pid"
EVENTS_PID="$RUN/cartridge-wifi-events.pid"
HOOK="$RUN/cartridge-wifi-hook"
LINK_MARK="$RUN/cartridge-link.session"

cli() {
	$WPA_CLI -p "$CTRL_DIR" -i "$IFACE" "$@" 2>/dev/null
}

ok() {
	cli "$@" | grep -q '^OK'
}

running() {
	cli ping | grep -q PONG
}

nap() {
	sleep 0.25 2>/dev/null || sleep 1
}

wait_dev() {
	w=0
	while [ ! -d "$NET/$IFACE" ] && [ "$w" -lt $(($1 * 4)) ]; do
		nap
		w=$((w + 1))
	done
	[ -d "$NET/$IFACE" ]
}

# wpa_supplicant needs a control socket to be driven and leave to save what it learns.
conf_ready() {
	/bin/mkdir -p "$(dirname "$CONF")"
	grep -q '^ctrl_interface=' "$CONF" 2>/dev/null && grep -q '^update_config=1' "$CONF" 2>/dev/null \
		&& return 0
	{
		grep -q '^ctrl_interface=' "$CONF" 2>/dev/null || echo "ctrl_interface=$CTRL_DIR"
		echo 'update_config=1'
		grep -v '^update_config=' "$CONF" 2>/dev/null
	} > "$CONF.new"
	/bin/mv -f "$CONF.new" "$CONF"
}

host_name() {
	cat "$HOSTNAME_FILE" 2>/dev/null
}

alive() {
	[ -s "$1" ] && kill -0 "$(cat "$1")" 2>/dev/null
}

stop_pid() {
	if alive "$1"; then
		kill "$(cat "$1")" 2>/dev/null
	fi
	/bin/rm -f "$1"
}

# A lease is asked for each time the radio joins a network, so a new network or a router
# that came back answers at once rather than at the client's next retry. A running client
# lets its lease go and asks again from the start.
dhcp_up() {
	if alive "$DHCP_PID"; then
		kill -USR2 "$(cat "$DHCP_PID")" 2>/dev/null
		kill -USR1 "$(cat "$DHCP_PID")" 2>/dev/null
		return 0
	fi
	name="$(host_name)"
	# BaseOS's default udhcpc script sets the address, route and DNS, and starts mDNS.
	$UDHCPC -i "$IFACE" -R -T 2 -p "$DHCP_PID" ${name:+-x "hostname:$name"} \
		</dev/null >/dev/null 2>&1 &
}

# wpa_cli runs the hook whenever the radio joins or leaves a network. It needs a file it
# may execute, which the card cannot promise, so the hook lives in /run.
events_up() {
	alive "$EVENTS_PID" && return 0
	{
		echo '#!/bin/sh'
		echo "exec /bin/sh '$SELF' event \"\$@\""
	} > "$HOOK"
	/bin/chmod 755 "$HOOK"
	cli -a "$HOOK" -B -P "$EVENTS_PID" >/dev/null
}

wifi_event() {
	case "${2:-}" in
	CONNECTED) dhcp_up ;;
	esac
}

supplicant_down() {
	running || return 0
	cli terminate >/dev/null
	s=0
	while [ -e "$CTRL_DIR/$IFACE" ] && [ "$s" -lt 8 ]; do
		nap
		s=$((s + 1))
	done
}

wifi_up() {
	: > "$UP_MARK"
	# A link owns the radio until it ends; it resumes home Wi-Fi then.
	[ -f "$LINK_MARK" ] && return 0
	wait_dev "$WAIT_S" || return 1
	rfkill unblock wifi 2>/dev/null || true
	$IP link set "$IFACE" up 2>/dev/null || true
	conf_ready
	if ! running; then
		/bin/mkdir -p "$CTRL_DIR"
		$WPA_SUPPLICANT -B -i "$IFACE" -c "$CONF" -Dnl80211 >/dev/null 2>&1 || return 1
	fi
	# Joining one network leaves the others disabled; any saved one may be joined again.
	cli enable_network all >/dev/null
	events_up
	if cli status | grep -q '^wpa_state=COMPLETED'; then
		dhcp_up
	fi
}

wifi_stop() {
	stop_pid "$EVENTS_PID"
	stop_pid "$DHCP_PID"
	supplicant_down
	$IP addr flush dev "$IFACE" 2>/dev/null || true
}

wifi_down() {
	/bin/rm -f "$UP_MARK"
	wifi_stop
}

wifi_status() {
	if [ -f "$UP_MARK" ]; then echo enabled=yes; else echo enabled=no; fi
	if running; then
		cli status
		cli signal_poll | grep '^RSSI='
	fi
	echo "hostname=$(host_name)"
	echo "ntp=$($TIMEDATECTL show -p NTP --value 2>/dev/null)"
	echo "ntp_synced=$($TIMEDATECTL show -p NTPSynchronized --value 2>/dev/null)"
}

wifi_scan() {
	running || return 1
	cli scan >/dev/null
	w=0
	while [ "$w" -lt $((SCAN_S * 4)) ]; do
		nap
		w=$((w + 1))
	done
	cli scan_results
}

# $1: the SSID in hex, which wpa_supplicant takes unquoted and so needs no escaping.
# $2: a saved network this one replaces. The passphrase, if any, comes on stdin.
wifi_join() {
	hex="${1:-}"
	case "$hex" in
	"" | *[!0-9a-fA-F]*) return 2 ;;
	esac
	psk=
	IFS= read -r psk || true
	[ -f "$UP_MARK" ] && running || wifi_up || return 1
	running || return 1
	if [ -n "${2:-}" ]; then
		cli remove_network "$2" >/dev/null
	fi
	id="$(cli add_network | tail -n 1)"
	case "$id" in
	"" | *[!0-9]*) return 1 ;;
	esac
	if ! ok set_network "$id" ssid "$hex"; then
		cli remove_network "$id" >/dev/null
		return 1
	fi
	if [ -n "$psk" ]; then
		if ! ok set_network "$id" psk "\"$psk\""; then
			cli remove_network "$id" >/dev/null
			return 4
		fi
	else
		ok set_network "$id" key_mgmt NONE
	fi
	ok select_network "$id" || return 1
	cli save_config >/dev/null
}

wifi_select() {
	case "${1:-}" in
	"" | *[!0-9]*) return 2 ;;
	esac
	running || return 1
	ok select_network "$1" || return 1
	cli save_config >/dev/null
}

wifi_forget() {
	case "${1:-}" in
	"" | *[!0-9]*) return 2 ;;
	esac
	running || return 1
	ok remove_network "$1" || return 1
	cli save_config >/dev/null
}

wifi_clock() {
	case "${1:-}" in
	on) $TIMEDATECTL set-ntp true ;;
	off) $TIMEDATECTL set-ntp false ;;
	*) return 2 ;;
	esac
}

case "${1:-}" in
up) wifi_up ;;
down) wifi_down ;;
pause) wifi_stop ;;
resume) if [ -f "$UP_MARK" ]; then wifi_up; fi ;;
status) wifi_status ;;
scan) wifi_scan ;;
saved) running && cli list_networks ;;
join) wifi_join "${2:-}" "${3:-}" ;;
select) wifi_select "${2:-}" ;;
forget) wifi_forget "${2:-}" ;;
clock) wifi_clock "${2:-}" ;;
event) wifi_event "${2:-}" "${3:-}" ;;
*)
	echo "usage: wifi.sh up|down|pause|resume|status|scan|saved|join HEX [ID]|select ID|forget ID|clock on|off" >&2
	exit 2
	;;
esac
