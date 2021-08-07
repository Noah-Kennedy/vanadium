# Vanadium

[![codecov](https://codecov.io/gh/Noah-Kennedy/vanadium/branch/master/graph/badge.svg?token=2KBFXPBR34)](https://codecov.io/gh/Noah-Kennedy/vanadium)

![Continuous integration](https://github.com/Noah-Kennedy/vanadium/workflows/Continuous%20integration/badge.svg?branch=master)

`vanadium-cli` is a cli tool and library for manipulating ENVI BIP, BIL, and BSQ files for processing hyperspectral image data.

## Installation

Building `vanadium-cli` requires the installation of the rust toolchain. If that is not on your system, you will need to install it first.

### Installing up the Rust toolchains

Use the Rustup toolchain installer/manager to download and install the toolchains. Navigate to [rustup.rs](https://rustup.rs/) and follow the instructions provided.

### Building and Installing `vanadium-cli`

In your terminal of choice:

```shell script
git clone https://github.com/Noah-Kennedy/vanadium.git
cd vanadium
RUSTFLAGS="-C target-cpu=native" cargo install --path .
```

The use of `RUSTFLAGS="-C target-cpu=native"` gives better performance, but makes the binary no longer portable to different systems with a matching ABI. If you are building `vanadium-cli` on a
different system than the one on which you will use the tool (such as a compute cluster with a shared filesystem), you may need to omit that variable.

## Usage

For usage instructions, invoke `vanadium-cli --help` or `man vanadium-cli`.

## OS Support

`vanadium-cli` aims to support Windows, MacOS, and Linux.

## Issues

If you find a bug in this tool, please file a GitHub issue.