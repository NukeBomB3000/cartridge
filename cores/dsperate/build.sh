#!/bin/sh
# Builds the DSperate Nintendo DS libretro core: DSperate's source at a pinned commit, with the
# libretro frontend here (from fragilesilver/slot-fragile, frameskip added) as one more target.
# On an x86_64 host CROSS=aarch64-linux-gnu builds for the device with that cross toolchain.
#
#   build.sh stamp COMMIT | build COMMIT TARBALL WORKDIR OUT
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
tune="-mcpu=cortex-a53"

usage() {
	echo "usage: $0 stamp COMMIT | build COMMIT TARBALL WORKDIR OUT" >&2
	exit 2
}

sha256() {
	if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1"; else shasum -a 256 "$1"; fi |
		cut -d' ' -f1
}

stamp() {
	echo "commit=$1"
	echo "source=https://github.com/beebono/DSperate/archive/$1.tar.gz"
	echo "tune=$tune"
	for f in libretro.cpp libretro.h CMakeLists.libretro.txt; do
		echo "frontend=$f sha256:$(sha256 "$here/$f")"
	done
}

build() {
	commit="$1" tarball="$2" work="$3" out="$4"
	src="$work/DSperate-$commit"
	rm -rf "$work"
	mkdir -p "$work"
	tar -xzf "$tarball" -C "$work"
	[ -f "$src/CMakeLists.txt" ] || { echo "$tarball does not hold DSperate-$commit" >&2; exit 1; }

	mkdir -p "$src/src/frontend/libretro"
	cp "$here/libretro.cpp" "$here/libretro.h" "$src/src/frontend/libretro/"
	cp "$here/CMakeLists.libretro.txt" "$src/src/frontend/libretro/CMakeLists.txt"
	echo 'add_subdirectory(src/frontend/libretro)' >>"$src/CMakeLists.txt"

	cross=""
	if [ -n "${CROSS:-}" ]; then
		cross="-DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=aarch64
			-DCMAKE_C_COMPILER=$CROSS-gcc -DCMAKE_CXX_COMPILER=$CROSS-g++
			-DCMAKE_AR=$(command -v $CROSS-gcc-ar) -DCMAKE_RANLIB=$(command -v $CROSS-gcc-ranlib)
			-DCMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER"
	fi
	# shellcheck disable=SC2086
	cmake -S "$src" -B "$work/build" -DCMAKE_BUILD_TYPE=Release $cross \
		-DDSPERATE_TUNE="$tune" -DDSPERATE_TESTS=OFF -DDSPERATE_HEADLESS=OFF -DDSPERATE_SDL=OFF \
		-DDSPERATE_WAYLAND=OFF -DDSPERATE_CHEEVOS=OFF -DDSPERATE_NET=OFF \
		-DCMAKE_POSITION_INDEPENDENT_CODE=ON >/dev/null
	cmake --build "$work/build" --target dsperate_libretro --parallel "$(getconf _NPROCESSORS_ONLN)"

	mkdir -p "$(dirname "$out")"
	cp "$work/build/src/frontend/libretro/dsperate_libretro.so" "$out"
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
