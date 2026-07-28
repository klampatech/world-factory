"""Shared invariant types used across the validator modules.

Lifted out of `validation.py` so per-layer validators (in
`world_factory.geology`, `world_factory.hydrology`,
`world_factory.atmosphere`, `world_factory.astronomy`) can import the
types without creating a circular dependency on `validation.py`,
which itself imports the per-layer validators to orchestrate.
"""

from pydantic import Field

from world_factory.models import StrictModel


class InvariantViolation(StrictModel):
    """A machine-readable cross-layer plausibility failure."""

    code: str
    path: str
    message: str


class ValidationReport(StrictModel):
    """Complete result of evaluating the cross-layer invariant set."""

    is_valid: bool
    violations: tuple[InvariantViolation, ...] = Field(default_factory=tuple)


def violation(code: str, path: str, message: str) -> InvariantViolation:
    """Construct an InvariantViolation in one line."""
    return InvariantViolation(code=code, path=path, message=message)