"""Skills loader module - Rust implementation with Python fallback."""

try:
    from nanobot_rust import SkillsLoader
except ImportError:
    from nanobot.agent._skills_py import SkillsLoader

__all__ = ["SkillsLoader"]
