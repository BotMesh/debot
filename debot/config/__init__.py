"""Configuration module for debot."""

from debot.config.loader import get_config_path, load_config
from debot.config.schema import Config

__all__ = ["Config", "load_config", "get_config_path"]
