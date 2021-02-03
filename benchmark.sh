#!/usr/bin/env bash

DEVICE='laptop'
IN_DIR='bench-data'
OUT_DIR='.'

PCA_SMALL="hyperspectra-cli pca --csv -i $IN_DIR/small.{type} -h $OUT_DIR/small.{type}.hdr -o $OUT_DIR/pca.{type} -d 3 --min 0.0 --max 1.0"
PCA_MED="hyperspectra-cli pca --csv -i $IN_DIR/medium.{type} -h $OUT_DIR/medium.{type}.hdr -o $OUT_DIR/pca.{type} -d 3 --min 0.0 --max 1.0"

WARMUP="rm $OUT_DIR/*.png $OUT_DIR/*.bil $OUT_DIR/*.bip $OUT_DIR/*.bsq $OUT_DIR/*.csv || true"
COLD_UP="rm $OUT_DIR/*.png $OUT_DIR/*.bil $OUT_DIR/*.bip $OUT_DIR/*.bsq $OUT_DIR/*.csv || true; sync; echo 3 | tee /proc/sys/vm/drop_caches"

CONVERT_IN_SMALL="hyperspectra-cli convert -i $IN_DIR/small.{type} -n $IN_DIR/small.{type}.hdr"
CONVERT_IN_MED="hyperspectra-cli convert -i $IN_DIR/medium.{type} -n $IN_DIR/medium.{type}.hdr"

CONVERT_OUT_BIP="-o $OUT_DIR/out.bip -t bip"
CONVERT_OUT_BIL="-o $OUT_DIR/out.bil -t bil"
CONVERT_OUT_BSQ="-o $OUT_DIR/out.bsq -t bsq"

# small

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/"$DEVICE"/spectral/small/BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bsq \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BIP" \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BSQ"

rm $OUT_DIR/*png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true
chown --recursive noah ./benchmark-results/"$DEVICE"/siproc/
chmod --recursive 755 ./benchmark-results/"$DEVICE"/siproc/

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/"$DEVICE"/spectral/small/BENCHMARKS_PCA_WARM.md \
  -L type bip,bsq \
  "$PCA_SMALL"

rm $OUT_DIR/*png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true
chown --recursive noah ./benchmark-results/"$DEVICE"/siproc/
chmod --recursive 755 ./benchmark-results/"$DEVICE"/siproc/

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/"$DEVICE"/spectral/small/BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bsq \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BIP" \
  "$CONVERT_IN_SMALL $CONVERT_OUT_BSQ"

rm $OUT_DIR/*png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true
chown --recursive noah ./benchmark-results/"$DEVICE"/siproc/
chmod --recursive 755 ./benchmark-results/"$DEVICE"/siproc/

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/"$DEVICE"/spectral/small/BENCHMARKS_PCA_COLD.md \
  -L type bip,bsq \
  "$PCA_SMALL"

rm $OUT_DIR/*png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true
chown --recursive noah ./benchmark-results/"$DEVICE"/siproc/
chmod --recursive 755 ./benchmark-results/"$DEVICE"/siproc/

# medium

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/"$DEVICE"/spectral/medium/BENCHMARKS_CONVERT_WARM.md \
  -L type bip,bsq \
  "$CONVERT_IN_MED $CONVERT_OUT_BIP" \
  "$CONVERT_IN_MED $CONVERT_OUT_BSQ"

rm $OUT_DIR/*png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true
chown --recursive noah ./benchmark-results/"$DEVICE"/siproc/
chmod --recursive 755 ./benchmark-results/"$DEVICE"/siproc/

time hyperfine --warmup=2 \
  --prepare "$WARMUP" \
  --export-markdown benchmark-results/"$DEVICE"/spectral/medium/BENCHMARKS_PCA_WARM.md \
  -L type bip,bsq \
  "$PCA_MED"

rm $OUT_DIR/*png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true
chown --recursive noah ./benchmark-results/"$DEVICE"/siproc/
chmod --recursive 755 ./benchmark-results/"$DEVICE"/siproc/

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/"$DEVICE"/spectral/medium/BENCHMARKS_CONVERT_COLD.md \
  -L type bip,bsq \
  "$CONVERT_IN_MED $CONVERT_OUT_BIP" \
  "$CONVERT_IN_MED $CONVERT_OUT_BSQ"

rm $OUT_DIR/*png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true
chown --recursive noah ./benchmark-results/"$DEVICE"/siproc/
chmod --recursive 755 ./benchmark-results/"$DEVICE"/siproc/

time hyperfine --warmup=2 \
  --prepare "$COLD_UP" \
  --export-markdown benchmark-results/"$DEVICE"/spectral/medium/BENCHMARKS_PCA_COLD.md \
  -L type bip,bsq \
  "$PCA_MED"

rm $OUT_DIR/*png $OUT_DIR/*bil $OUT_DIR/*bip $OUT_DIR/*bsq $OUT_DIR/*.csv || true
chown --recursive noah ./benchmark-results/"$DEVICE"/siproc/
chmod --recursive 755 ./benchmark-results/"$DEVICE"/siproc/