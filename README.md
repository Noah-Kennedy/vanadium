# Vanadium

Vanadium is a tool for hyperspectral image processing, similar to siproc.

## Roadmap

- Features
    - [x] Basic statistical operations
    - [x] Principal Component Analysis
    - [x] 32-bit float support
    - [x] Image cropping
    - [x] BIP support
    - [ ] BSQ support
    - [ ] BIL support
    - [ ] Image conversion
    - [ ] Image rendering
    - [ ] Image masking
    - [ ] Minimum Noise Fraction
    - [ ] 64-bit float support
    - [ ] Python wrapper
- Performance
    - [x] Better performance than siproc
    - [x] io_uring support
    - [x] BLAS acceleration
    - [ ] CUDA acceleration
- Platform Support
    - [x] Linux support
    - [x] MacOS support
    - [x] Windows support

## CLI Tool Installation

### Rust Toolchains

You will need to first have the [Rust toolchain](https://rustup.rs/) installed. Navigate to the linked page and follow the instructions given. It is *not* recommended to use an OS package manager to
install the toolchain, unless you know what you are doing.

### BLAS

You will need to select a blas implementation. Currently supported are:

* `openblas`
* `netlib`

## Linux CLI Install

```bash
git clone https://github.com/Noah-Kennedy/vanadium.git
cd vanadium
cargo install --path . --features="linux, {YOUR_BLAS_HERE}"
```

## MacOS CLI Install

```bash
git clone https://github.com/Noah-Kennedy/vanadium.git
cd vanadium
cargo install --path . --features="macos, {YOUR_BLAS_HERE}"
```

## Windows CLI Install

```bash
git clone https://github.com/Noah-Kennedy/vanadium.git
cd vanadium
cargo install --path . --features="windows, {YOUR_BLAS_HERE}"
```

## Usage

For CLI documentation:

```bash
vanadium-cli --help
```

Note that vanadium-cli does not accept normal ENVI header files, but uses its own format. You can use the tool to construct header files quite easily.

## Benchmarks

Benchmarks were all run with a cold file cache between each trial, with 1 warmup trial. Benchmarks were run using hyperfine.

### Configurations

| Name          | RAM [GiB] | CPU (#cores)         | SSD Class | GPU         | Kernel |
|---------------|-----------|----------------------|-----------|-------------|--------|
| Workstation | 64G       | AMD Ryzen 3900X (12) | SATA      | RTX 2080 Ti | 5.13   |
| Laptop      | 16G       |                      | NVME      | NA          | 5.13   |
| Nvidia DJX  | 16G       |                      | NVME      | TODO        | 4.x    |

### Commands

| Tool                | Operation | File  | Command |
|---------------------|-----------|-------|---------|
| vanadium (io-uring) | means     | small | hyperfine "vanadium-cli --backend glommio means -o small-means.json --header small-header.json" -p "sudo sh scripts/drop.sh" |
| vanadium (syscall)  | means     | small | hyperfine "vanadium-cli --backend syscall means -o small-means.json --header small-header.json" -p "sudo sh scripts/drop.sh" |
| vanadium (mmap)     | means     | small | hyperfine "vanadium-cli --backend mmap means -o small-means.json --header small-header.json" -p "sudo sh scripts/drop.sh" |
| vanadium (io-uring) | means     | large | hyperfine "vanadium-cli --backend glommio means -o large-means.json --header large-header.json" -p "sudo sh scripts/drop.sh" |
| vanadium (syscall)  | means     | large | hyperfine "vanadium-cli --backend syscall means -o large-means.json --header large-header.json" -p "sudo sh scripts/drop.sh" |
| vanadium (mmap)     | means     | large | hyperfine "vanadium-cli --backend mmap means -o large-means.json --header large-header.json" -p "sudo sh scripts/drop.sh" |
| siproc | means     | large | |

### Results

#### Small File (5 bands, 11.75 GiB)

##### Spectral Means

| Tool                | Machine Configuration | Time (mean ± σ)      |
|:--------------------|:----------------------|---------------------:|
| vanadium (io-uring) | Laptop                |    4.511 s ± 0.219 s |
| vanadium (syscall)  | Laptop                |   12.531 s ± 0.139 s |
| vanadium (mmap)     | Laptop                |   13.486 s ± 0.119 s |
| vanadium (io-uring) | Workstation           |   22.276 s ± 0.020 s |
| vanadium (syscall)  | Workstation           |   22.839 s ± 0.012 s |
| vanadium (mmap)     | Workstation           |   24.768 s ± 0.057 s |
| siproc              | Workstation           |  484.511 s ± 2.917 s |
| siproc              | Laptop                |  590.850 s ± 2.809 s |

##### Covariance Matrix

| Tool                | Machine Configuration | Time (mean ± σ)     |
|:--------------------|:----------------------|--------------------:|
| vanadium (io_uring) | Laptop                |   8.718 s ± 0.019 s |
| vanadium (syscall)  | Laptop                |  14.171 s ± 0.050 s |
| vanadium (mmap)     | Laptop                |  14.457 s ± 0.063 s |
| vanadium (io-uring) | Workstation           |  22.308 s ± 0.009 s |
| vanadium (syscall)  | Workstation           |  22.840 s ± 0.003 s |
| vanadium (mmap)     | Workstation           |  24.859 s ± 0.049 s |
| siproc              | Laptop                | 669.587 s ± 4.004 s |
| siproc              | Workstation           | s ± s |

#### Medium File (394 bands, ~106 GiB)

##### Spectral Means

| Tool                | Machine Configuration | Time (mean ± σ)     |
|:--------------------|:----------------------|--------------------:|
| vanadium (io_uring) | Laptop                |  39.123 s ± 3.695 s |
| vanadium (syscall)  | Laptop                |  79.864 s ± 0.840 s |
| vanadium (mmap)     | Laptop                | 176.015 s ± 5.354 s |
| siproc              | Laptop                | 199.767 s ± 1.026 s |
| vanadium (io_uring) | Workstation           | 202.313 s ± 0.504 s |
| vanadium (syscall)  | Workstation           | 210.469 s ± 0.301 s |
| siproc              | Workstation           | 210.635 s ± 0.627 s |
| vanadium (mmap)     | Workstation           | 232.937 s ± 0.458 s |

##### Covariance Matrix

| Tool                | Machine Configuration | Time (mean ± σ) |
|:--------------------|:----------------------|--------------------:|
| vanadium (io_uring) | Workstation           |   352.336 s ± 0.103 s |
| vanadium (io_uring) | Laptop                |   503.445 s ± 2.007 s |
| vanadium (syscall)  | Workstation           |   513.370 s ± 1.796 s |
| vanadium (mmap)     | Workstation           |   536.006 s ± 2.626 s |
| siproc (cuda)       | Workstation           | ~1541.157 s           |
| siproc (cpu)        | Laptop                | ~7079.293 s           |
| vanadium (syscall)  | Laptop                | s ± s |
| vanadium (mmap)     | Laptop                | s ± s |
| siproc (cpu)        | Workstation           | s ± s |