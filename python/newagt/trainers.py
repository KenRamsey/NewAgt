"""Pure-Python helpers for flattening AGT frames/bboxes into columnar lists for PyTorch.

No hard dependency on ``torch`` — outputs are plain lists and dicts suitable for
``torch.tensor(...)`` or ``pandas`` in user code. COMMENT/KEYWORD normalization stays
in Python (see ``comment_keyword_columns``); the Rust core preserves raw extension text.
"""

from __future__ import annotations

from typing import Any, Iterable, Mapping, Sequence


def flatten_frames(frames: Sequence[Mapping[str, Any]]) -> dict[str, list[Any]]:
    """Columnar view of frame-level sensor/target metadata."""
    out: dict[str, list[Any]] = {
        "frame_index": [],
        "pairing": [],
        "sensor_time": [],
        "sensor_azimuth": [],
        "sensor_elevation": [],
        "sensor_roll": [],
        "target_count": [],
    }
    for fr in frames:
        out["frame_index"].append(fr.get("index"))
        out["pairing"].append(fr.get("pairing"))
        sensor = fr.get("sensor") or {}
        out["sensor_time"].append(sensor.get("time"))
        out["sensor_azimuth"].append(sensor.get("azimuth"))
        out["sensor_elevation"].append(sensor.get("elevation"))
        out["sensor_roll"].append(sensor.get("roll"))
        target = fr.get("target") or {}
        targets = target.get("targets") or []
        out["target_count"].append(len(targets))
    return out


def flatten_bboxes(records: Sequence[Mapping[str, Any]]) -> dict[str, list[Any]]:
    """Columnar bbox records (one row per target per frame)."""
    out: dict[str, list[Any]] = {
        "frame_index": [],
        "tgt_index": [],
        "tgt_type": [],
        "pix_loc_x": [],
        "pix_loc_y": [],
        "x1": [],
        "y1": [],
        "x2": [],
        "y2": [],
        "provenance": [],
    }
    for rec in records:
        out["frame_index"].append(rec.get("frame_index"))
        out["tgt_index"].append(rec.get("tgt_index"))
        out["tgt_type"].append(rec.get("tgt_type"))
        out["pix_loc_x"].append(rec.get("pix_loc_x"))
        out["pix_loc_y"].append(rec.get("pix_loc_y"))
        bbox = rec.get("bbox")
        if bbox:
            out["x1"].append(bbox.get("x1"))
            out["y1"].append(bbox.get("y1"))
            out["x2"].append(bbox.get("x2"))
            out["y2"].append(bbox.get("y2"))
            prov = bbox.get("provenance")
            out["provenance"].append(prov)
        else:
            out["x1"].append(None)
            out["y1"].append(None)
            out["x2"].append(None)
            out["y2"].append(None)
            out["provenance"].append(None)
    return out


def comment_keyword_columns(
    comments: Iterable[str],
    keywords: Iterable[str],
    *,
    frame_hint_prefix: str = "frame",
) -> dict[str, list[Any]]:
    """Example thin helper: keep COMMENT/KEYWORD mapping in Python, not in Rust.

    Parses simple ``frame=N`` hints from keyword lines; extend for your corpus patterns.
    """
    rows: list[dict[str, Any]] = []
    for text in comments:
        rows.append({"kind": "comment", "text": text, "frame_hint": None})
    for text in keywords:
        hint = None
        lower = text.strip().lower()
        if lower.startswith(frame_hint_prefix):
            parts = lower.replace("=", " ").split()
            if len(parts) >= 2 and parts[1].isdigit():
                hint = int(parts[1])
        rows.append({"kind": "keyword", "text": text, "frame_hint": hint})
    if not rows:
        return {"kind": [], "text": [], "frame_hint": []}
    return {
        "kind": [r["kind"] for r in rows],
        "text": [r["text"] for r in rows],
        "frame_hint": [r["frame_hint"] for r in rows],
    }
