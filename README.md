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
Coming soon!