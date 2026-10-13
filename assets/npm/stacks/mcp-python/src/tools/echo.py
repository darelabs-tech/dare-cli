"""Echo tool implementation — testable as a plain function."""


async def echo(text: str) -> str:
    """Returns its input verbatim.

    Raises:
        ValueError: when text is empty or None.
    """
    if not isinstance(text, str) or not text:
        raise ValueError("text must be a non-empty string")
    return text
