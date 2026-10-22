import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "claude-say.sh"


class ClaudeSayTests(unittest.TestCase):
    def run_hook(self, payload, **env):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            transcript = root / "transcript.jsonl"
            transcript.write_text("\n".join(json.dumps(row) for row in payload.pop("messages")) + "\n", encoding="utf-8")
            payload["transcript_path"] = str(transcript)
            clean_path = root / "bin"
            clean_path.mkdir()
            (clean_path / "bash").symlink_to("/bin/bash")
            (clean_path / "python3").symlink_to("/usr/bin/python3")
            (clean_path / "cat").symlink_to("/bin/cat")
            result = root / "alacod-voice.txt"
            environment = {"ALACOD_VOICE": "1", "PATH": str(clean_path), "TMPDIR": str(root), **env}
            completed = subprocess.run([str(SCRIPT)], input=json.dumps(payload), text=True,
                                       capture_output=True, env=environment)
            self.assertEqual(completed.returncode, 0)
            return result.exists(), result.read_text(encoding="utf-8") if result.exists() else ""

    def test_reads_last_assistant_and_cleans_markdown(self):
        exists, text = self.run_hook({
            "session_id": "s",
            "messages": [
                {"type": "assistant", "message": {"content": [{"type": "text", "text": "premier"}]}},
                {"type": "assistant", "message": {"content": [{"type": "text", "text": "# Dernier *message* avec `code` et [lien](https://example.test). ```python\nprint(1)\n```"}]}},
            ],
        })
        self.assertTrue(exists)
        self.assertIn("Dernier message avec et lien", text)
        self.assertIn("bloc de code", text)
        self.assertNotIn("premier", text)

    def test_disabled_is_silent(self):
        exists, _ = self.run_hook({"messages": [{"type": "assistant", "message": {"content": [{"type": "text", "text": "secret"}]}}]}, ALACOD_VOICE="")
        self.assertFalse(exists)

    def test_stop_hook_active_is_silent(self):
        exists, _ = self.run_hook({"stop_hook_active": True, "messages": [{"type": "assistant", "message": {"content": [{"type": "text", "text": "boucle"}]}}]})
        self.assertFalse(exists)


if __name__ == "__main__":
    unittest.main()
