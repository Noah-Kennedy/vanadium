#!/usr/bin/env bash

PCA='bins/siproc /data/bench-data/siproc/small-{type} /data/small.{type}.pca.siproc.csv --pca'
WARMUP='rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true'
COLD_UP='rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true; sync; echo 3 | tee /proc/sys/vm/drop_caches'

CONVERT_IN='bins/siproc /data/bench-data/siproc/small-{type}'

CONVERT_OUT_BIP='data/siproc-convert.bip --convert bip'
CONVERT_OUT_BIL='data/siproc-convert.bil --convert bil'
CONVERT_OUT_BSQ='data/siproc-convert.bsq --convert bsq'

time hyperfine -u second -i --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/siproc/SIPROC_BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bsq,bil \
  "$CONVERT_IN $CONVERT_OUT_BIP" \
  "$CONVERT_IN $CONVERT_OUT_BIL" \
  "$CONVERT_IN $CONVERT_OUT_BSQ"

time hyperfine -u second -i --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/siproc/SIPROC_BENCHMARKS_PCA_WARM.md \
  -L type bip,bsq,bil \
  "$PCA"

time hyperfine -u second -i --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/siproc/SIPROC_BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bsq,bil \
  "$CONVERT_IN $CONVERT_OUT_BIP" \
  "$CONVERT_IN $CONVERT_OUT_BIL" \
  "$CONVERT_IN $CONVERT_OUT_BSQ"

time hyperfine -u second --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/siproc/SIPROC_BENCHMARKS_PCA_COLD.md \
  -L type bip,bsq,bil \
  "$PCA"

rm /data/*.png /data/*.bil /data/*.bip /data/*.bsq /data/*.csv || true

chown noah ./benchmark-results/siproc/*.md
chmod 755 ./benchmark-results/siproc/*.md
