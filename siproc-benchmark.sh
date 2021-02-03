#!/usr/bin/env bash

DEVICE='laptop'
IN_DIR='/bench-data'
OUT_DIR='.'

PCA_SMALL="bins/siproc $IN_DIR/small-{type} $OUT_DIR/small.{type}.pca.siproc.csv --pca --cuda -1"
PCA_MED="bins/siproc $IN_DIR/medium-{type} $OUT_DIR/medium.{type}.csv --pca --cuda -1"

WARMUP="rm $OUT_DIR/*.png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true"
COLD_UP="rm $OUT_DIR/*.png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true; sync; echo 3 | tee /proc/sys/vm/drop_caches"

CONVERT_IN_SMALL="bins/siproc $IN_DIR/small-{type}"
CONVERT_IN_MED="bins/siproc $IN_DIR/medium-{type}"

CONVERT_OUT_BIP="$OUT_DIR/siproc-convert.bip --convert bip"
CONVERT_OUT_BIL="$OUT_DIR/siproc-convert.bil --convert bil"
CONVERT_OUT_BSQ="$OUT_DIR/siproc-convert.bsq --convert bsq"

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
#
#time hyperfine -u second -i --warmup=2 \
#  --prepare "$WARMUP" \
#  --export-markdown benchmark-results/"$DEVICE"/siproc/medium/BENCHMARKS_CONVERT_WARM.md \
#  -L type bip,bil,bsq \
#  "$CONVERT_IN_MED $CONVERT_OUT_BIP" \
#  "$CONVERT_IN_MED $CONVERT_OUT_BIL" \
#  "$CONVERT_IN_MED $CONVERT_OUT_BSQ"
#
#rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
#chown --recursive noah ./benchmark-results/siproc/
#chmod --recursive 755 ./benchmark-results/siproc/
#
#time hyperfine -u second -i --warmup=2 \
#  --prepare "$WARMUP" \
#  --export-markdown benchmark-results/"$DEVICE"/siproc/medium/BENCHMARKS_PCA_WARM.md \
#  -L type bip,bil,bsq \
#  "$PCA_MED"
#
#rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
#chown --recursive noah ./benchmark-results/siproc/
#chmod --recursive 755 ./benchmark-results/siproc/
#
#time hyperfine -u second -i --warmup=2 \
#  --prepare "$COLD_UP" \
#  --export-markdown benchmark-results/"$DEVICE"/siproc/medium/BENCHMARKS_CONVERT_COLD.md \
#  -L type bip,bil,bsq \
#  "$CONVERT_IN_MED $CONVERT_OUT_BIP" \
#  "$CONVERT_IN_MED $CONVERT_OUT_BIL" \
#  "$CONVERT_IN_MED $CONVERT_OUT_BSQ"
#
#rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
#chown --recursive noah ./benchmark-results/siproc/
#chmod --recursive 755 ./benchmark-results/siproc/
#
#time hyperfine -u second -i --warmup=2 \
#  --prepare "$COLD_UP" \
#  --export-markdown benchmark-results/"$DEVICE"/siproc/medium/BENCHMARKS_PCA_COLD.md \
#  -L type bip,bil,bsq \
#  "$PCA_MED"
#
#rm /data/*png /data/*bil /data/*bip /data/*bsq /data/*.csv || true
#chown --recursive noah ./benchmark-results/siproc/
#chmod --recursive 755 ./benchmark-results/siproc/
