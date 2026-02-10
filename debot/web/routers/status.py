"""Status API endpoints."""

from __future__ import annotations

import inspect
import time

from fastapi import APIRouter, Depends

from debot import __version__
from debot.channels.manager import ChannelManager
from debot.cron import CronService
from debot.web.dependencies import get_channel_manager, get_cron_service, get_start_time

router = APIRouter(prefix="/api/status", tags=["status"])


async def _maybe_await(result):
    """Await the result if it's a coroutine/future (Rust CronService), else return as-is."""
    if inspect.isawaitable(result):
        return await result
    return result


@router.get("")
async def overall_status(
    channel_manager: ChannelManager = Depends(get_channel_manager),
    cron_service: CronService = Depends(get_cron_service),
    start_time: float = Depends(get_start_time),
):
    uptime_s = time.time() - start_time
    cron = await _maybe_await(cron_service.status())
    return {
        "version": __version__,
        "uptime_s": round(uptime_s, 1),
        "channels": channel_manager.get_status(),
        "cron": cron,
    }


@router.get("/cron")
async def cron_status(
    cron_service: CronService = Depends(get_cron_service),
):
    status = await _maybe_await(cron_service.status())
    jobs = await _maybe_await(cron_service.list_jobs(include_disabled=True))
    return {
        "status": status,
        "jobs": [
            {
                "id": j.id,
                "name": j.name,
                "enabled": j.enabled,
                "schedule": {
                    "kind": j.schedule.kind,
                    "every_ms": j.schedule.every_ms,
                    "expr": j.schedule.expr,
                    "at_ms": j.schedule.at_ms,
                },
                "last_status": j.state.last_status,
                "last_run_at_ms": j.state.last_run_at_ms,
                "next_run_at_ms": j.state.next_run_at_ms,
            }
            for j in jobs
        ],
    }
