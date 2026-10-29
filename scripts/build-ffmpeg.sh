#!/bin/sh
# Builds the FFmpeg the video player runs (System/ffmpeg on the card): FFmpeg's own source from
# ffmpeg.org, its signature checked, cross-compiled for the H700 (aarch64) in docker, LGPL only
# (no GPL parts, nothing non-free), static, with just the decoders, demuxers and filters the
# player needs. Run from the repository's root; the program comes out as target-ffmpeg/ffmpeg.
set -e
VERSION="${FFMPEG_VERSION:-7.1.5}"
# FFmpeg's release signing key
FINGERPRINT="FCF986EA15E6E293A5644F10B4322F04D67658D8"
OUT="$PWD/target-ffmpeg"
mkdir -p "$OUT/src" "$OUT/work"
cd "$OUT/src"
for f in "ffmpeg-$VERSION.tar.xz" "ffmpeg-$VERSION.tar.xz.asc" ffmpeg-devel.asc; do
	case "$f" in
	ffmpeg-devel.asc) url="https://ffmpeg.org/$f" ;;
	*) url="https://ffmpeg.org/releases/$f" ;;
	esac
	[ -f "$f" ] || curl -sSLO "$url"
done
export GNUPGHOME="$OUT/gnupg"
mkdir -p -m 700 "$GNUPGHOME"
gpg --quiet --import ffmpeg-devel.asc
gpg --status-fd 1 --verify "ffmpeg-$VERSION.tar.xz.asc" "ffmpeg-$VERSION.tar.xz" 2>/dev/null \
	| grep -q "VALIDSIG .* $FINGERPRINT\$" || {
	echo "the source's signature is not FFmpeg's release key" >&2
	exit 1
}
cd "$OLDPWD"
docker run --rm --platform linux/amd64 \
	-v "$OUT/src:/src:ro" -v "$OUT/work:/work" -v "$OUT:/out" rust:1-bullseye sh -c "
set -e
sed -i '/security/d' /etc/apt/sources.list
apt-get update -qq >/dev/null
apt-get install -y -qq gcc-aarch64-linux-gnu make xz-utils >/dev/null 2>&1
cd /work
tar xf /src/ffmpeg-$VERSION.tar.xz
cd ffmpeg-$VERSION
./configure --enable-cross-compile --cross-prefix=aarch64-linux-gnu- --arch=aarch64 --target-os=linux \
  --disable-everything --disable-doc --disable-debug --disable-network --disable-autodetect \
  --disable-ffplay --disable-ffprobe --enable-ffmpeg \
  --enable-decoder=h264,hevc,mpeg4,msmpeg4v3,mpeg2video,vp8,vp9,aac,aac_latm,mp3,mp3float,ac3,eac3,flac,vorbis,opus,pcm_s16le \
  --enable-demuxer=mov,matroska,avi,mpegts,mpegps,mp3,flac,ogg,wav,aac \
  --enable-parser=h264,hevc,mpeg4video,mpegvideo,vp8,vp9,aac,aac_latm,mpegaudio,ac3,flac,vorbis,opus \
  --enable-protocol=file,pipe --enable-muxer=null,rawvideo,pcm_s16le \
  --enable-encoder=rawvideo,wrapped_avframe,pcm_s16le \
  --enable-bsf=h264_mp4toannexb,hevc_mp4toannexb,aac_adtstoasc,extract_extradata \
  --enable-filter=null,anull,scale,format,aformat,aresample,fps \
  --enable-swscale --enable-swresample --enable-avfilter --disable-shared --enable-static \
  --extra-cflags=-O3 --extra-ldflags=-static --disable-stripping
make -j\$(nproc) >/dev/null
cp ffmpeg /out/ffmpeg
"
echo "built $OUT/ffmpeg"
