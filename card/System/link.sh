#!/bin/sh
set -u

NET="${AGS_NET_SYS:-/sys/class/net}"
RUN="${AGS_RUN:-/run}"

LINK_SSID="${AGS_LINK_SSID:-cartridge-link}"
LINK_PSK="${AGS_LINK_PSK:-cartridge-link0}"
LINK_FREQ="${AGS_LINK_FREQ:-5745}"
LINK_HOST_IP="${AGS_LINK_HOST_IP:-10.42.0.1/24}"
LINK_PEER_IP="${AGS_LINK_PEER_IP:-10.42.0.2/24}"
LINK_WAIT_S="${AGS_LINK_WAIT_S:-10}"
LINK_PIN_WAIT_S="${AGS_LINK_PIN_WAIT_S:-20}"
LINK_RESCAN="${AGS_LINK_RESCAN:-2}"
LINK_AP_GLOBAL="${AGS_LINK_AP_GLOBAL:-}"
LINK_TRACE="${AGS_LINK_TRACE:-}"
LINK_MARK="$RUN/cartridge-link.session"
AP_MARK="$RUN/cartridge-link.ap"

HOME_WIFI="${AGS_HOME_WIFI:-$(dirname "$0")/wifi.sh}"

WPA_SUPPLICANT="${AGS_WPA_SUPPLICANT:-wpa_supplicant}"
WPA_CLI="${AGS_WPA_CLI:-wpa_cli}"
IP="${AGS_IP:-ip}"
UPTIME="${AGS_UPTIME:-/proc/uptime}"
CTRL_DIR="${AGS_CTRL_DIR:-/var/run/wpa_supplicant}"

trace() {
	[ -n "$LINK_TRACE" ] || return 0
	echo "cartridge-link: $(cut -d' ' -f1 "$UPTIME" 2>/dev/null) $*" >> "$LINK_TRACE" 2>/dev/null
	return 0
}

wait_dev() {
	w=0
	while [ ! -d "$NET/$1" ] && [ "$w" -lt "$2" ]; do
		sleep 0.25 2>/dev/null || sleep 1
		w=$((w + 1))
	done
	[ -d "$NET/$1" ]
}

wifi_up() {
	wait_dev wlan0 40 || return 0
	trace "wlan0 present"
	rfkill unblock wifi 2>/dev/null || true
	$IP link set wlan0 up 2>/dev/null || true
}

supplicant_stop() {
	if $WPA_CLI -p "$1" -i "$2" terminate >/dev/null 2>&1; then
		s=0
		while [ -e "$1/$2" ] && [ "$s" -lt 8 ]; do
			sleep 0.25 2>/dev/null || sleep 1
			s=$((s + 1))
		done
	elif [ -e "$1/$2" ]; then
		trace "$2: stale socket removed"
		/bin/rm -f "$1/$2"
	fi
	return 0
}

ap_if() {
	cat "$AP_MARK" 2>/dev/null || echo wlan1
}

link_clear() {
	# Home Wi-Fi gives wlan0 up to the link, its lease client included: one left running
	# would drop the link's address when it finds no DHCP server.
	sh "$HOME_WIFI" pause >/dev/null 2>&1 || true
	ap="$(ap_if)"
	supplicant_stop "$RUN/wpa_ap" "$ap"
	$IP addr flush dev "$ap" 2>/dev/null || true
	/bin/rm -f "$RUN/cartridge-link-ap.conf" "$AP_MARK"
	if [ -f "$RUN/cartridge-link-sta.conf" ]; then
		supplicant_stop "$CTRL_DIR" wlan0
		$IP addr flush dev wlan0 2>/dev/null || true
		/bin/rm -f "$RUN/cartridge-link-sta.conf"
	fi
}

link_wait() {
	p=0
	was=
	end=$(($(date +%s) + $3))
	while :; do
		state=$($WPA_CLI -p "$1" -i "$2" status 2>/dev/null | sed -n 's/^wpa_state=//p')
		if [ "$state" != "$was" ]; then
			trace "$2 ${state:-no answer}"
			was=$state
		fi
		[ "$state" = COMPLETED ] && return 0
		[ "$(date +%s)" -ge "$end" ] && return 1
		if [ "$4" -gt 0 ] && [ "$p" -gt 0 ] && [ $((p % $4)) -eq 0 ]; then
			case "$state" in
			DISCONNECTED | INACTIVE | SCANNING)
				$WPA_CLI -p "$1" -i "$2" scan ${5:+freq=$5} >/dev/null 2>&1
				;;
			esac
		fi
		sleep 0.25 2>/dev/null || sleep 1
		p=$((p + 1))
	done
}

link_host() {
	echo "$$" > "$LINK_MARK"
	trace "host: asked"
	link_clear
	wifi_up
	if wait_dev wlan1 4; then
		ap=wlan1
	elif [ -d "$NET/wlan0" ]; then
		ap=wlan0
		supplicant_stop "$CTRL_DIR" wlan0
		$IP addr flush dev wlan0 2>/dev/null || true
	else
		return 1
	fi
	trace "host: on $ap"
	command -v "$WPA_SUPPLICANT" >/dev/null 2>&1 || return 1
	echo "$ap" > "$AP_MARK"

	/bin/mkdir -p "$RUN/wpa_ap"
	{
		echo "ctrl_interface=$RUN/wpa_ap"
		echo 'ap_scan=2'
		if [ -n "$LINK_AP_GLOBAL" ]; then
			printf '%s\n' "$LINK_AP_GLOBAL"
		fi
		echo 'network={'
		echo "	ssid=\"$LINK_SSID\""
		echo '	mode=2'
		echo "	frequency=$LINK_FREQ"
		echo '	key_mgmt=WPA-PSK'
		echo '	proto=RSN'
		echo '	pairwise=CCMP'
		echo "	psk=\"$LINK_PSK\""
		echo '}'
	} > "$RUN/cartridge-link-ap.conf"
	/bin/chmod 600 "$RUN/cartridge-link-ap.conf"

	$IP link set "$ap" up 2>/dev/null || true
	$WPA_SUPPLICANT -B -i "$ap" -c "$RUN/cartridge-link-ap.conf" -Dnl80211 >/dev/null 2>&1 || return 1
	link_wait "$RUN/wpa_ap" "$ap" "$LINK_WAIT_S" 0 "" || return 1
	$IP addr add "$LINK_HOST_IP" dev "$ap" 2>/dev/null || true
	trace "host: addressed"
}

link_sta_conf() {
	{
		echo "ctrl_interface=$CTRL_DIR"
		echo 'network={'
		echo "	ssid=\"$LINK_SSID\""
		echo '	key_mgmt=WPA-PSK'
		echo "	psk=\"$LINK_PSK\""
		if [ -n "${1:-}" ]; then
			echo "	scan_freq=$1"
			echo "	freq_list=$1"
		fi
		echo '}'
	} > "$RUN/cartridge-link-sta.conf"
	/bin/chmod 600 "$RUN/cartridge-link-sta.conf"
}

link_join() {
	echo "$$" > "$LINK_MARK"
	trace "join: asked"
	link_clear
	wifi_up
	[ -d "$NET/wlan0" ] || return 1
	/bin/mkdir -p "$CTRL_DIR"
	$IP link set wlan0 up 2>/dev/null || true

	tries="${AGS_LINK_TRIES:-2}"
	attempt=1
	while [ "$attempt" -le "$tries" ]; do
		if [ "$attempt" -eq 1 ]; then
			freq="$LINK_FREQ"
			bound="$LINK_PIN_WAIT_S"
		else
			freq=
			bound="$LINK_WAIT_S"
		fi
		link_sta_conf "$freq"
		supplicant_stop "$CTRL_DIR" wlan0
		$WPA_SUPPLICANT -B -i wlan0 -c "$RUN/cartridge-link-sta.conf" -Dnl80211 >/dev/null 2>&1 \
			|| { attempt=$((attempt + 1)); continue; }
		trace "join: attempt $attempt on ${freq:-every channel}"
		if link_wait "$CTRL_DIR" wlan0 "$bound" "$LINK_RESCAN" "$freq"; then
			$IP addr add "$LINK_PEER_IP" dev wlan0 2>/dev/null || true
			trace "join: addressed"
			return 0
		fi
		attempt=$((attempt + 1))
	done
	trace "join: no host found"
	return 3
}

link_down() {
	trace "down"
	ap="$(ap_if)"
	link_clear
	[ "$ap" = wlan1 ] && $IP link set wlan1 down 2>/dev/null
	true
	rmdir "$RUN/wpa_ap" 2>/dev/null || true
	/bin/rm -f "$LINK_MARK"
	sh "$HOME_WIFI" resume >/dev/null 2>&1 || true
	return 0
}

link_warm() {
	trace "warm"
	wifi_up
	[ -d "$NET/wlan0" ] || return 1
	wait_dev wlan1 4 || true
}

case "${1:-} ${2:-}" in
"link host") link_host ;;
"link join") link_join ;;
"link down") link_down ;;
"link warm") link_warm ;;
"link cool") exit 0 ;;
*)
	echo "usage: link.sh link [host|join|down|warm|cool]" >&2
	exit 2
	;;
esac
