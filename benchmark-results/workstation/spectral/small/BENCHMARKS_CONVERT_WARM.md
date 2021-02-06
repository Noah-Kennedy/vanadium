| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `hyperspectra-cli convert -i /data/bench-data/small.bip -n /data/bench-data/small.bip.hdr -o /data/out.bip -t bip` | 9.996 ± 0.058 | 9.922 | 10.122 | 1.20 ± 0.02 |
| `hyperspectra-cli convert -i /data/bench-data/small.bip -n /data/bench-data/small.bip.hdr -o /data/out.bsq -t bsq` | 8.386 ± 0.158 | 8.049 | 8.613 | 1.01 ± 0.02 |
| `hyperspectra-cli convert -i /data/bench-data/small.bsq -n /data/bench-data/small.bsq.hdr -o /data/out.bip -t bip` | 9.154 ± 0.446 | 8.681 | 9.967 | 1.10 ± 0.06 |
| `hyperspectra-cli convert -i /data/bench-data/small.bsq -n /data/bench-data/small.bsq.hdr -o /data/out.bsq -t bsq` | 8.331 ± 0.126 | 8.166 | 8.526 | 1.00 |
