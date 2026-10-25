#!/usr/bin/env python3
"""Validate the exact Operator-approved Slice30 archive-doc result."""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
from html.parser import HTMLParser
from pathlib import Path
import sys
from typing import Any

SOURCE = Path("/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/public-api-documentation")
EXPECTED_HEAD = "40e6bc8f2b26606393931a232e604ce0e6f38e86"
EXPECTED_TARGET = Path("/private/tmp/nms-slice30-cc-approved-archive-af61924901ce4f8789579d2b2e354ca4")
EXPECTED_ARGV = [
    "cargo", "doc", "-p", "nms-core", "--all-features", "--no-deps",
    "--offline", "--message-format=json",
]
EXPECTED_OVERRIDES = {
    "RUSTDOCFLAGS": "-D warnings -W missing_docs",
    "CARGO_TARGET_DIR": str(EXPECTED_TARGET),
}
EXPECTED_MANIFEST = str(SOURCE / "crates/nms-core/Cargo.toml")
EXPECTED_PACKAGE_ID = f"path+file://{SOURCE}/crates/nms-core#0.3.0"
EXPECTED_DIAGNOSTIC = "missing documentation for a struct field"
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()


class IdCollector(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.ids: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        for key, value in attrs:
            if key == "id" and value is not None:
                self.ids.add(value)

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.handle_starttag(tag, attrs)


def exact_known_warning(event: dict[str, Any]) -> bool:
    """Pure predicate for the one expressly approved generated-field warning."""
    if event.get("reason") != "compiler-message":
        return False
    if event.get("manifest_path") != EXPECTED_MANIFEST:
        return False
    if event.get("package_id") != EXPECTED_PACKAGE_ID:
        return False
    target = event.get("target")
    if not isinstance(target, dict) or target.get("name") != "nms_core" or target.get("kind") != ["lib"]:
        return False
    message = event.get("message")
    if not isinstance(message, dict):
        return False
    if message.get("level") != "warning" or message.get("message") != EXPECTED_DIAGNOSTIC:
        return False
    code = message.get("code")
    if not isinstance(code, dict) or code.get("code") != "missing_docs":
        return False
    spans = message.get("spans")
    if not isinstance(spans, list):
        return False
    primary = [span for span in spans if isinstance(span, dict) and span.get("is_primary") is True]
    if len(primary) != 1:
        return False
    span = primary[0]
    if (
        span.get("file_name") != "crates/nms-core/src/system.rs"
        or span.get("line_start") != 38
        or span.get("line_end") != 38
        or span.get("column_start") != 12
        or span.get("column_end") != 25
    ):
        return False
    expansion = span.get("expansion")
    if not isinstance(expansion, dict) or expansion.get("macro_decl_name") != "#[derive(rkyv::Archive)]":
        return False
    definition = expansion.get("def_site_span")
    if not isinstance(definition, dict):
        return False
    return str(definition.get("file_name", "")).endswith("/rkyv_derive-0.8.15/src/lib.rs")


def diagnostic_list_is_exact(events: list[dict[str, Any]]) -> bool:
    """Accept only a complete diagnostic list containing the single allowed event."""
    return len(events) == 1 and exact_known_warning(events[0])


def verified_channel(channel: Any, label: str) -> bytes:
    if not isinstance(channel, dict):
        raise ValueError(f"raw {label} channel is missing")
    encoded = channel.get("base64")
    if not isinstance(encoded, str):
        raise ValueError(f"raw {label} base64 is missing")
    try:
        raw = base64.b64decode(encoded, validate=True)
    except Exception as exc:
        raise ValueError(f"raw {label} base64 is invalid: {exc}") from exc
    if channel.get("bytes") != len(raw):
        raise ValueError(f"raw {label} byte count does not match decoded bytes")
    if channel.get("sha256") != hashlib.sha256(raw).hexdigest():
        raise ValueError(f"raw {label} SHA-256 does not match decoded bytes")
    return raw


def parse_events(stdout: bytes) -> list[dict[str, Any]]:
    try:
        text = stdout.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ValueError(f"Cargo stdout is not UTF-8: {exc}") from exc
    events: list[dict[str, Any]] = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"Cargo stdout line {line_number} is not JSON: {exc}") from exc
        if not isinstance(event, dict):
            raise ValueError(f"Cargo stdout line {line_number} is not a JSON object")
        events.append(event)
    if not events:
        raise ValueError("Cargo stdout contained no JSON events")
    return events


def verify_capture(log_path: Path, record_id: str) -> tuple[list[dict[str, Any]], bytes, bytes]:
    data = json.loads(log_path.read_text(encoding="utf-8"))
    attempts = data.get("attempts") if isinstance(data, dict) else None
    if not isinstance(attempts, list):
        raise ValueError("capture log has no attempts list")
    matches = [item for item in attempts if isinstance(item, dict) and item.get("id") == record_id]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one capture record for {record_id}; found {len(matches)}")
    record = matches[0]
    if record.get("status") != "complete" or record.get("kind") != "SUT gate":
        raise ValueError("selected record is not a complete SUT gate")
    if record.get("argv") != EXPECTED_ARGV:
        raise ValueError(f"unexpected argv: {record.get('argv')!r}")
    if record.get("cwd") != str(SOURCE):
        raise ValueError(f"unexpected cwd: {record.get('cwd')!r}")
    if record.get("overrides") != EXPECTED_OVERRIDES:
        raise ValueError(f"unexpected overrides: {record.get('overrides')!r}")
    before, after = record.get("before"), record.get("after")
    if not isinstance(before, dict) or not isinstance(after, dict):
        raise ValueError("source before/after metadata is incomplete")
    if before != after:
        raise ValueError("source metadata changed during the Cargo command")
    if (
        before.get("head") != EXPECTED_HEAD
        or before.get("status") != ""
        or before.get("working_diff_sha256") != EMPTY_SHA256
        or before.get("untracked_files") != {}
        or before.get("working_identity_sha256") != after.get("working_identity_sha256")
    ):
        raise ValueError("source identity is not the expected clean committed candidate")
    completion = record.get("raw_completion")
    if not isinstance(completion, dict) or completion.get("exit_code") != 0:
        raise ValueError("actual Cargo command did not complete with exit 0")
    if not isinstance(completion.get("ended_utc"), str) or not completion["ended_utc"]:
        raise ValueError("raw completion end timestamp is missing")
    stdout = verified_channel(completion.get("stdout"), "stdout")
    stderr = verified_channel(completion.get("stderr"), "stderr")
    return parse_events(stdout), stdout, stderr


def verify_events(events: list[dict[str, Any]]) -> dict[str, Any]:
    finished = [event for event in events if event.get("reason") == "build-finished"]
    if len(finished) != 1 or finished[0].get("success") is not True:
        raise ValueError("expected exactly one successful build-finished event")
    compiler_messages = [event for event in events if event.get("reason") == "compiler-message"]
    if not diagnostic_list_is_exact(compiler_messages):
        raise ValueError("compiler-message list is not exactly the approved generated-field warning")

    duplicate_control = compiler_messages + [json.loads(json.dumps(compiler_messages[0]))]
    duplicate_rejected = not diagnostic_list_is_exact(duplicate_control)
    if not duplicate_rejected:
        raise ValueError("duplicate-diagnostic negative control was not rejected")

    wrong_origin = json.loads(json.dumps(compiler_messages))
    wrong_span = next(span for span in wrong_origin[0]["message"]["spans"] if span.get("is_primary") is True)
    wrong_span["file_name"] = "crates/nms-core/src/player.rs"
    wrong_origin_rejected = not diagnostic_list_is_exact(wrong_origin)
    if not wrong_origin_rejected:
        raise ValueError("wrong-origin negative control was not rejected")

    return {
        "event_count": len(events),
        "compiler_message_count": len(compiler_messages),
        "build_finished_count": len(finished),
        "success": finished[0]["success"],
        "positive_exact_warning": diagnostic_list_is_exact(compiler_messages),
        "duplicate_diagnostic_rejected": duplicate_rejected,
        "wrong_origin_rejected": wrong_origin_rejected,
    }


def verify_html(target: Path) -> dict[str, Any]:
    expected = target.resolve(strict=True)
    if expected != EXPECTED_TARGET:
        raise ValueError(f"unexpected target directory: {expected}")
    doc_root = expected / "doc" / "nms_core"
    paths = {
        "index": doc_root / "index.html",
        "archived_discovery_source": doc_root / "system" / "enum.ArchivedDiscoverySource.html",
        "discovery_source_resolver": doc_root / "system" / "enum.DiscoverySourceResolver.html",
    }
    page_meta: dict[str, Any] = {}
    page_bytes: dict[str, bytes] = {}
    for name, path in paths.items():
        if not path.is_file():
            raise ValueError(f"required fresh rustdoc page is missing: {path}")
        body = path.read_bytes()
        if not body:
            raise ValueError(f"required rustdoc page is empty: {path}")
        page_bytes[name] = body
        page_meta[name] = {"path": str(path), "bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()}
    try:
        html = page_bytes["discovery_source_resolver"].decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ValueError(f"resolver HTML is not UTF-8: {exc}") from exc
    parser = IdCollector()
    parser.feed(html)
    required_ids = {"variant.Community", "variant.Community.field.source"}
    missing = sorted(required_ids - parser.ids)
    if missing:
        raise ValueError(f"resolver HTML lacks required anchors: {missing}")
    page_meta["discovery_source_resolver"]["required_ids"] = sorted(required_ids)
    return page_meta


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture_log", type=Path)
    parser.add_argument("build_record_id")
    parser.add_argument("target_directory", type=Path)
    args = parser.parse_args()
    try:
        events, stdout, stderr = verify_capture(args.capture_log, args.build_record_id)
        event_result = verify_events(events)
        html_result = verify_html(args.target_directory)
    except Exception as exc:
        print(json.dumps({"result": "fail", "error": f"{type(exc).__name__}: {exc}"}), file=sys.stderr)
        return 1
    print(json.dumps({
        "result": "pass",
        "build_record_id": args.build_record_id,
        "source_head": EXPECTED_HEAD,
        "target_directory": str(EXPECTED_TARGET),
        "cargo_stdout_bytes": len(stdout),
        "cargo_stdout_sha256": hashlib.sha256(stdout).hexdigest(),
        "cargo_stderr_bytes": len(stderr),
        "cargo_stderr_sha256": hashlib.sha256(stderr).hexdigest(),
        "diagnostic_validation": event_result,
        "html": html_result,
        "exception_scope": "one exact pinned derive-generated missing_docs field only",
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
