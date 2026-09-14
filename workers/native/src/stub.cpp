#include "celyrion/worker.h"

namespace celyrion {

WorkerKind native_worker_kind() noexcept {
#if defined(CELYRION_HAS_CUDA)
  return WorkerKind::Cuda;
#else
  return WorkerKind::Native;
#endif
}

bool has_cuda() noexcept {
#if defined(CELYRION_HAS_CUDA)
  return true;
#else
  return false;
#endif
}

}  // namespace celyrion
