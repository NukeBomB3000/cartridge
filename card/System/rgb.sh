#!/bin/sh
# The stick lights of the RG40XX H, the RG40XX V and the RG CubeXX. They hang off a small
# controller on UART5 that the axp2202's mcu_pwr rail powers; a frame is written to it as
#
#   <mode> <brightness> <payload> <checksum>
#
# mode 1 is a solid colour, 2, 3 and 4 breathe fast, medium and slow, 5 and 6 are a rainbow of
# one colour or of many. Brightness is a byte. A solid or breathing frame carries 16 RGB triplets,
# the right bank of eight first and then the left; a rainbow's payload is 1, 1 and a speed byte.
# The checksum is the sum of all that comes before it, modulo 256. The V has one stick and one
# light, which takes the same colour as the left bank.
#
#   rgb.sh solid R G B [BRIGHT]       a colour, each 0 to 255, BRIGHT 0 to 100 (default 60)
#   rgb.sh breath fast|medium|slow R G B [BRIGHT]
#   rgb.sh rainbow [BRIGHT [SPEED]]   SPEED 0 to 255, default 128
#   rgb.sh off                        dark, and the controller's power cut
#
# Exit codes: 0 done, 1 this handheld has no stick lights, 2 a verb or value this script does
# not take.
set -u

PSY="${AGS_PSY:-/sys/class/power_supply/axp2202-battery}"
PWR="$PSY/mcu_pwr"
TTY="${AGS_RGB_TTY:-/dev/ttyS5}"

[ -e "$PWR" ] && [ -e "$TTY" ] || exit 1

# the models that have lights; any other with a UART5 must not have frames written to it
target="$(sed -n 's/^BASEOS_TARGET=//p' "${AGS_BASEOS_RELEASE:-/etc/baseos-release}" 2>/dev/null)"
case "$target" in
rg40xxh | rg40xxv | rgcubexx) ;;
*) exit 1 ;;
esac

SUM=0
OUT=""

# appends byte $1 to the frame being built
put() {
	b=$(($1 & 255))
	SUM=$((SUM + b))
	OUT="$OUT$(printf '\\%03o' "$b")"
}

num() {
	case "$1" in
	'' | *[!0-9]*) exit 2 ;;
	esac
	[ "$1" -le "$2" ] || exit 2
}

# writes the frame in $OUT with its checksum, after the controller has power and the port is set
write_frame() {
	if [ "$(cat "$PWR" 2>/dev/null)" != 1 ]; then
		echo 1 > "$PWR"
		sleep 0.2
	fi
	stty -F "$TTY" 115200 raw -echo cs8 -cstopb -parenb -crtscts 2>/dev/null
	body="$OUT$(printf '\\%03o' $((SUM & 255)))"
	# one writer at a time
	(
		flock -n 9 || exit 0
		printf "$body" > "$TTY"
	) 9> "${AGS_RUN:-/run}/cartridge-rgb.lock"
}

bright() {
	b="${1:-60}"
	num "$b" 100
	echo $((b * 255 / 100))
}

colours() {
	# 16 triplets: both banks the same
	i=0
	while [ "$i" -lt 16 ]; do
		put "$1"
		put "$2"
		put "$3"
		i=$((i + 1))
	done
}

case "${1:-}" in
solid)
	[ $# -ge 4 ] || exit 2
	num "$2" 255; num "$3" 255; num "$4" 255
	put 1
	put "$(bright "${5:-60}")"
	colours "$2" "$3" "$4"
	write_frame
	;;
breath)
	[ $# -ge 5 ] || exit 2
	case "$2" in
	fast) m=2 ;;
	medium) m=3 ;;
	slow) m=4 ;;
	*) exit 2 ;;
	esac
	num "$3" 255; num "$4" 255; num "$5" 255
	put "$m"
	put "$(bright "${6:-60}")"
	colours "$3" "$4" "$5"
	write_frame
	;;
rainbow)
	num "${3:-128}" 255
	put 6
	put "$(bright "${2:-60}")"
	put 1
	put 1
	put "${3:-128}"
	write_frame
	;;
off)
	put 1
	put 0
	colours 0 0 0
	write_frame
	sleep 0.1
	echo 0 > "$PWR"
	;;
*)
	exit 2
	;;
esac
