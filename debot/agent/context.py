"""Context builder module - Rust implementation with Python fallback."""

try:
    from debot_rust import ContextBuilder
except ImportError:
    from debot.agent._context_py import ContextBuilder

__all__ = ["ContextBuilder"]
