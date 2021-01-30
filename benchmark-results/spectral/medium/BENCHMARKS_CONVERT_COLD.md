| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `hyperspectra-cli convert -i /data/bench-data/medium.bip -n /data/bench-data/medium.bip.hdr -o /data/out.bip -t bip` | 91.145 ± 0.458 | 90.402 | 91.905 | 1.11 ± 0.02 |
| `hyperspectra-cli convert -i /data/bench-data/medium.bip -n /data/bench-data/medium.bip.hdr -o /data/out.bsq -t bsq` | 90.285 ± 0.645 | 89.167 | 91.171 | 1.10 ± 0.02 |
| `hyperspectra-cli convert -i /data/bench-data/medium.bsq -n /data/bench-data/medium.bsq.hdr -o /data/out.bip -t bip` | 82.368 ± 1.465 | 79.511 | 84.160 | 1.00 |
| `hyperspectra-cli convert -i /data/bench-data/medium.bsq -n /data/bench-data/medium.bsq.hdr -o /data/out.bsq -t bsq` | 82.600 ± 0.792 | 81.273 | 83.467 | 1.00 ± 0.02 |
