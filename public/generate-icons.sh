#!/bin/sh
# Regenerate all favicon/icon assets from mybd.svg
set -eu

cd "$(dirname "$0")"

# Rasterize at high density then downscale for sharp results
OPTS="-background none -density 1200 mybd.svg -filter Lanczos"

magick $OPTS -resize 16x16   favicon-16x16.png
magick $OPTS -resize 32x32   favicon-32x32.png
magick $OPTS -resize 180x180 apple-touch-icon.png
magick $OPTS -resize 192x192 icon-192.png
magick $OPTS -resize 512x512 icon-512.png

magick $OPTS -resize 16x16 \
       $OPTS -resize 32x32 \
       $OPTS -resize 48x48 \
       favicon.ico

echo "Icons regenerated from mybd.svg"
