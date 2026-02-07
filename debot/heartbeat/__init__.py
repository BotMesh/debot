"""Heartbeat service for periodic agent wake-ups."""

try:
    from debot_rust import HeartbeatService
except ImportError:
    from debot.heartbeat._service_py import HeartbeatService

__all__ = ["HeartbeatService"]
