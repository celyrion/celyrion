# Workers

Model workers are the pieces of Celyrion that actually execute a model. They
are deliberately kept outside the Cargo workspace because they are written in
whatever language the hardware demands. The Rust side of the contract lives in
[`crates/celyrion-workers`](../crates/celyrion-workers): a worker **proposes**
execution variants, the runtime **prepares** one, and the worker emits a
**completion event** that the runtime verifies.

| Directory | `WorkerKind` | Contents |
|-----------|--------------|----------|
| [`python/`](python/) | `Python`, `Triton` | Python model adapters and Triton kernels (`celyrion_workers` package, managed with `uv`). |
| [`native/`](native/) | `Native`, `Cuda` | C++ workers and CUDA kernels (CMake project, CUDA optional). |

Nothing in this directory is built or linked by `cargo build` yet. Each
subdirectory has its own build instructions.
