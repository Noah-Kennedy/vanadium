| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `hyperspectra-cli convert -i /data/bench-data/medium.bip -n /data/bench-data/medium.bip.hdr -o /data/out.bip -t bip` | 33.660 ± 0.155 | 33.424 | 33.863 | 1.05 ± 0.03 |
| `hyperspectra-cli convert -i /data/bench-data/medium.bip -n /data/bench-data/medium.bip.hdr -o /data/out.bsq -t bsq` | 32.176 ± 0.793 | 31.311 | 33.264 | 1.00 |
| `hyperspectra-cli convert -i /data/bench-data/medium.bsq -n /data/bench-data/medium.bsq.hdr -o /data/out.bip -t bip` | 35.182 ± 0.763 | 34.678 | 37.314 | 1.09 ± 0.04 |
| `hyperspectra-cli convert -i /data/bench-data/medium.bsq -n /data/bench-data/medium.bsq.hdr -o /data/out.bsq -t bsq` | 32.485 ± 0.235 | 32.016 | 32.805 | 1.01 ± 0.03 |
