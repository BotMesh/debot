"""Configuration API endpoints."""

from __future__ import annotations

import re
from typing import Any

from fastapi import APIRouter, HTTPException

from debot.config.loader import load_config, save_config

router = APIRouter(prefix="/api/config", tags=["config"])

VALID_SECTIONS = {"providers", "channels", "agents", "tools", "gateway"}

# Pattern to detect potential API keys / secrets
_SECRET_FIELDS = re.compile(r"(api_key|token|secret|password)", re.IGNORECASE)


def _mask_secrets(data: Any) -> Any:
    """Recursively mask values whose keys look like secrets."""
    if isinstance(data, dict):
        out = {}
        for k, v in data.items():
            if _SECRET_FIELDS.search(k) and isinstance(v, str) and len(v) > 4:
                out[k] = v[:4] + "*" * (len(v) - 4)
            else:
                out[k] = _mask_secrets(v)
        return out
    if isinstance(data, list):
        return [_mask_secrets(item) for item in data]
    return data


def _is_masked(value: str) -> bool:
    """Check if a value looks like a masked secret (contains ****)."""
    return isinstance(value, str) and "****" in value


def _strip_masked(update: Any, original: Any) -> Any:
    """Recursively remove masked secret values from update, keeping originals."""
    if isinstance(update, dict) and isinstance(original, dict):
        out = {}
        for k, v in update.items():
            orig_v = original.get(k)
            if _SECRET_FIELDS.search(k) and _is_masked(v):
                # Keep original value — user didn't change the masked field
                if orig_v is not None:
                    out[k] = orig_v
                # else: skip entirely, don't set a masked placeholder
            else:
                out[k] = _strip_masked(v, orig_v) if orig_v is not None else v
        return out
    return update


@router.get("")
async def get_config():
    config = load_config()
    return _mask_secrets(config.model_dump())


@router.put("")
async def update_config(body: dict[str, Any]):
    config = load_config()
    original = config.model_dump()
    for section in VALID_SECTIONS:
        if section in body:
            cleaned = _strip_masked(body[section], original.get(section, {}))
            current = getattr(config, section)
            merged = type(current).model_validate({**current.model_dump(), **cleaned})
            setattr(config, section, merged)
    save_config(config)
    return _mask_secrets(config.model_dump())


@router.get("/{section}")
async def get_section(section: str):
    if section not in VALID_SECTIONS:
        raise HTTPException(status_code=404, detail=f"Unknown section: {section}")
    config = load_config()
    data = getattr(config, section).model_dump()
    return _mask_secrets(data)


@router.put("/{section}")
async def update_section(section: str, body: dict[str, Any]):
    if section not in VALID_SECTIONS:
        raise HTTPException(status_code=404, detail=f"Unknown section: {section}")
    config = load_config()
    current = getattr(config, section)
    original = current.model_dump()
    cleaned = _strip_masked(body, original)
    merged = type(current).model_validate({**original, **cleaned})
    setattr(config, section, merged)
    save_config(config)
    return _mask_secrets(merged.model_dump())
