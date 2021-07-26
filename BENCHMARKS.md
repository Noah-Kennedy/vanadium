# Benchmarks

## Configurations
| Name          | RAM [GiB] | CPU (#cores)         | SSD Class | GPU         | Kernel |
|---------------|-----------|----------------------|-----------|-------------|--------|
| Workstation A | 64G       | AMD Ryzen 3900X (12) | SATA      | RTX 2080 Ti | 5.13   |
| Workstation B | 64G       | AMD Ryzen 3900X (12) | NVME      | RTX 2080 Ti | 5.13   |
| Laptop        | 16G       |                      | NVME      | NA          | 5.13   |
| HPC Node A    |           |                      | NVME      |             |        |

## Results

### Covariance Matrix

#### Small File
The "small" files are 11.75 GiB in size, with 5 bands.

This benchmark is intended to check for both performance with files small enough to fit in memory and performance with few bands.

Files which can fit in memory allow OS page caches to improve performance, and thus benefit backends which do not rely on direct IO.
Memory mapping in particular should pair excellently with this.

This benchmark should also heavily reward batching compute and IO operations, as there is some potential for
"economies of scale" and parallelism when operating on a large batch at once.
This also minimizes any overhead associated with system calls, and works more effectively with how disk IO tends to work.

| Tool                | Machine Configuration | Cache State | Format | Average Time [s] | Human Time       |
|:--------------------|:----------------------|:-----------:|:------:|-----------------:|-----------------:|
| vanadium (syscall)  | Workstation A         | Warm        | bip    |             9.16 |           9.16 s |
| vanadium (syscall)  | Workstation A         | Cool        | bip    |            29.49 |          29.49 s |
| vanadium (io-uring) | Workstation A         | N/A         | bip    |            44.52 |          44.52 s |
| siproc (cuda)       | Workstation A         | Cool        | bip    |          6744.68 | 1 h 52 m 24.68 s |
| siproc (cpu)        | Workstation A         | Cool        | bip    |           975.79 |     16 m 15.79 s |

#### Large File
The "large" files are 464.41 GiB in size, with 394 bands.

| Tool                | Machine Configuration | Format | Average Time [s] | Human Time       |
|:--------------------|:----------------------|:------:|-----------------:|-----------------:|
| siproc (cuda)       | Workstation A         | bip    |          4926.85 | 1 h 20 m 44.75 s |
| siproc (cpu)        | Workstation A         | bip    |          7638.12 | 2 h  7 m 18.12 s |
| vanadium (syscall)  | Workstation A         | bip    |          3127.82 |     52 m  7.82 s |
| vanadium (io_uring) | Workstation A         | bip    |          2411.57 |     40 m 11.57 s |

## Analysis