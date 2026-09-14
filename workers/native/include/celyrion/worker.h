// Native worker contract. Mirrors `Worker` in crates/celyrion-workers.
#pragma once

#include <cstdint>

namespace celyrion {

enum class WorkerKind : std::uint8_t { Python, Native, Cuda, Triton };

// Returns the kind this native library reports to the runtime.
WorkerKind native_worker_kind() noexcept;

// True when the library was built with CELYRION_ENABLE_CUDA.
bool has_cuda() noexcept;

}  // namespace celyrion
