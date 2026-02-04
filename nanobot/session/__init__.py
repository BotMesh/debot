"""Session management module - Rust implementation with Python fallback."""

try:
    from nanobot_rust import Session, SessionManager
except ImportError:
    from nanobot.session._manager_py import Session, SessionManager

__all__ = ["SessionManager", "Session"]
