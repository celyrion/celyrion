"""Placeholder for Triton kernels.

Real kernels will be decorated with ``@triton.jit``; Triton is not a
dependency yet, so this module must stay importable without it.
"""


def placeholder_kernel() -> None:
    """Stands in for the first Triton kernel; does nothing."""
    raise NotImplementedError("Triton kernels are not implemented yet")
