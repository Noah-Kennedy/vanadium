| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `hyperspectra-cli convert -i /data/bench-data/small.bip -n /data/bench-data/small.bip.hdr -o /data/out.bip -t bip` | 43.514 ± 0.686 | 42.942 | 45.253 | 1.18 ± 0.05 |
| `hyperspectra-cli convert -i /data/bench-data/small.bip -n /data/bench-data/small.bip.hdr -o /data/out.bsq -t bsq` | 42.939 ± 0.136 | 42.744 | 43.276 | 1.16 ± 0.05 |
| `hyperspectra-cli convert -i /data/bench-data/small.bsq -n /data/bench-data/small.bsq.hdr -o /data/out.bip -t bip` | 36.873 ± 1.485 | 34.005 | 39.235 | 1.00 |
| `hyperspectra-cli convert -i /data/bench-data/small.bsq -n /data/bench-data/small.bsq.hdr -o /data/out.bsq -t bsq` | 37.320 ± 0.703 | 36.625 | 38.687 | 1.01 ± 0.04 |
