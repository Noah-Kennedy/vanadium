# Hyperspectral
TODO overview

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
cargo install --path .
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

#### Colorization
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

## Design
TODO

## Benchmarks
Benchmarks were performed on a machine running Arch Linux with a Ryzen 3900X, an Nvidia RTX 2080 TI,
64 GiB of Ram, and SSD storage.

Input files had 5 bands and were 28740 x 21954.

Warm cache benchmarks occurred after the program was run twice before measured runs began, in order
to ensure that the file cache had been prepared.
Cold cache benchmarks flushed the file cache before each run, ensuring that no files were present
in the cache when the run began.

### Conversion
#### Warm Cache
|Input|Output|Time (mean ± σ)|
|-----|------|----|
|bip|bil|24.469 s ± 0.815 s|
|bip|bsq|9.458 s ± 0.429 s|
|bil|bip|24.576 s ± 1.149 s|
|bil|bsq|11.683 s ± 0.246 s|
|bsq|bip|24.582 s ± 0.431 s|
|bsq|bil|24.679 s ± 0.811 s|

#### Cold Cache
|Input|Output|Time (mean ± σ)|
|-----|------|----|
|bip|bil||
|bip|bsq||
|bil|bip||
|bil|bsq|58.449 s ±  0.254 s|
|bsq|bip|56.836 s ± 0.577 s|
|bsq|bil|58.137 s ± 0.660 s|

### Colorization
#### Warm Cache
|Input|Color Map|Time (mean ± σ)   |
|-----|---------|------------------|
|bip  |RGB      |35.433 s ± 0.748 s|
|bil  |RGB      |34.939 s ± 0.602 s|
|bsq  |RGB      |33.863 s ± 0.348 s|
|bip  |Coolwarm |23.700 s ± 0.916 s|
|bil  |Coolwarm |22.795 s ± 0.274 s|
|bsq  |Coolwarm |22.860 s ± 0.861 s|
|bip  |Gray     |14.600 s ± 0.198 s|
|bil  |Gray     |14.131 s ± 0.083 s|
|bsq  |Gray     |13.763 s ± 0.133 s|

#### Cold Cache
|Input|Color Map|Time (mean ± σ)   |
|-----|---------|------------------|
|bip  |RGB      |56.159 s ± 1.152 s|
|bil  |RGB      |55.467 s ± 0.470 s|
|bsq  |RGB      |54.522 s ± 0.392 s|
|bip  |Coolwarm |43.794 s ± 0.388 s|
|bil  |Coolwarm |43.282 s ± 0.440 s|
|bsq  |Coolwarm |43.353 s ± 0.341 s|
|bip  |Gray     |35.436 s ± 0.274 s|
|bil  |Gray     |34.771 s ± 0.280 s|
|bsq  |Gray     |34.259 s ± 0.161 s|

## Testing
TODO