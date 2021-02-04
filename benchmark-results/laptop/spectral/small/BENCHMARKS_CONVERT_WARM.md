| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `hyperspectra-cli convert -i bench-data/small-bip -n bench-data/small-bip.hdr -o ./out-bip -t bip` | 35.836 ± 1.536 | 32.157 | 37.397 | 1.23 ± 0.06 |
| `hyperspectra-cli convert -i bench-data/small-bip -n bench-data/small-bip.hdr -o ./out-bsq -t bsq` | 33.633 ± 1.712 | 31.109 | 36.176 | 1.16 ± 0.07 |
| `hyperspectra-cli convert -i bench-data/small-bsq -n bench-data/small-bsq.hdr -o ./out-bip -t bip` | 32.445 ± 0.425 | 31.735 | 33.011 | 1.11 ± 0.03 |
| `hyperspectra-cli convert -i bench-data/small-bsq -n bench-data/small-bsq.hdr -o ./out-bsq -t bsq` | 29.116 ± 0.747 | 27.708 | 30.169 | 1.00 |
