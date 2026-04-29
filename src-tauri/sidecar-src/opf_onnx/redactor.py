from __future__ import annotations

import json
import logging
import os
import time
from itertools import groupby
from pathlib import Path
from types import SimpleNamespace

import numpy as np
import onnxruntime as ort
from transformers import AutoTokenizer

from opf_common.config import PLACEHOLDER_MAP
from opf_common.types import DetectedSpan, RedactionResult

logger = logging.getLogger("opf-onnx.redactor")


def _find_onnx_model(model_path: str) -> tuple[Path, Path]:
    path = Path(model_path).expanduser()

    if path.is_file():
        if path.suffix.lower() != ".onnx":
            raise FileNotFoundError(f"Expected an .onnx model file, got {path}")
        return path.parent, path

    if not path.is_dir():
        raise FileNotFoundError(f"Model path not found: {path}")

    candidates = [p for p in path.rglob("*.onnx") if p.is_file()]
    if not candidates:
        raise FileNotFoundError(f"No .onnx model file found in {path}")

    candidates.sort(
        key=lambda candidate: (
            0 if candidate.parent == path else 1,
            len(candidate.relative_to(path).parts),
            str(candidate.relative_to(path)).lower(),
        )
    )
    return path, candidates[0]


def _select_providers() -> list[str]:
    available = ort.get_available_providers()
    logger.info("Available ONNX Runtime providers: %s", available)

    preference = [
        "CUDAExecutionProvider",
        "DirectMLExecutionProvider",
        "CoreMLExecutionProvider",
        "CPUExecutionProvider",
    ]

    selected = [p for p in preference if p in available]
    if not selected:
        selected = ["CPUExecutionProvider"]

    logger.info("Selected providers: %s", selected)
    return selected


class PIIRedactor:
    def __init__(self, model_path: str):
        logger.info("Loading ONNX model from %s ...", model_path)
        t0 = time.monotonic()

        model_dir, onnx_path = _find_onnx_model(model_path)
        logger.info("Using ONNX model file: %s", onnx_path)

        providers = _select_providers()
        self.session = ort.InferenceSession(str(onnx_path), providers=providers)
        self.tokenizer = AutoTokenizer.from_pretrained(str(model_dir))

        config_path = model_dir / "config.json"
        if config_path.exists():
            with open(config_path, encoding="utf-8") as f:
                config_data = json.load(f)
        else:
            config_data = {}

        self.config = SimpleNamespace(
            _name_or_path=config_data.get(
                "_name_or_path", model_dir.name
            ),
            architectures=config_data.get("architectures", []),
            model_type=config_data.get("model_type", "Unknown"),
            hidden_size=config_data.get("hidden_size", 0),
            vocab_size=config_data.get("vocab_size", 0),
            max_position_embeddings=config_data.get("max_position_embeddings", 0),
        )

        raw_id2label = config_data.get("id2label", {})
        self.id2label: dict[str, str] = {str(k): v for k, v in raw_id2label.items()}

        active = self.session.get_providers()
        logger.info(
            "ONNX model loaded in %.1fs (providers: %s)",
            time.monotonic() - t0,
            active,
        )

    def _entity_type(self, pred_id: int) -> str | None:
        label = self.id2label.get(str(pred_id), "O")
        if label == "O":
            return None
        return label.split("-", 1)[-1]

    def redact(self, text: str) -> RedactionResult:
        if not isinstance(text, str):
            text = str(text)

        inputs = self.tokenizer(text, return_tensors="np")
        input_ids = inputs["input_ids"]
        attention_mask = inputs["attention_mask"]

        ort_inputs = {
            "input_ids": input_ids.astype(np.int64),
            "attention_mask": attention_mask.astype(np.int64),
        }

        outputs = self.session.run(None, ort_inputs)
        logits = outputs[0]
        preds = np.argmax(logits, axis=-1)[0].tolist()
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
