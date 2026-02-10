"""FastAPI application factory."""

from __future__ import annotations

import time
from pathlib import Path
from typing import TYPE_CHECKING

from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles

from debot.web.routers import config, status

if TYPE_CHECKING:
    from debot.channels.manager import ChannelManager
    from debot.cron import CronService

STATIC_DIR = Path(__file__).parent / "static"


def create_app(
    *,
    channel_manager: ChannelManager,
    cron_service: CronService,
) -> FastAPI:
    """Create and configure the FastAPI application."""
    app = FastAPI(title="debot Gateway", docs_url="/docs")

    # Store service references on app.state for dependency injection
    app.state.channel_manager = channel_manager
    app.state.cron_service = cron_service
    app.state.start_time = time.time()

    # API routers
    app.include_router(status.router)
    app.include_router(config.router)

    # Serve static frontend (must be last so it doesn't shadow API routes)
    app.mount("/", StaticFiles(directory=str(STATIC_DIR), html=True), name="static")

    return app
