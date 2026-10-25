#!/usr/bin/env python3
"""Render complete CLI output as a monochrome image without altering its raw text."""

from __future__ import annotations

import argparse
import math
import re
import sys
from pathlib import Path


_DOCUMENTED_SGR = re.compile(r"\x1b\[(30|37|39|40|49)m")


def presentation_text(raw: str) -> str:
    """Return monochrome image text while preserving all non-style content."""
    text = _DOCUMENTED_SGR.sub("", raw)
    if any(
        (ord(char) < 32 and char != "\n") or 127 <= ord(char) <= 159
        for char in text
    ):
        raise RuntimeError("capture contains unsupported control characters")
    return text


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render UTF-8 CLI output as a monochrome PNG image."
    )
    parser.add_argument("input", type=Path, help="UTF-8 text capture")
    parser.add_argument("output", type=Path, help="PNG output path")
    parser.add_argument(
        "--command", required=True, help="command caption shown above the capture"
    )
    parser.add_argument(
        "--font", required=True, type=Path, help="path to a TrueType or OpenType font"
    )
    return parser.parse_args()


def render(input_path: Path, output_path: Path, command: str, font_path: Path) -> None:
    try:
        from PIL import Image, ImageDraw, ImageFont
    except ImportError as exc:
        raise RuntimeError(
            "Pillow is required to render CLI captures; install it separately for this optional task."
        ) from exc

    try:
        text = input_path.read_bytes().decode("utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        raise RuntimeError(f"cannot read UTF-8 capture {input_path}: {exc}") from exc

    if any(
        (ord(char) < 32) or 127 <= ord(char) <= 159
        for char in command
    ):
        raise RuntimeError("caption contains unsupported control characters")
    text = presentation_text(text)

    try:
        font = ImageFont.truetype(str(font_path), 18)
    except (OSError, ValueError) as exc:
        raise RuntimeError(f"cannot load font {font_path}: {exc}") from exc

    lines = text.split("\n")
    padding = 24
    line_height = 26
    caption_gap = 14
    caption_box = font.getbbox(command)
    caption_height = max(1, caption_box[3] - caption_box[1])
    measured = [(command, caption_box)]
    measured.extend((line, font.getbbox(line)) for line in lines)
    left_edge = min(0, *(box[0] for _, box in measured))
    right_edge = max(
        *(box[2] for _, box in measured),
        *(font.getlength(value) for value, _ in measured),
    )
    image_width = max(1, math.ceil(right_edge - left_edge) + 2 * padding)
    image_height = (
        2 * padding + caption_height + caption_gap + line_height * len(lines)
    )

    image = Image.new("RGB", (image_width, image_height), color="#111827")
    draw = ImageDraw.Draw(image)
    foreground = "#f3f4f6"
    draw.text(
        (padding - left_edge, padding - caption_box[1]),
        command,
        font=font,
        fill=foreground,
    )

    text_top = padding + caption_height + caption_gap
    for index, (line, line_box) in enumerate(measured[1:]):
        draw.text(
            (
                padding - left_edge,
                text_top + index * line_height - line_box[1],
            ),
            line,
            font=font,
            fill=foreground,
        )

    try:
        image.save(output_path, format="PNG")
    except OSError as exc:
        raise RuntimeError(f"cannot write PNG {output_path}: {exc}") from exc


def main() -> int:
    args = parse_args()
    try:
        render(args.input, args.output, args.command, args.font)
    except RuntimeError as exc:
        print(f"render-cli-capture: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
