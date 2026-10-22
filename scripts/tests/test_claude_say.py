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

    def run_with_fake_say(self, voice_name=None, listed="Thomas"):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            transcript = root / "transcript.jsonl"
            transcript.write_text(json.dumps({"type": "assistant", "message": {"content": [{"type": "text", "text": "bonjour"}]}}) + "\n", encoding="utf-8")
            clean_path = root / "bin"
            clean_path.mkdir()
            for name, target in (("bash", "/bin/bash"), ("python3", "/usr/bin/python3"),
                                 ("cat", "/bin/cat"), ("awk", "/usr/bin/awk"), ("grep", "/usr/bin/grep")):
                (clean_path / name).symlink_to(target)
            args_file = root / "say-args"
            fake_say = clean_path / "say"
            fake_say.write_text(
                "#!/bin/sh\n"
                "if [ \"$1\" = '-v' ] && [ \"$2\" = '?' ]; then printf '%s\\n' \"$FAKE_SAY_VOICES\"; exit 0; fi\n"
                "printf '%s\\n' \"$*\" > \"$FAKE_SAY_ARGS\"\n", encoding="utf-8")
            fake_say.chmod(0o755)
            environment = {"ALACOD_VOICE": "1", "PATH": str(clean_path), "TMPDIR": str(root),
                           "FAKE_SAY_VOICES": listed, "FAKE_SAY_ARGS": str(args_file)}
            if voice_name is not None:
                environment["ALACOD_VOICE_NAME"] = voice_name
            payload = {"transcript_path": str(transcript)}
            completed = subprocess.run([str(SCRIPT)], input=json.dumps(payload), text=True,
                                       capture_output=True, env=environment)
            self.assertEqual(completed.returncode, 0)
            for _ in range(20):
                if args_file.exists():
                    break
                __import__("time").sleep(0.01)
            return args_file.read_text(encoding="utf-8") if args_file.exists() else ""

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
    def test_say_uses_system_voice_when_name_absent(self):
        self.assertNotIn("-v", self.run_with_fake_say())

    def test_say_ignores_unlisted_voice(self):
        self.assertNotIn("-v", self.run_with_fake_say("Inexistante", listed="Thomas Amélie"))


    def test_disabled_is_silent(self):
        exists, _ = self.run_hook({"messages": [{"type": "assistant", "message": {"content": [{"type": "text", "text": "secret"}]}}]}, ALACOD_VOICE="")
        self.assertFalse(exists)

    def test_stop_hook_active_is_silent(self):
        exists, _ = self.run_hook({"stop_hook_active": True, "messages": [{"type": "assistant", "message": {"content": [{"type": "text", "text": "boucle"}]}}]})
        self.assertFalse(exists)


if __name__ == "__main__":
    unittest.main()
