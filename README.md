# Hyperspectra

`hyperspectra` is a cli tool for manipulating ENVI BIP, BIL, and BSQ files for processing
hyperspectral image data.

## Installation

### Installing From Source
#### Installing up the Rust toolchains
Use the Rustup toolchain installer/manager to download and install the toolchains.
Navigate to [rustup.rs](https://rustup.rs/) and follow the instructions provided.

#### Building and Installing Hyperspectral
In your terminal of choice:
```shell script
git clone https://github.com/Noah-Kennedy/hyperspectral.git
cd hyperspectral
RUSTFLAGS="-C target-cpu=native" cargo install --path .
```

## Usage
For help, invoke:
```shell script
hyperspectra --help
```

### Examples
#### Conversion
```shell script
hyperspectra convert -i input.bsq -n input.hdr -o out.bil -t bil
```

#### Image Rendering
##### RGB

```shell script
hyperspectra color -i input.bsq -n input.hdr -o rgb.png -m 0 0 0 -x 0.5 0.5 1 -b 1 3 4 -c rgb
```

##### Grayscale

```shell script
hyperspectra color -i input.bsq -n input.hdr -o gray.png -m 0 -x 0.5 -b 3 -c gray
```

##### Coolwarm

```shell script
hyperspectra color -i input.bsq -n input.hdr -o coolwarm.png -m 0 -x 0.5 -b 3 -c coolwarm
```

##### Masking

```shell script
hyperspectra color -i input.bsq -n input.hdr -o mask.png -m 0 -x 1.0 -b 3 -c mask
```

## Design
TODO

## Benchmarks
Benchmarks were performed on a machine running Arch Linux with a Ryzen 3900X, an Nvidia RTX 2080 TI,
64 GiB of Ram, and SSD storage.
The machine was running a Linux 5.4 LTS kernel release.
Some testing (not formally benchmarked yet) shows that this <b>might</b> run faster on a 5.8+ kernel
ir with `mitigations=off` in `grub.cfg`.

Input files had 5 bands and were 28740 x 21954.

Warm cache benchmarks occurred after the program was run twice before measured runs began, in order
to ensure that the file cache had been prepared.
Cold cache benchmarks flushed the file cache before each run, ensuring that no files were present
in the cache when the run began.

### Conversion
#### Warm Cache
| Input| Output | Mean [s] | Min [s] | Max [s] |
|:---|:---|---:|---:|---:|
| bip | bip | 16.744 ± 0.407 | 16.070 | 17.234 |
| bip | bil | 14.597 ± 0.429 | 13.714 | 15.171 |
| bip | bsq | 14.504 ± 0.396 | 13.639 | 14.944 |
| bil | bip | 12.945 ± 0.859 | 11.918 | 14.283 |
| bil | bil | 14.146 ± 0.399 | 13.631 | 14.730 |
| bil | bsq | 13.746 ± 0.874 | 12.111 | 14.829 |
| bsq | bip | 10.918 ± 0.458 | 10.395 | 11.771 |
| bsq | bil | 13.978 ± 0.554 | 12.851 | 14.742 |
| bsq | bsq | 13.941 ± 0.747 | 12.380 | 15.100 |

#### Cold Cache
| Input| Output | Mean [s] | Min [s] | Max [s] |
|:---|:---|---:|---:|---:|
| bip | bip | 39.153 ± 1.239 | 37.339 | 42.041 |
| bip | bil | 37.684 ± 0.648 | 36.101 | 38.228 |
| bip | bsq | 37.709 ± 0.210 | 37.399 | 38.023 |
| bil | bip | 36.161 ± 0.980 | 35.244 | 38.292 |
| bil | bil | 37.131 ± 0.426 | 36.210 | 37.637 |
| bil | bsq | 37.029 ± 0.266 | 36.659 | 37.382 |
| bsq | bip | 34.321 ± 0.396 | 33.436 | 34.706 |
| bsq | bil | 36.851 ± 0.395 | 36.236 | 37.493 |
| bsq | bsq | 36.993 ± 0.244 | 36.629 | 37.259 |

### Colorization
#### Warm Cache
| Input| Output | Mean [s] | Min [s] | Max [s] |
|:---|:---|---:|---:|---:|
| bip | rgb         | 23.540 ± 0.052 | 23.478 | 23.642 |
| bip | coolwarm    | 17.000 ± 1.101 | 16.488 | 20.015 |
| bip | grey        | 8.746 ± 0.196  |  8.647 |  9.292 |
| bil | rgb         | 23.251 ± 0.040 | 23.197 | 23.329 |
| bil | coolwarm    | 16.711 ± 1.206 | 16.188 | 20.060 |
| bil | grey        | 8.878 ± 0.079  |  8.832 |  9.094 |
| bsq | rgb         | 23.405 ± 0.530 | 23.187 | 24.911 |
| bsq | coolwarm    | 16.758 ± 1.245 | 16.142 | 20.089 |
| bsq | grey        | 8.995 ± 0.308  |  8.823 |  9.625 |

#### Cold Cache
| Input| Output | Mean [s] | Min [s] | Max [s] |
|:---|:---|---:|---:|---:|
| bip | rgb      | 46.864 ± 0.686 | 46.472 | 48.173 |
| bip | coolwarm | 40.113 ± 1.171 | 39.440 | 43.336 |
| bip | grey     | 31.969 ± 0.511 | 31.626 | 33.259 |
| bil | rgb      | 46.203 ± 0.051 | 46.131 | 46.262 |
| bil | coolwarm | 39.165 ± 0.050 | 39.087 | 39.248 |
| bil | grey     | 31.880 ± 0.380 | 31.694 | 32.953 |
| bsq | rgb      | 46.297 ± 0.243 | 46.110 | 46.926 |
| bsq | coolwarm | 39.181 ± 0.080 | 39.053 | 39.322 |
| bsq | grey     | 32.010 ± 0.323 | 31.743 | 32.497 |

## Testing
Tests can be run by invoking `cargo test` on the command line.

Benchmarks currently look for files in `./bench-data/` named `small.bil`, `small.bip`, `small.bsq`,
`small.bil.hdr`, `small.bip.hdr`, `small.bsq.hdr` and will delete any png, bip, bil, or bsq files
in the directory in which the benchmarks are run. Benchmarks use the `hyperfine` tool.