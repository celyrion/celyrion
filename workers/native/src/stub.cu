// Placeholder CUDA kernel; only compiled when CELYRION_ENABLE_CUDA=ON.
#include <cuda_runtime.h>

namespace celyrion {

__global__ void placeholder_kernel() {}

void launch_placeholder_kernel() { placeholder_kernel<<<1, 1>>>(); }

}  // namespace celyrion
