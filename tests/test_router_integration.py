import asyncio
import json
import sys
from types import SimpleNamespace
from pathlib import Path

import pytest

from debot.agent.loop import AgentLoop
from debot.bus._events_py import InboundMessage
from debot.bus._queue_py import MessageBus
from debot.providers.base import LLMResponse, LLMProvider


class MockProvider(LLMProvider):
    def __init__(self):
        super().__init__()
        self.last_model = None

    async def chat(self, messages, tools=None, model=None, max_tokens=4096, temperature=0.7):
        self.last_model = model
        return LLMResponse(content="ok")

    def get_default_model(self) -> str:
        return "base-model"


@pytest.mark.asyncio
async def test_router_selects_model(monkeypatch, tmp_path: Path):
    # Install a fake debot_rust module with route_text
    def fake_route_text(prompt, max_tokens):
        return json.dumps({"model": "gpt-4o", "tier": "fast", "confidence": 0.9})

    fake_mod = SimpleNamespace(route_text=fake_route_text)
    monkeypatch.setitem(sys.modules, "debot_rust", fake_mod)

    bus = MessageBus()
    provider = MockProvider()
    agent = AgentLoop(bus=bus, provider=provider, workspace=tmp_path, model="base-model")

    msg = InboundMessage(channel="cli", sender_id="user", chat_id="direct", content="hello")

    resp = await agent._process_message(msg)

    assert provider.last_model == "gpt-4o"
    assert resp.content == "ok"


@pytest.mark.asyncio
async def test_router_fallback_on_error(monkeypatch, tmp_path: Path):
    # Make debot_rust unavailable
    monkeypatch.setitem(sys.modules, "debot_rust", None)

    bus = MessageBus()
    provider = MockProvider()
    agent = AgentLoop(bus=bus, provider=provider, workspace=tmp_path, model="base-model")

    msg = InboundMessage(channel="cli", sender_id="user", chat_id="direct", content="hello")

    resp = await agent._process_message(msg)

    assert provider.last_model == "base-model"
    assert resp.content == "ok"
