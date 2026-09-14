"""Python model adapters and Triton kernels for Celyrion.

The Rust side of this contract is ``crates/celyrion-workers``.
"""

from celyrion_workers.adapter import Adapter, CompletionEvent, PreparedVariant, Proposal

__version__ = "0.1.0"

__all__ = ["Adapter", "CompletionEvent", "PreparedVariant", "Proposal", "__version__"]
