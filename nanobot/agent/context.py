"""Context builder module - Rust implementation with Python fallback."""

try:
    from nanobot_rust import ContextBuilder
except ImportError:
    from nanobot.agent._context_py import ContextBuilder

__all__ = ["ContextBuilder"]
