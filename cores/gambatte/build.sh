#!/bin/sh
# Builds the Gambatte Game Boy and Game Boy Color libretro core from its pinned source archive,
# after slot-fragile's and roygravy/slot's. On an x86_64 host CROSS=aarch64-linux-gnu builds for
# the device with that cross toolchain.
#
#   build.sh stamp COMMIT | build COMMIT TARBALL WORKDIR OUT
set -eu

device_cflags="-mcpu=cortex-a53 -mtune=cortex-a53"
recipe="make platform=unix"

usage() {
	echo "usage: $0 stamp COMMIT | build COMMIT TARBALL WORKDIR OUT" >&2
	exit 2
}

stamp() {
	echo "commit=$1"
	echo "source=https://github.com/libretro/gambatte-libretro/archive/$1.tar.gz"
	echo "recipe=$recipe"
	echo "device_cflags=$device_cflags"
}

build() {
	commit="$1" tarball="$2" work="$3" out="$4"
	src="$work/gambatte-libretro-$commit"
	rm -rf "$work"
	mkdir -p "$work"
	tar -xzf "$tarball" -C "$work"
	[ -f "$src/Makefile" ] && [ -f "$src/COPYING" ] ||
		{ echo "$tarball does not hold gambatte-libretro-$commit" >&2; exit 1; }
	cross=""
	if [ -n "${CROSS:-}" ]; then
		cross="CC=$CROSS-gcc CXX=$CROSS-g++ AR=$CROSS-ar"
	fi
	# shellcheck disable=SC2086
	(cd "$src" && CFLAGS="$device_cflags" CXXFLAGS="$device_cflags" $recipe $cross \
		GIT_VERSION="\" $(printf %s "$commit" | cut -c1-7)\"" -j"$(getconf _NPROCESSORS_ONLN)")
	mkdir -p "$(dirname "$out")"
	cp "$src/gambatte_libretro.so" "$out"
	stamp "$commit" >"$out.meta"
}

case "${1:-}" in
stamp)
	[ $# -eq 2 ] && [ -n "$2" ] || usage
	stamp "$2"
	;;
build)
	[ $# -eq 5 ] || usage
	build "$2" "$3" "$4" "$5"
	;;
*)
	usage
	;;
esac
