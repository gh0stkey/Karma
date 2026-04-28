from __future__ import annotations

import logging
import time
from itertools import groupby

import mlx.core as mx
from mlx_embeddings.utils import load

from opf_common.config import PLACEHOLDER_MAP
from opf_common.types import DetectedSpan, RedactionResult

logger = logging.getLogger("opf-mlx.redactor")


class PIIRedactor:
    def __init__(self, model_path: str):
        logger.info("Loading MLX model from %s ...", model_path)
        t0 = time.monotonic()
        self.model, self.tokenizer = load(model_path)
        self.config = self.model.config
        self.id2label: dict[str, str] = self.model.config.id2label
        logger.info("MLX model loaded in %.1fs", time.monotonic() - t0)

    def _entity_type(self, pred_id: int) -> str | None:
        label = self.id2label.get(str(pred_id), "O")
        if label == "O":
            return None
        return label.split("-", 1)[-1]

    def redact(self, text: str) -> RedactionResult:
        inputs = self.tokenizer(text, return_tensors="mlx")
        input_ids = inputs["input_ids"]

        outputs = self.model(input_ids, attention_mask=inputs["attention_mask"])
        preds = mx.argmax(outputs.logits, axis=-1)[0].tolist()
        token_ids = input_ids[0].tolist()

        entity_spans: list[tuple[str, str]] = []
        for ent, grp in groupby(
            zip(token_ids, preds), key=lambda x: self._entity_type(x[1])
        ):
            if ent is not None:
                decoded = self.tokenizer.decode([tid for tid, _ in grp]).strip()
                entity_spans.append((ent, decoded))
            else:
                for _ in grp:
                    pass

        spans: list[DetectedSpan] = []
        summary: dict[str, int] = {}
        search_from = 0

        for ent, span_text in entity_spans:
            idx = text.find(span_text, search_from)
            if idx == -1:
                idx = text.find(span_text)
            if idx == -1:
                continue
            placeholder = PLACEHOLDER_MAP.get(ent, f"[{ent.upper()}]")
            spans.append(
                DetectedSpan(
                    label=ent,
                    start=idx,
                    end=idx + len(span_text),
                    text=span_text,
                    placeholder=placeholder,
                )
            )
            summary[ent] = summary.get(ent, 0) + 1
            search_from = idx + len(span_text)

        parts: list[str] = []
        prev_end = 0
        for s in spans:
            parts.append(text[prev_end : s.start])
            parts.append(s.placeholder)
            prev_end = s.end
        parts.append(text[prev_end:])

        return RedactionResult(
            schema_version=1,
            text=text,
            redacted_text="".join(parts),
            detected_spans=spans,
            summary=summary,
        )
