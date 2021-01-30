#!/usr/bin/env bash

PCA='hyperspectra-cli pca --csv -i /data/bench-data/small.{type} -h /data/bench-data/small.{type}.hdr -o /data/small.pca.{type} -d 3 --min 0.0 --max 1.0'

WARMUP='rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true'
COLD_UP='rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true; sync; echo 3 | tee /proc/sys/vm/drop_caches'

CONVERT_IN='hyperspectra-cli convert -i /data/bench-data/small.{type} -n /data/bench-data/small.{type}.hdr'

CONVERT_OUT_BIP='-o /data/out.bip -t bip'
CONVERT_OUT_BIL='-o /data/out.bil -t bil'
CONVERT_OUT_BSQ='-o /data/out.bsq -t bsq'

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/spectral/BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bsq \
  "$CONVERT_IN $CONVERT_OUT_BIP" \
  "$CONVERT_IN $CONVERT_OUT_BSQ"

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/spectral/BENCHMARKS_PCA_WARM.md \
  -L type bip,bsq \
  "$PCA"

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/spectral/BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bsq \
  "$CONVERT_IN $CONVERT_OUT_BIP" \
  "$CONVERT_IN $CONVERT_OUT_BSQ"

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/spectral/BENCHMARKS_PCA_COLD.md \
  -L type bip,bsq \
  "$PCA"

rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true

chown noah *.md
chmod 755 *.md
