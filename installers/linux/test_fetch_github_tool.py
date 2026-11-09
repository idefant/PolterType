#!/usr/bin/env python3
"""Offline regressions for release-tool download failures."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("fetch-github-tool.sh")


class FetchToolTests(unittest.TestCase):
    def run_download(self, mode, existing=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            mock = root / "curl"
            mock.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
out = pathlib.Path(args[args.index("--output") + 1])
url = args[-1]
mode = os.environ["FETCH_TEST_MODE"]
with open(os.environ["FETCH_TEST_CALLS"], "a") as log:
    log.write(url + "\\n")
if "releases/download" in url and mode != "direct":
    out.write_bytes(b"partial download")
    sys.exit(22)
if mode == "offline":
    sys.exit(22)
if "/releases/tags/" in url:
    out.write_text(json.dumps({"assets": [{"name": "tool.AppImage",
        "url": "https://api.github.com/repos/example/tool/releases/assets/123"}]}))
else:
    out.write_bytes(b"not an ELF" if mode == "invalid" else b"\\x7fELFpayload")
''')
            mock.chmod(0o755)
            dest = root / "tool"
            if existing is not None:
                dest.write_bytes(existing)
            env = dict(os.environ, PATH=f"{root}:{os.environ['PATH']}",
                       FETCH_TEST_MODE=mode, FETCH_TEST_CALLS=str(root / "calls"))
            result = subprocess.run(
                ["bash", str(SCRIPT), "example/tool", "continuous", "tool.AppImage", str(dest)],
                env=env, capture_output=True, text=True,
            )
            content = dest.read_bytes() if dest.exists() else None
            executable = dest.exists() and os.access(dest, os.X_OK)
            leftovers = list(root.glob("tool.*"))
            calls = (root / "calls").read_text().splitlines()
            return result, content, executable, leftovers, calls

    def test_direct_download(self):
        result, content, executable, leftovers, calls = self.run_download("direct")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(content, b"\x7fELFpayload")
        self.assertTrue(executable)
        self.assertEqual(leftovers, [])
        self.assertEqual(len(calls), 1)

    def test_persistent_504_uses_asset_api(self):
        result, content, executable, leftovers, calls = self.run_download("fallback")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(content, b"\x7fELFpayload")
        self.assertTrue(executable)
        self.assertEqual(leftovers, [])
        self.assertTrue(calls[-1].endswith("/releases/assets/123"))
        self.assertEqual(len(calls), 3)

    def test_invalid_download_preserves_existing_tool(self):
        result, content, _, leftovers, _ = self.run_download("invalid", b"previous tool")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(content, b"previous tool")
        self.assertEqual(leftovers, [])

    def test_total_failure_does_not_cache_partial_file(self):
        result, content, _, leftovers, _ = self.run_download("offline")
        self.assertNotEqual(result.returncode, 0)
        self.assertIsNone(content)
        self.assertEqual(leftovers, [])


if __name__ == "__main__":
    unittest.main()
