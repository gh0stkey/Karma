from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class DetectedSpan:
    label: str
    start: int
    end: int
    text: str
    placeholder: str


@dataclass
class RedactionResult:
    schema_version: int
    text: str
    redacted_text: str
    detected_spans: list[DetectedSpan] = field(default_factory=list)
    summary: dict[str, int] = field(default_factory=dict)
