#!/usr/bin/env bash

PCA_SMALL='hyperspectra-cli pca --csv -i /data/bench-data/small.{type} -h /data/bench-data/small.{type}.hdr -o /data/pca.{type} -d 3 --min 0.0 --max 1.0'
PCA_MED='hyperspectra-cli pca --csv -i /data/bench-data/medium.{type} -h /data/bench-data/medium.{type}.hdr -o /data/pca.{type} -d 3 --min 0.0 --max 1.0'

WARMUP='rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true'
COLD_UP='rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true; sync; echo 3 | tee /proc/sys/vm/drop_caches'

CONVERT_IN_SMALL='hyperspectra-cli convert -i /data/bench-data/small.{type} -n /data/bench-data/small.{type}.hdr'
CONVERT_IN_MED='hyperspectra-cli convert -i /data/bench-data/medium.{type} -n /data/bench-data/medium.{type}.hdr'

CONVERT_OUT_BIP='-o /data/out.bip -t bip'
CONVERT_OUT_BIL='-o /data/out.bil -t bil'
CONVERT_OUT_BSQ='-o /data/out.bsq -t bsq'

# small

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/spectral/small/BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bsq \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BIP" \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BSQ"

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/spectral/small/BENCHMARKS_PCA_WARM.md \
  -L type bip,bsq \
  "$PCA_SMALL"

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/spectral/small/BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bsq \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BIP" \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BSQ"

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/spectral/small/BENCHMARKS_PCA_COLD.md \
  -L type bip,bsq \
  "$PCA_SMALL"

# medium

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/spectral/medium/BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bsq \
  "$CONVERT_IN_MED $CONVERT_OUT_BIP" \
  "$CONVERT_IN_MED $CONVERT_OUT_BSQ"

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/spectral/medium/BENCHMARKS_PCA_WARM.md \
  -L type bip,bsq \
  "$PCA_MED"

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/spectral/medium/BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bsq \
  "$CONVERT_IN_MED $CONVERT_OUT_BIP" \
  "$CONVERT_IN_MED $CONVERT_OUT_BSQ"

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/spectral/medium/BENCHMARKS_PCA_COLD.md \
  -L type bip,bsq \
  "$PCA_MED"

rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true

chown --recursive noah ./benchmark-results/spectral/
chmod --recursive 755 ./benchmark-results/spectral/