| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `hyperspectra-cli convert -i /data/bench-data/small.bip -n /data/bench-data/small.bip.hdr -o /data/out.bip -t bip` | 62.175 ± 0.481 | 61.715 | 63.026 | 1.07 ± 0.01 |
| `hyperspectra-cli convert -i /data/bench-data/small.bip -n /data/bench-data/small.bip.hdr -o /data/out.bsq -t bsq` | 63.545 ± 1.887 | 62.186 | 67.933 | 1.09 ± 0.03 |
| `hyperspectra-cli convert -i /data/bench-data/small.bsq -n /data/bench-data/small.bsq.hdr -o /data/out.bip -t bip` | 64.238 ± 1.163 | 63.289 | 66.465 | 1.10 ± 0.02 |
| `hyperspectra-cli convert -i /data/bench-data/small.bsq -n /data/bench-data/small.bsq.hdr -o /data/out.bsq -t bsq` | 58.326 ± 0.335 | 58.008 | 59.238 | 1.00 |
