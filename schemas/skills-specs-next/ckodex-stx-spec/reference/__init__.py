"""
ckodex-stx-spec reference implementation v0.1.0.

Stdlib-only stubs that exercise the publisher and consumer contracts
in pure Python. Adapter modules for grpcio, fastapi, and graphql-core
are out of scope for v0.1; this package's job is to be the harness-
neutral, dependency-free core that conformance vectors can exercise.

  publisher.py — publisher-side state container + event emission
  consumer.py  — consumer-side query client + response validation
"""
from .publisher import (
    Publisher, build_capability_descriptor,
    SessionState, SkillStateRecord, TierTransitionRecord,
    EvidenceBundleRecord,
)
from .consumer import (
    Consumer, QueryResult,
    validate_query_response, redact_for_mode,
)

__all__ = [
    "Publisher", "build_capability_descriptor",
    "SessionState", "SkillStateRecord", "TierTransitionRecord", "EvidenceBundleRecord",
    "Consumer", "QueryResult",
    "validate_query_response", "redact_for_mode",
]

__version__ = "0.1.0"
