"""Model-adapter contract: propose -> prepare -> complete.

These dataclasses mirror ``Proposal``, ``PreparedVariant`` and
``CompletionEvent`` in ``crates/celyrion-workers``. Keep the field names in
sync; they will be serialized across the process boundary.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Literal

WorkerKind = Literal["Python", "Native", "Cuda", "Triton"]


@dataclass(frozen=True)
class PreparedVariant:
    """One concrete way of executing a batch."""

    id: str
    kind: WorkerKind
    artifact: bytes | None = None


@dataclass(frozen=True)
class Proposal:
    """How this adapter would execute work for a model."""

    adapter: str
    variants: list[PreparedVariant] = field(default_factory=list)


@dataclass(frozen=True)
class CompletionEvent:
    """Emitted when execution finishes; ``verified`` is set by the runtime."""

    variant: str
    output: bytes
    verified: bool = False


class Adapter:
    """Base class for Python model adapters. Subclass and override the three steps."""

    kind: WorkerKind = "Python"

    def propose(self, adapter: str) -> Proposal:
        raise NotImplementedError("Adapter.propose")

    def prepare(self, variant: PreparedVariant) -> PreparedVariant:
        raise NotImplementedError("Adapter.prepare")

    def complete(self, variant: PreparedVariant) -> CompletionEvent:
        raise NotImplementedError("Adapter.complete")
