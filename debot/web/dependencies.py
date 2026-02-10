"""FastAPI dependency injection helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from fastapi import Request

if TYPE_CHECKING:
    from debot.channels.manager import ChannelManager
    from debot.cron import CronService


def get_channel_manager(request: Request) -> ChannelManager:
    return request.app.state.channel_manager


def get_cron_service(request: Request) -> CronService:
    return request.app.state.cron_service


def get_start_time(request: Request) -> float:
    return request.app.state.start_time
