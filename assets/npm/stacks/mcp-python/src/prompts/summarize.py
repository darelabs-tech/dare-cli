"""Summarize prompt — returns a templated user message."""


def summarize(text: str) -> str:
    """Returns the user message body for a summarize prompt.

    Args:
        text: Text to summarize.
    """
    return f"Summarize the following text in 1-2 sentences.\n\n{text}"
