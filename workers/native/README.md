# Native workers (C++ / CUDA)

C++ worker implementations and CUDA kernels for Celyrion. Not yet linked into
the Cargo build; the Rust contract is in `crates/celyrion-workers`.

```sh
cmake -S . -B build                                  # CPU only
cmake -S . -B build -DCELYRION_ENABLE_CUDA=ON        # with CUDA kernels
cmake --build build
```
