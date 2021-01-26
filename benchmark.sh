#!/usr/bin/env bash

COLOR='hyperspectra-cli color -i data/raw/unnormalized/unnorm.{type} -n data/raw/unnormalized/unnorm.{type}.hdr'

RGB='-o rgb.png -m 0 0 0 -x 0.5 0.5 1 -b 1 3 4 -c rgb'
COOL='-o cool.png -m 0 -x 1 -b 3 -c coolwarm'
GREY='-o grey.png -m 0 -x 1 -b 3 -c grey'

CONVERT_IN='hyperspectra-cli convert -i data/raw/unnormalized/unnorm.{type} -n data/raw/unnormalized/unnorm.{type}.hdr'

CONVERT_OUT_BIP='-o out.bip -t bip'
CONVERT_OUT_BIL='-o out.bil -t bil'
CONVERT_OUT_BSQ='-o out.bsq -t bsq'

time hyperfine --warmup=2 \
  --prepare 'rm *.png *.bil *.bip *.bsq || true' \
  --export-markdown BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bil,bsq \
  "$CONVERT_IN $CONVERT_OUT_BIP" \
  "$CONVERT_IN $CONVERT_OUT_BIL" \
  "$CONVERT_IN $CONVERT_OUT_BSQ"

time hyperfine --warmup=2 \
  --prepare 'rm *.png *.bil *.bip *.bsq || true' \
  --export-markdown BENCHMARKS_COLOR_WARM.md \
  -L type bip,bil,bsq \
  "$COLOR $RGB" \
  "$COLOR $COOL" \
  "$COLOR $GREY"

time hyperfine --warmup=2 \
  --prepare 'rm *.png *.bil *.bip *.bsq || true; sync; echo 3 | tee /proc/sys/vm/drop_caches' \
  --export-markdown BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bil,bsq \
  "$CONVERT_IN $CONVERT_OUT_BIP" \
  "$CONVERT_IN $CONVERT_OUT_BIL" \
  "$CONVERT_IN $CONVERT_OUT_BSQ"

time hyperfine --warmup=2 \
  --prepare 'rm *.png *.bil *.bip *.bsq || true; sync; echo 3 | tee /proc/sys/vm/drop_caches' \
  --export-markdown BENCHMARKS_COLOR_COLD.md \
  -L type bip,bil,bsq \
  "$COLOR $RGB" \
  "$COLOR $COOL" \
  "$COLOR $GREY"

rm *.png *.bil *.bip *.bsq || true

chown noah *.md
chmod 755 *.md
