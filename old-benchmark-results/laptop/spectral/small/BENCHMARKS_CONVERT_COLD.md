| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `hyperspectra-cli convert -i bench-data/small-bip -n bench-data/small-bip.hdr -o ./out-bip -t bip` | 36.971 ± 1.292 | 34.505 | 38.229 | 1.33 ± 0.07 |
| `hyperspectra-cli convert -i bench-data/small-bip -n bench-data/small-bip.hdr -o ./out-bsq -t bsq` | 33.457 ± 1.644 | 31.755 | 36.403 | 1.21 ± 0.07 |
| `hyperspectra-cli convert -i bench-data/small-bsq -n bench-data/small-bsq.hdr -o ./out-bip -t bip` | 32.014 ± 0.630 | 31.340 | 33.588 | 1.15 ± 0.05 |
| `hyperspectra-cli convert -i bench-data/small-bsq -n bench-data/small-bsq.hdr -o ./out-bsq -t bsq` | 27.753 ± 1.040 | 26.164 | 29.285 | 1.00 |
