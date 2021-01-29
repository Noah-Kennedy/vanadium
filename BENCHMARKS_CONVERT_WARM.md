| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `hyperspectra-cli convert -i /data/bench-data/small.bip -n /data/bench-data/small.bip.hdr -o /data/out.bip -t bip` | 23.545 ± 0.313 | 23.416 | 24.435 | 2.55 ± 0.09 |
| `hyperspectra-cli convert -i /data/bench-data/small.bip -n /data/bench-data/small.bip.hdr -o /data/out.bsq -t bsq` | 9.244 ± 0.337 | 8.328 | 9.601 | 1.00 ± 0.05 |
| `hyperspectra-cli convert -i /data/bench-data/small.bsq -n /data/bench-data/small.bsq.hdr -o /data/out.bip -t bip` | 23.464 ± 0.048 | 23.425 | 23.591 | 2.54 ± 0.08 |
| `hyperspectra-cli convert -i /data/bench-data/small.bsq -n /data/bench-data/small.bsq.hdr -o /data/out.bsq -t bsq` | 9.221 ± 0.296 | 8.387 | 9.383 | 1.00 |
