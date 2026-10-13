"""Smoke tests for echo tool and summarize prompt."""
import pytest

from src.prompts.summarize import summarize
from src.tools.echo import echo


@pytest.mark.asyncio
async def test_echo_returns_input():
    assert await echo("hello") == "hello"


@pytest.mark.asyncio
async def test_echo_rejects_empty():
    with pytest.raises(ValueError):
        await echo("")


@pytest.mark.asyncio
async def test_echo_rejects_non_string():
    with pytest.raises(ValueError):
        await echo(None)  # type: ignore[arg-type]


def test_summarize_includes_input_text():
    out = summarize("long text here")
    assert "long text here" in out
    assert "Summarize" in out
