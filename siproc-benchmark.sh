#!/usr/bin/env bash

DEVICE='laptop'

PCA_SMALL='bins/siproc /data/bench-data/siproc/small-{type} /data/small.{type}.pca.siproc.csv --pca --cuda -1'
PCA_MED='bins/siproc /data/bench-data/siproc/medium-{type} /data/medium.{type}.csv --pca --cuda -1'

WARMUP='rm /data/*.png /data/*bil /data/*bip /data/*bsq /data/*.csv || true'
COLD_UP='rm /data/*.png /data/*bil /data/*bip /data/*bsq /data/*.csv || true; sync; echo 3 | tee /proc/sys/vm/drop_caches'

CONVERT_IN_SMALL='bins/siproc /data/bench-data/siproc/small-{type}'
CONVERT_IN_MED='bins/siproc /data/bench-data/siproc/medium-{type}'

CONVERT_OUT_BIP='data/siproc-convert.bip --convert bip'
CONVERT_OUT_BIL='data/siproc-convert.bil --convert bil'
CONVERT_OUT_BSQ='data/siproc-convert.bsq --convert bsq'

# small

time hyperfine -u second -i --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/"$DEVICE"/siproc/small/BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bsq,bil \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BIP" \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BIL" \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BSQ"

rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
chown --recursive noah ./benchmark-results/siproc/
chmod --recursive 755 ./benchmark-results/siproc/

time hyperfine -u second -i --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/"$DEVICE"/siproc/small/BENCHMARKS_PCA_WARM.md \
  -L type bip,bsq,bil \
  "$PCA_SMALL"

rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
chown --recursive noah ./benchmark-results/siproc/
chmod --recursive 755 ./benchmark-results/siproc/

time hyperfine -u second -i --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/"$DEVICE"/siproc/small/BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bsq,bil \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BIP" \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BIL" \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BSQ"

rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
chown --recursive noah ./benchmark-results/siproc/
chmod --recursive 755 ./benchmark-results/siproc/

# rerun
time hyperfine -u second -i --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/"$DEVICE"/siproc/small/BENCHMARKS_PCA_COLD.md \
  -L type bip,bsq,bil \
  "$PCA_SMALL"

rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
chown --recursive noah ./benchmark-results/siproc/
chmod --recursive 755 ./benchmark-results/siproc/

# medium

time hyperfine -u second -i --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/"$DEVICE"/siproc/medium/BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bil,bsq \
  "$CONVERT_IN_MED $CONVERT_OUT_BIP" \
  "$CONVERT_IN_MED $CONVERT_OUT_BIL" \
  "$CONVERT_IN_MED $CONVERT_OUT_BSQ"

rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
chown --recursive noah ./benchmark-results/siproc/
chmod --recursive 755 ./benchmark-results/siproc/

time hyperfine -u second -i --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/"$DEVICE"/siproc/medium/BENCHMARKS_PCA_WARM.md \
  -L type bip,bil,bsq \
  "$PCA_MED"

rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
chown --recursive noah ./benchmark-results/siproc/
chmod --recursive 755 ./benchmark-results/siproc/

time hyperfine -u second -i --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/"$DEVICE"/siproc/medium/BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bil,bsq \
  "$CONVERT_IN_MED $CONVERT_OUT_BIP" \
  "$CONVERT_IN_MED $CONVERT_OUT_BIL" \
  "$CONVERT_IN_MED $CONVERT_OUT_BSQ"

rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
chown --recursive noah ./benchmark-results/siproc/
chmod --recursive 755 ./benchmark-results/siproc/

time hyperfine -u second -i --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/"$DEVICE"/siproc/medium/BENCHMARKS_PCA_COLD.md \
  -L type bip,bil,bsq \
  "$PCA_MED"

rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
chown --recursive noah ./benchmark-results/siproc/
chmod --recursive 755 ./benchmark-results/siproc/