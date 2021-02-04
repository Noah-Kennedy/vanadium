# Hyperspectra

[![codecov](https://codecov.io/gh/Noah-Kennedy/hyperspectral/branch/master/graph/badge.svg?token=2KBFXPBR34)](https://codecov.io/gh/Noah-Kennedy/hyperspectral)

![Continuous integration](https://github.com/Noah-Kennedy/hyperspectral/workflows/Continuous%20integration/badge.svg?branch=master)

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
cd hyperspectra-cli
RUSTFLAGS="-C target-cpu=native" cargo install --path .
```

## Usage
For help, invoke:
```shell script
hyperspectra-cli --help
```

### Examples
#### Conversion
```shell script
hyperspectra-cli convert -i input.bsq -n input.hdr -o out.bil -t bil
```

#### Image Rendering
##### RGB

```shell script
hyperspectra-cli color -i input.bsq -n input.hdr -o rgb.png -m 0 0 0 -x 0.5 0.5 1 -b 1 3 4 -c rgb
```

##### Grayscale

```shell script
hyperspectra-cli color -i input.bsq -n input.hdr -o gray.png -m 0 -x 0.5 -b 3 -c gray
```

##### Coolwarm

```shell script
hyperspectra-cli color -i input.bsq -n input.hdr -o coolwarm.png -m 0 -x 0.5 -b 3 -c coolwarm
```

##### Masking
The max (-x) flag currently is required but does nothing.
```shell script
hyperspectra-cli color -i input.bsq -n input.hdr -o mask.png -m 0 -x 1.0 -c mask
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
| Program | Input| Output | Mean ± Std. Dev. [s] | Min [s] | Max [s] |
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

## Testing
Tests can be run by invoking `cargo test` on the command line.

Benchmarks currently look for files in `./bench-data/` named `small.bil`, `small.bip`, `small.bsq`,
`small.bil.hdr`, `small.bip.hdr`, `small.bsq.hdr` and will delete any png, bip, bil, or bsq files
in the directory in which the benchmarks are run. Benchmarks use the `hyperfine` tool.