from __future__ import annotations

import io
import json
import logging
import os
import sys
import time
import traceback
from typing import Protocol

_ipc_fd = os.dup(sys.stdout.fileno())
_ipc_out = io.FileIO(_ipc_fd, mode="w")
_ipc_writer = io.TextIOWrapper(_ipc_out, encoding="utf-8", line_buffering=True)

if hasattr(sys.stdin, "reconfigure"):
    sys.stdin.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="backslashreplace")

sys.stdout = sys.stderr

logger = logging.getLogger("opf.ipc")


class Redactor(Protocol):
    id2label: dict[str, str]

    def redact(self, text: str): ...


redactor: Redactor | None = None
redactor_factory = None


def respond(data: dict):
    _ipc_writer.write(json.dumps(data, ensure_ascii=False) + "\n")
    _ipc_writer.flush()


def _extract_model_info(r: Redactor, model_path: str) -> dict:
    config = getattr(r, "config", getattr(r, "model", r))
    if hasattr(config, "config"):
        config = config.config

    name = getattr(config, "_name_or_path", model_path.rstrip("/").split("/")[-1])
    if "/" in name:
        name = name.rstrip("/").split("/")[-1]

    arch = "Unknown"
    if hasattr(config, "architectures") and config.architectures:
        arch = config.architectures[0]
    elif hasattr(config, "model_type"):
        arch = config.model_type

    return {
        "name": name,
        "architecture": arch,
        "num_labels": len(r.id2label),
        "hidden_size": getattr(config, "hidden_size", 0),
        "vocab_size": getattr(config, "vocab_size", 0),
        "max_position_embeddings": getattr(config, "max_position_embeddings", 0),
    }


def handle_load(params: dict) -> dict:
    global redactor
    model_path = params.get("model_path", "")
    if not isinstance(model_path, str):
        return {"error": "model_path must be a string"}
    if not model_path:
        return {"error": "model_path is required"}

    t0 = time.monotonic()
    redactor = redactor_factory(model_path)
    elapsed = time.monotonic() - t0

    info = _extract_model_info(redactor, model_path)
    return {"ok": True, "info": info, "load_time_ms": round(elapsed * 1000, 1)}


def handle_info(_params: dict) -> dict:
    if redactor is None:
        return {"error": "Model not loaded"}
    return _extract_model_info(redactor, "")


def handle_redact(params: dict) -> dict:
    if redactor is None:
        return {"error": "Model not loaded"}

    text = params.get("text", "")
    if not isinstance(text, str):
        return {"error": "text must be a string"}
    if not text:
        return {"error": "text is required"}

    t0 = time.perf_counter()
    result = redactor.redact(text)
    latency_ms = (time.perf_counter() - t0) * 1000.0

    return {
        "schema_version": result.schema_version,
        "text": result.text,
        "redacted_text": result.redacted_text,
        "detected_spans": [
            {
                "label": s.label,
                "start": s.start,
                "end": s.end,
                "text": s.text,
                "placeholder": s.placeholder,
            }
            for s in result.detected_spans
        ],
        "summary": result.summary,
        "latency_ms": round(latency_ms, 2),
    }


def handle_health(_params: dict) -> dict:
    return {"status": "ok", "model_loaded": redactor is not None}


HANDLERS = {
    "load": handle_load,
    "info": handle_info,
    "redact": handle_redact,
    "health": handle_health,
}


def main(factory):
    global redactor_factory
    redactor_factory = factory

    logging.basicConfig(level=logging.INFO, stream=sys.stderr)
    logger.info("OPF sidecar started, waiting for commands on stdin...")

    respond({"ready": True})

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError as e:
            respond({"error": f"Invalid JSON: {e}"})
            continue

        cmd = msg.get("cmd")
        req_id = msg.get("id")
        params = msg.get("params", {})

        handler = HANDLERS.get(cmd)
        if handler is None:
            respond({"id": req_id, "error": f"Unknown command: {cmd}"})
            continue

        try:
            result = handler(params)
            result["id"] = req_id
            respond(result)
        except Exception as e:
            logger.error("Command %s failed: %s", cmd, traceback.format_exc())
            respond({"id": req_id, "error": str(e)})
