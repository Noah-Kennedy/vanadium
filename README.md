# Vanadium

Vanadium is a tool for hyperspectral image processing, similar to siproc.

Vanadium is split into two parts.
`vanadium-core` is a Rust library that provides an API to work with hyperspectral files.
`vanadium-cli` is a CLI interface which provides a nice interface for many of the operations in the core library.

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
    - [x] Linux & Mac support
    - [ ] Windows support

## CLI Tool Installation

### Rust Toolchains

You will need to first have the [Rust toolchain](https://rustup.rs/) installed. Navigate to the linked page and follow the instructions given. It is *not* recommended to use an OS package manager to
install the toolchain, unless you know what you are doing.

### Linux & Mac

```bash
git clone https://github.com/Noah-Kennedy/vanadium.git
cd vanadium
cargo install --path ../vanadium-cli
```

## Usage
For CLI documentation:
```bash
vanadium-cli --help
```

Note that vanadium-cli does not accept normal ENVI header files, but uses its own format.
You can use the tool to construct header files quite easily.

## Benchmarks

### Configurations

| Name          | RAM [GiB] | CPU (#cores)         | SSD Class | GPU         | Kernel |
|---------------|-----------|----------------------|-----------|-------------|--------|
| Workstation A | 64G       | AMD Ryzen 3900X (12) | SATA      | RTX 2080 Ti | 5.13   |
| Workstation B | 64G       | AMD Ryzen 3900X (12) | NVME      | RTX 2080 Ti | 5.13   |
| Laptop        | 16G       |                      | NVME      | NA          | 5.13   |
| HPC Node A    |           |                      | NVME      |             |        |

## Results

### Small File (5 bands, 11.75 GiB)
#### Spectral Means
| Tool                | Machine Configuration | Cache State | Time (mean ± σ)      |
|:--------------------|:----------------------|:-----------:|---------------------:|
| vanadium (syscall)  | Workstation A         | Warm        |    2.672 s ± 0.004 s |
| vanadium (io-uring) | Laptop                | N/A         |    4.067 s ± 0.025 s |
| vanadium (syscall)  | Laptop                | Cool        |   12.531 s ± 0.139 s |
| vanadium (io-uring) | Workstation A         | N/A         |   22.276 s ± 0.020 s |
| vanadium (syscall)  | Workstation A         | Cool        |   22.839 s ± 0.012 s |
| siproc              | Workstation A         | Cool        |                      |

#### Covariance Matrix
| Tool                | Machine Configuration | Cache State | Average Time [s] |
|:--------------------|:----------------------|:-----------:|-----------------:|
| vanadium (syscall)  | Workstation A         | Cool        |                  |
| vanadium (io-uring) | Workstation A         | N/A         |                  |
| siproc (cpu)        | Workstation A         | Cool        |                  |

### Medium File (394 bands, ~106 GiB)
#### Spectral Means
| Tool                | Machine Configuration | Time [s] |
|:--------------------|:----------------------|---------:|
| siproc              | Workstation A         |          |
| vanadium (syscall)  | Workstation A         |          |
| vanadium (io_uring) | Workstation A         |          |
| vanadium (io_uring) | Laptop                | s ± s |
| vanadium (syscall)  | Laptop                | 79.864 s ± 0.840 s |


#### Covariance Matrix
| Tool                | Machine Configuration | Time [s] |
|:--------------------|:----------------------|---------:|
| siproc (cuda)       | Workstation A         |          |
| siproc (cpu)        | Workstation A         |          |
| vanadium (syscall)  | Workstation A         |          |
| vanadium (io_uring) | Workstation A         |          |
| vanadium (io_uring) | Laptop                | s ± s |
| vanadium (syscall)  | Laptop                | s ± s |