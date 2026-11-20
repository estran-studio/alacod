"""Trous de la musique (M-7b, docs/conventions.md § 124) : les débuts d'effets sont lus dans le repère du WAV musical, chaque
flux du pacer ayant sa propre origine (`monotonic=` de son journal)."""
import math
import struct
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "audio-gaps.py"
RATE = 8000


def write_wav(path, seconds, loud):
    """WAV flottant mono : un sinus aux instants où `loud(t)` est vrai, du silence ailleurs."""
    frames = int(seconds * RATE)
    samples = [0.3 * math.sin(2 * math.pi * 440 * i / RATE) if loud(i / RATE) else 0.0 for i in range(frames)]
    data = struct.pack(f"<{frames}f", *samples)
    header = b"RIFF" + struct.pack("<I", 36 + len(data)) + b"WAVEfmt " + struct.pack("<IHHIIHH", 16, 3, 1, RATE, RATE * 4, 4, 32)
    path.write_bytes(header + b"data" + struct.pack("<I", len(data)) + data)


def write_log(path, origin):
    path.write_text(f"open pid=1 stream=0 name=default monotonic={origin:.4f}\nt=0.0001 start fifo=0\n", encoding="utf-8")


class AudioGapsTests(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())
        # Musique ouverte à 100 s : un trou de 3,0 à 3,5 s de son WAV.
        write_wav(self.dir / "musique.wav", 6.0, lambda t: not 3.0 <= t < 3.5)
        # Effets ouverts 1 s plus tard (101 s) : un coup à 2,1 s de leur WAV = 3,1 s de la musique, dans le trou.
        write_wav(self.dir / "effets.wav", 5.0, lambda t: 2.1 <= t < 2.2)

    def run_gaps(self, *extra):
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(self.dir / "musique.wav"), "--effects", str(self.dir / "effets.wav"), *extra],
            capture_output=True,
            text=True,
        )

    def test_les_effets_sont_ramenes_dans_le_repere_de_la_musique(self):
        write_log(self.dir / "musique.log", 100.0)
        write_log(self.dir / "effets.log", 101.0)
        out = self.run_gaps()
        self.assertEqual(out.returncode, 1, out.stdout + out.stderr)
        self.assertIn("origine décalée de +1.00 s", out.stdout)
        self.assertIn("trou : 3.00 s → 3.50 s (500 ms) ; effet à 3.10 s", out.stdout)

    def test_une_origine_anterieure_eloigne_l_effet_du_trou(self):
        # Effets ouverts 1,5 s **avant** la musique : le coup tombe à 0,6 s de la musique, à plus d'une seconde du trou.
        write_log(self.dir / "musique.log", 100.0)
        write_log(self.dir / "effets.log", 98.5)
        out = self.run_gaps()
        self.assertIn("origine décalée de -1.50 s", out.stdout)
        self.assertIn("débuts (repère de la musique) : 0.60", out.stdout)
        self.assertIn("trou : 3.00 s → 3.50 s (500 ms)\n", out.stdout)

    def test_les_options_remplacent_les_journaux(self):
        out = self.run_gaps("--music-origin", "50", "--effects-origin", "51")
        self.assertIn("effet à 3.10 s", out.stdout)

    def test_une_origine_inconnue_est_une_erreur(self):
        write_log(self.dir / "musique.log", 100.0)
        out = self.run_gaps()
        self.assertEqual(out.returncode, 2)
        self.assertIn("origine d'un flux inconnue", out.stderr)


if __name__ == "__main__":
    unittest.main()
