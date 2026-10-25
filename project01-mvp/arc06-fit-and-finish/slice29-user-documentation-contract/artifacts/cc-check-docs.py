from __future__ import annotations

import base64
import hashlib
import importlib.util
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CAPTURE_RECORD = Path("/private/tmp/nms-slice29-cc-attempts.json")
HELP_ID = "eeae09e86c65466eadf4774ad6e4e087"
STATS_ID = "e974a83b290146298e30fce60ada9b08"
EXPECTED_METRICS = {
    "Systems": 6,
    "Planets": 13,
    "Bases": 2,
    "Named Systems": 0,
    "Named Planets": 0,
    "Infested": 1,
}
METRIC_LINE = re.compile(
    r"^\s*(Systems|Planets|Bases|Named Systems|Named Planets|Infested)\s+(\d+)\s*$"
)


def renderer_module():
    source = ROOT / "docs/render-cli-capture.py"
    spec = importlib.util.spec_from_file_location("render_cli_capture", source)
    if spec is None or spec.loader is None:
        raise AssertionError(f"cannot load renderer module: {source}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def capture_rows():
    data = json.loads(CAPTURE_RECORD.read_text())
    return data if isinstance(data, list) else data["attempts"]


def raw_stdout(rows, capture_id):
    row = next(row for row in rows if row["id"] == capture_id)
    completion = row["raw_completion"]
    assert row["status"] == "complete" and completion["exit_code"] == 0
    return base64.b64decode(completion["stdout"]["base64"], validate=True)


def metrics_match(text):
    start = text.find("GALAXY STATISTICS")
    end = text.find("BIOME DISTRIBUTION", start + 1)
    if start < 0 or end < 0:
        return False
    found = {}
    for line in text[start:end].split("\n"):
        match = METRIC_LINE.fullmatch(line)
        if match:
            name, value = match.groups()
            if name in found:
                return False
            found[name] = int(value)
        elif line.strip() and line.strip() != "GALAXY STATISTICS" and line.strip() != "Metric Count":
            if re.search(r"\d+\s*$", line):
                return False
    return found == EXPECTED_METRICS


def route_record(rows, *, comma_biome):
    for row in rows:
        argv = row.get("argv", [])
        try:
            route_at = argv.index("route")
            biome_at = argv.index("--biome", route_at + 1)
        except ValueError:
            continue
        biome = argv[biome_at + 1] if biome_at + 1 < len(argv) else ""
        try:
            save_at = argv.index("--save")
        except ValueError:
            continue
        save_path = argv[save_at + 1] if save_at + 1 < len(argv) else ""
        if save_path != "data/test/save56_multi_system.json":
            continue
        if ("," in biome) == comma_biome:
            if comma_biome or (biome == "Lush" and "--round-trip" in argv):
                return row
    raise AssertionError(f"missing captured route with comma_biome={comma_biome}")


def check_local_markdown_links(paths):
    link_pattern = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
    for markdown in paths:
        for target in link_pattern.findall(markdown.read_text()):
            target = target.strip().split(maxsplit=1)[0].strip("<>")
            if not target or re.match(r"(?:[a-z]+:)?//", target) or target.startswith("mailto:"):
                continue
            local = target.split("#", 1)[0]
            if local:
                assert (markdown.parent / local).exists(), f"broken local link: {markdown}: {target}"


def check_bash_syntax(paths):
    code_pattern = re.compile(r"```bash\s*\n(.*?)\n```", re.DOTALL)
    for markdown in paths:
        for number, code in enumerate(code_pattern.findall(markdown.read_text()), 1):
            result = subprocess.run(
                ["bash", "-n"], input=code, text=True, capture_output=True
            )
            assert result.returncode == 0, (
                f"bash syntax error in {markdown} block {number}: {result.stderr}"
            )


def main():
    renderer = renderer_module()
    rows = capture_rows()
    help_raw = raw_stdout(rows, HELP_ID)
    stats_raw = raw_stdout(rows, STATS_ID)
    help_path = ROOT / "docs/captures/nms-help.txt"
    stats_path = ROOT / "docs/captures/nms-stats.txt"
    assert help_path.read_bytes() == help_raw
    assert stats_path.read_bytes() == stats_raw
    assert hashlib.sha256(help_raw).hexdigest() == "2bba2f1030814740c5063af89546e4b214b0cbfd77332829cb7215da737f00bb"
    assert hashlib.sha256(stats_raw).hexdigest() == "d6ae97e45f6007dffd6a467bc074b86717d2502cb3fb7e9b9f5beca01519db53"

    help_text = help_raw.decode("utf-8")
    stats_text = stats_raw.decode("utf-8")
    assert renderer.presentation_text(help_text) == help_text
    stats_presentation = renderer.presentation_text(stats_text)
    assert len(stats_presentation.encode("utf-8")) == 589
    assert stats_presentation.count("\n") == 25
    assert hashlib.sha256(stats_presentation.encode("utf-8")).hexdigest() == "aef4bcdb07328f749142888d8f0b3774812d33d6d4d077262c1d815dd76bc0ad"
    assert renderer.presentation_text("A\x1b[37mB\x1b[39m\n") == "AB\n"
    for invalid in ("A\x1b[2JB", "A\x1b[31mB", "A\rB"):
        try:
            renderer.presentation_text(invalid)
        except RuntimeError:
            pass
        else:
            raise AssertionError(f"unsupported control was accepted: {invalid!r}")

    assert metrics_match(stats_presentation)
    mutant = re.sub(r"(?m)^(\s*Systems\s+)6(\s*)$", r"\g<1>7\g<2>", stats_presentation, count=1)
    assert mutant != stats_presentation and not metrics_match(mutant)

    invalid_route = route_record(rows, comma_biome=True)
    invalid_raw = b"".join(
        base64.b64decode(invalid_route["raw_completion"][name]["base64"], validate=True)
        for name in ("stdout", "stderr")
    )
    assert invalid_route["raw_completion"]["exit_code"] != 0
    assert b"Invalid biome" in invalid_raw
    valid_route = route_record(rows, comma_biome=False)
    assert valid_route["raw_completion"]["exit_code"] == 0

    check_local_markdown_links(
        [ROOT / "README.md", ROOT / "CONTRIBUTING.md", ROOT / "docs/captures/README.md"]
    )
    check_bash_syntax([ROOT / "README.md", ROOT / "CONTRIBUTING.md"])
    from PIL import Image

    for image_path in (ROOT / "docs/captures/nms-help.png", ROOT / "docs/captures/nms-stats.png"):
        with Image.open(image_path) as image:
            image.verify()

    print("documentation checks passed")


if __name__ == "__main__":
    main()
