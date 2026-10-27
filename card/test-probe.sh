#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/System/probe.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
FAILS=0

fail() {
	echo "FAIL: $*"
	FAILS=$((FAILS + 1))
}

put() {
	mkdir -p "$(dirname "$1")"
	printf '%s\n' "$2" > "$1"
}

fake() {
	{
		echo '#!/bin/sh'
		echo "echo \"fake $1 \$*\""
	} > "$TMP/bin/$1"
	chmod 755 "$TMP/bin/$1"
}

mkdir -p "$TMP/bin" "$TMP/card/System" "$TMP/dev"
put "$TMP/etc/baseos-release" 'BASEOS=1
BASEOS_TARGET=rg35xxsp
BASEOS_MODEL_STRING=RG35xxSP
BASEOS_PANEL_ROTATION_CCW=0'
printf '\001\002RG35xxSP\000junk\n' > "$TMP/vendor-dmenu"
mkdir -p "$TMP/vendor/bin"
mv "$TMP/vendor-dmenu" "$TMP/vendor/bin/dmenu.bin"
put "$TMP/sys/class/graphics/fb0/virtual_size" '640,960'
put "$TMP/sys/class/graphics/fb0/bits_per_pixel" '32'
put "$TMP/sys/class/input/event1/device/name" 'ANBERNIC-keys'
put "$TMP/sys/class/input/event1/device/capabilities/key" '7fff000000000000 0'
put "$TMP/sys/class/input/event1/device/capabilities/abs" '30000'
put "$TMP/sys/class/power_supply/axp2202-battery/hallkey" '1'
put "$TMP/sys/class/power_supply/axp2202-battery/capacity" '87'
put "$TMP/sys/class/power_supply/axp2202-battery/moto" '0'
put "$TMP/sys/class/leds/led0/brightness" '255'
put "$TMP/debug/gpio" ' gpio-259 (                    |Headphone detection ) in  lo'
put "$TMP/proc/bus/input/devices" 'N: Name="ANBERNIC-keys"'
put "$TMP/proc/modules" '8821cs 2863104 0 - Live 0x0
hidp 20480 0 - Live 0x0'
mkdir -p "$TMP/sys/class/net/wlan0" "$TMP/sys/class/net/wlan1"
: > "$TMP/dev/uhid"
for c in aplay amixer ip iw wpa_cli; do fake "$c"; done
put "$TMP/proc/sys/kernel/hostname" 'rg35xxsp'
: > "$TMP/card/System/cartridge"

probe() {
	PATH="$TMP/bin:$PATH" \
		AGS_CARD="$TMP/card" AGS_ETC="$TMP/etc" AGS_VENDOR="$TMP/vendor" \
		AGS_SYSFS="$TMP/sys" AGS_PROC="$TMP/proc" AGS_DEBUGFS="$TMP/debug" AGS_DEV="$TMP/dev" \
		sh "$SCRIPT" "$@"
}

out="$(probe -)"
has() {
	printf '%s\n' "$out" | grep -qF -- "$1" || fail "report lacks: $1"
}
has 'BASEOS_TARGET=rg35xxsp'
has 'dmenu.bin model = RG35xxSP'
has 'fb0/virtual_size = 640,960'
has 'event1/device/name = ANBERNIC-keys'
has 'capabilities/abs = 30000'
has 'axp2202-battery/hallkey = 1'
has 'axp2202-battery/moto = 0'
has 'led0/brightness = 255'
has '|Headphone detection'
has 'fake aplay -l'
has 'fake amixer -c 0 scontrols'
has 'wlan1'
has "$TMP/dev/uhid present"
has 'hidp 20480'
has 'bluetoothctl = '
has "wpa_cli = $TMP/bin/wpa_cli"
has 'hostname = rg35xxsp'

file="$(probe)"
case "$file" in
"$TMP/card/probe-rg35xxsp-"*.txt) ;;
*) fail "default report path is $file" ;;
esac
[ -s "$file" ] || fail "report file $file is empty"
[ -e "$file.tmp" ] && fail "temporary report left behind"

rm -f "$TMP/etc/baseos-release"
file="$(probe)"
case "$file" in
"$TMP/card/probe-unknown-"*.txt) ;;
*) fail "report without baseos-release is $file" ;;
esac

named="$TMP/named.txt"
[ "$(probe "$named")" = "$named" ] || fail "an explicit path was not echoed back"
[ -s "$named" ] || fail "explicit report path is empty"

if [ "$FAILS" -gt 0 ]; then
	echo "probe: $FAILS failed"
	exit 1
fi
echo "probe: all passed"
