"""Memory system module - Rust implementation with Python fallback."""

try:
    from nanobot_rust import MemoryStore
except ImportError:
    from nanobot.agent._memory_py import MemoryStore

__all__ = ["MemoryStore"]
