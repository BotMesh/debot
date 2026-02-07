"""Session management module - Rust implementation with Python fallback."""

try:
    from debot_rust import Session, SessionManager
except ImportError:
    from debot.session._manager_py import Session, SessionManager

__all__ = ["SessionManager", "Session"]
