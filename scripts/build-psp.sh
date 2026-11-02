#!/bin/sh
# Builds what PSP games need beside the frontend: the stand-alone PPSSPP, which draws on the
# screen with OpenGL ES through the Mali driver, wants an SDL that has one, and the handhelds'
# own SDL has none (it is 32-bit besides). Run from the repository root, with docker:
#
#   scripts/build-psp.sh PSP.pak.zip OUT
#
# writes OUT/System/psp/ for the card's System/: PPSSPP and its assets (from the PSP pak for
# NextUI at https://github.com/ben16w/minui-psp, whose zip is the argument: PPSSPP is GPL and is
# not kept in this repository), the SDL built here, and padmap, which tells PPSSPP what the
# buttons are.
#
# The SDL is 2.30.6 with the handhelds' Mali video driver as Knulli's H700 builds carry it
# (https://github.com/knulli-cfw/knulli-linux, the commit below), and third_party/psp/
# sdl-cartridge.patch on it, which makes the driver draw the volume gauge cartridge draws, and
# the audio thread scale the sound by the volume's square, as cartridge does. All of it is
# zlib-licensed SDL code.
set -eu

KNULLI=7dabcbbfaa93512530e99a130e40ab1e4166987a
PATCHES="0001-add-video-malifb-driver 0002-enable_no_signal_handlers 0002-mali-fbdev-Implemented-MALI-Zero-Copy-fbdev-rotation 0003-No-need-to-set-glClearColor-at-all 0004-Rework-blitter-thread-sync 0005-additional-mali-fixes 0006-refactoring 0007-no-MSAA 0008-add-sRGB 0009-memleak-fix 0010-blitter-only-with-rotation"

zip="${1:?usage: build-psp.sh PSP.pak.zip OUT}"
out="${2:?usage: build-psp.sh PSP.pak.zip OUT}"
here="$(cd "$(dirname "$0")/.." && pwd)"
zip="$(cd "$(dirname "$zip")" && pwd)/$(basename "$zip")"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT HUP INT TERM

# the libEGL the handheld has is what the SDL links against: the real one is its Mali driver's
if [ -z "${EGL_STUB:-}" ]; then
	echo "build-psp: EGL_STUB, a libEGL.so.1 from the handheld (adb pull /usr/lib/libEGL.so.1), is needed to link" >&2
	exit 1
fi
cp "$EGL_STUB" "$work/libEGL.so"
cp "$here/third_party/psp/sdl-cartridge.patch" "$here/third_party/psp/padmap.c" "$work/"

cat > "$work/build.sh" <<EOF
set -e
sed -i "/security/d" /etc/apt/sources.list
dpkg --add-architecture arm64
apt-get update -qq >/dev/null
apt-get install -y -qq gcc-aarch64-linux-gnu cmake make git curl patch >/dev/null
mkdir -p /sysroot /tmp/d && cd /tmp/d
apt-get download libasound2:arm64 libasound2-dev:arm64 >/dev/null
for f in *.deb; do dpkg -x \$f /sysroot; done
cd /work
git clone -q --depth 1 --branch release-2.30.6 https://github.com/libsdl-org/SDL.git SDL
cd SDL
for p in $PATCHES; do
  curl -fsSL https://raw.githubusercontent.com/knulli-cfw/knulli-linux/$KNULLI/board/allwinner/h700/patches/sdl2/\$p.patch | patch -p1 -s
done
patch -p1 -s < /work/sdl-cartridge.patch
mkdir -p build && cd build
cmake .. -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=aarch64 \\
  -DCMAKE_C_COMPILER=aarch64-linux-gnu-gcc -DCMAKE_BUILD_TYPE=Release \\
  -DCMAKE_C_FLAGS="-I/work/SDL/src/video/khronos -I/sysroot/usr/include" \\
  -DCMAKE_SHARED_LINKER_FLAGS=-L/work \\
  -DSDL_MALI=ON -DVIDEO_MALI=ON -DSDL_X11=OFF -DSDL_WAYLAND=OFF -DSDL_KMSDRM=OFF -DSDL_VIVANTE=OFF \\
  -DSDL_OPENGL=OFF -DSDL_OPENGLES=ON -DSDL_VULKAN=OFF -DSDL_PULSEAUDIO=OFF -DSDL_JACK=OFF \\
  -DSDL_PIPEWIRE=OFF -DSDL_SNDIO=OFF -DSDL_ALSA=ON -DSDL_ALSA_SHARED=ON -DSDL_STATIC=OFF \\
  -DSDL_TEST=OFF -DSDL_LIBUDEV=OFF -DSDL_DBUS=OFF -DSDL_IBUS=OFF -DSDL_RPATH=OFF \\
  -DALSA_INCLUDE_DIR=/sysroot/usr/include -DALSA_LIBRARY=/sysroot/usr/lib/aarch64-linux-gnu/libasound.so.2 >/dev/null
make -j8 >/dev/null
cp libSDL2-2.0.so.0.3000.6 /work/libSDL2-2.0.so.0
cd /work
aarch64-linux-gnu-gcc padmap.c -ISDL/include -o padmap -Wl,--unresolved-symbols=ignore-in-shared-libs SDL/build/libSDL2-2.0.so.0.3000.6
chown -R $(id -u):$(id -g) /work
EOF
docker run --rm --platform linux/amd64 -v "$work":/work -w /work rust:1-bullseye sh /work/build.sh

dest="$out/System/psp"
mkdir -p "$dest"
unzip -q -o "$zip" 'PPSSPP/*' -d "$work/pak"
cp -R "$work/pak/PPSSPP/." "$dest/"
mv "$dest/PPSSPPSDL_tg5040" "$dest/PPSSPPSDL"
rm -f "$dest/PPSSPPSDL_tg5050"
cp "$work/libSDL2-2.0.so.0" "$work/padmap" "$dest/"
chmod 755 "$dest/PPSSPPSDL" "$dest/padmap"
echo "build-psp: $dest"
