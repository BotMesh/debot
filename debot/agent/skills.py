"""Skills loader module - Rust implementation with Python fallback."""

try:
    from debot_rust import SkillsLoader
except ImportError:
    from debot.agent._skills_py import SkillsLoader

__all__ = ["SkillsLoader"]
