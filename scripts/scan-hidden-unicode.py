#!/usr/bin/env python3
"""Scan the committed text files for hidden and bidirectional code points.

The hk.pkl pre-commit hook runs this scanner on the staged files. The CI typos
job runs it on all tracked files. A hit stops the commit or fails the job,
because a hidden code point can make the code that a reviewer sees differ from
the code that the compiler reads.

fallow also has an `agent` mode that scans local editor instruction files from
a session start hook. This repository has no such hook, so only the
`committed` mode stays. The flag stays the same as in fallow.

Exit code: 0 when the scan finds no hit, 1 on a hit, 2 on a usage error.
The scanner uses only the Python standard library.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Zero-width and bidirectional-override code points. They have no correct use
# in source text. A U+FEFF (byte order mark) at the start of a file is
# accepted. A U+FEFF at a different position is a hit. U+200D (zero-width
# joiner) has a separate rule, because emoji sequences use it.
BLOCK_CODEPOINTS = {
    0x200B,  # zero-width space
    0x200C,  # zero-width non-joiner
    0x2060,  # word joiner
    0x202A,  # left-to-right embedding
    0x202B,  # right-to-left embedding
    0x202C,  # pop directional formatting
    0x202D,  # left-to-right override
    0x202E,  # right-to-left override
    0x2066,  # left-to-right isolate
    0x2067,  # right-to-left isolate
    0x2068,  # first strong isolate
    0x2069,  # pop directional isolate
}
ZWJ = 0x200D
BOM = 0xFEFF

# The text file extensions that the scanner reads. fallow also reads the
# TypeScript and JSX extensions. This repository has no such files.
COMMITTED_EXTS = {
    ".md", ".rs", ".js", ".mjs", ".cjs", ".py", ".sh",
    ".json", ".yml", ".yaml", ".toml", ".pkl",
}


def is_emoji_ish(cp: int) -> bool:
    """Return True for a code point that can be next to a ZWJ in an emoji sequence."""
    return (
        0x1F000 <= cp <= 0x1FAFF  # supplemental symbols, emoji, pictographs
        or 0x2600 <= cp <= 0x27BF  # misc symbols + dingbats
        or 0x2300 <= cp <= 0x23FF  # misc technical (some emoji)
        or cp == 0xFE0F  # variation selector-16 (emoji presentation)
        or 0x1F3FB <= cp <= 0x1F3FF  # skin-tone modifiers
        or 0x1F1E6 <= cp <= 0x1F1FF  # regional indicators
        or cp in (0x2640, 0x2642, 0x2695, 0x2696, 0x2708, 0x2764)  # signs in ZWJ sequences
    )


def codepoint_hits(text: str) -> list[tuple[int, int, str]]:
    """Return (line, col, label) for each disallowed code point in `text`."""
    hits: list[tuple[int, int, str]] = []
    line = 1
    col = 1
    for i, ch in enumerate(text):
        cp = ord(ch)
        if cp == 0x0A:
            line += 1
            col = 1
            continue
        if cp in BLOCK_CODEPOINTS:
            is_bidi = 0x202A <= cp <= 0x202E or 0x2066 <= cp <= 0x2069
            hits.append((line, col, f"U+{cp:04X} {'bidi-override' if is_bidi else 'zero-width'}"))
        elif cp == BOM and i != 0:
            hits.append((line, col, "U+FEFF byte-order-mark mid-file"))
        elif cp == ZWJ:
            prev_cp = ord(text[i - 1]) if i > 0 else 0
            next_cp = ord(text[i + 1]) if i + 1 < len(text) else 0
            if not (is_emoji_ish(prev_cp) and is_emoji_ish(next_cp)):
                hits.append((line, col, "U+200D zero-width-joiner outside an emoji sequence"))
        col += 1
    return hits


def read_text(path: Path) -> str | None:
    """Return the UTF-8 text of `path`, or None for a binary or unreadable file."""
    try:
        data = path.read_bytes()
    except (OSError, ValueError):
        return None
    if b"\x00" in data[:4096]:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def git_lines(args: list[str]) -> list[Path]:
    """Run git in the repository root and return each output line as a path."""
    out = subprocess.run(
        ["git", *args], cwd=REPO_ROOT, capture_output=True, text=True, check=True
    ).stdout
    return [REPO_ROOT / line for line in out.splitlines() if line]


def committed_surface(staged_only: bool) -> list[Path]:
    """Return the tracked or staged text files that the scanner reads."""
    if staged_only:
        files = git_lines(["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
    else:
        files = git_lines(["ls-files"])
    return [p for p in files if p.suffix in COMMITTED_EXTS and p.is_file()]


def scan_committed(staged_only: bool) -> int:
    """Print each hit and return the exit code."""
    errors = 0
    for path in committed_surface(staged_only):
        text = read_text(path)
        if text is None:
            continue
        for line, col, label in codepoint_hits(text):
            errors += 1
            rel = path.relative_to(REPO_ROOT).as_posix()
            print(f"error: {rel}:{line}:{col}: hidden code point ({label})", file=sys.stderr)
    if errors:
        print(
            f"\n{errors} hidden code point(s) found. Remove them from the source.\n"
            "The scanner accepts a zero-width joiner inside an emoji sequence.",
            file=sys.stderr,
        )
    return 1 if errors else 0


def main(argv: list[str]) -> int:
    mode = None
    staged_only = False
    for arg in argv:
        if arg in ("--mode=committed", "committed"):
            mode = "committed"
        elif arg == "--mode":
            continue
        elif arg == "--staged":
            staged_only = True
    if mode == "committed":
        return scan_committed(staged_only)
    print("usage: scan-hidden-unicode.py --mode committed [--staged]", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
