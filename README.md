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

### Results

#### Small File (5 bands, 11.75 GiB)
##### Spectral Means
| Tool                | Machine Configuration | Time (mean ± σ)      |
|:--------------------|:----------------------|---------------------:|
| vanadium (syscall)  | Laptop                |   12.531 s ± 0.139 s |
| vanadium (io-uring) | Workstation A         |   22.276 s ± 0.020 s |
| vanadium (syscall)  | Workstation A         |   22.839 s ± 0.012 s |
| siproc              | Workstation A         |  484.511 s ± 2.917 s |
| siproc              | Laptop                |  590.850 s ± 2.809 s |

##### Covariance Matrix
| Tool                | Machine Configuration | Time (mean ± σ) |
|:--------------------|:----------------------|-----------------:|
| vanadium (syscall)  | Workstation A         | s ± s |
| vanadium (io-uring) | Workstation A         | 22.308 s ± 0.009 s |
| siproc (cpu)        | Workstation A         | s ± s |
| vanadium (io_uring) | Laptop                | s ± s |
| vanadium (syscall)  | Laptop                | s ± s |
| siproc              | Laptop                | s ± s |

#### Medium File (394 bands, ~106 GiB)
##### Spectral Means
| Tool                | Machine Configuration | Time (mean ± σ)     |
|:--------------------|:----------------------|--------------------:|
| vanadium (io_uring) | Laptop                |  39.123 s ± 3.695 s |
| vanadium (syscall)  | Laptop                |  79.864 s ± 0.840 s |
| siproc              | Laptop                | 199.767 s ± 1.026 s |
| vanadium (io_uring) | Workstation A         | 202.313 s ± 0.504 s |
| vanadium (syscall)  | Workstation A         | 210.469 s ± 0.301 s |
| siproc              | Workstation A         | s ± s |


##### Covariance Matrix
| Tool                | Machine Configuration | Time (mean ± σ) |
|:--------------------|:----------------------|----------------:|
| siproc (cuda)       | Workstation A         | s ± s |
| siproc (cpu)        | Workstation A         | s ± s |
| vanadium (syscall)  | Workstation A         | s ± s |
| vanadium (io_uring) | Workstation A         | s ± s |
| vanadium (io_uring) | Laptop                | s ± s |
| vanadium (syscall)  | Laptop                | s ± s |