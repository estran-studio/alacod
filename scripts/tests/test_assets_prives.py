"""Surcharge privée (S-0b, docs/conventions.md § 99) : le dépôt public n'en suit rien, le script la copie sans fuite,
et le build web ne l'embarque que sous ALACOD_WEB_PRIVATE_ASSETS=1 (D161)."""
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def private_rules():
    dirs, suffixes = [], []
    for line in (ROOT / "scripts" / "assets-prives.patterns").read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("*"):
            suffixes.append(line[1:])
        else:
            dirs.append(line)
    return dirs, suffixes


def is_private(rel):
    dirs, suffixes = private_rules()
    return any(rel.startswith(d) for d in dirs) or any(rel.endswith(s) for s in suffixes)


class PublicRepoTests(unittest.TestCase):
    def test_aucun_fichier_de_surcharge_n_est_suivi_sous_games(self):
        tracked = subprocess.run(["git", "ls-files", "games"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.splitlines()
        leaked = []
        for path in tracked:
            parts = path.split("/", 3)  # games/<jeu>/assets/<rel>
            if len(parts) == 4 and parts[2] == "assets" and is_private(parts[3]):
                leaked.append(path)
        self.assertEqual(leaked, [], "fichiers de la surcharge privée suivis par git (dépôt public)")

    def test_les_chemins_de_la_surcharge_sont_gitignores(self):
        for rel in ["sprites/postapo/x.png", "sprites/postapo/hud/x.png", "sprites/sprites.local.ron", "ui/hud.local.ron", "sounds/sonniss/x.ogg"]:
            self.assertTrue(is_private(rel), rel)
            result = subprocess.run(["git", "check-ignore", "-q", "--", f"games/zombies/assets/{rel}"], cwd=ROOT)
            self.assertEqual(result.returncode, 0, f"games/zombies/assets/{rel} n'est pas gitignoré")


class ScriptTests(unittest.TestCase):
    """Le script et la liste copiés dans un dépôt jetable (jamais le vrai `games/`)."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="assets-prives-test-"))
        (self.tmp / "scripts").mkdir()
        for name in ["assets-prives.sh", "assets-prives.patterns"]:
            shutil.copy(ROOT / "scripts" / name, self.tmp / "scripts" / name)
        shutil.copy(ROOT / ".gitignore", self.tmp / ".gitignore")
        (self.tmp / "games" / "zombies" / "assets").mkdir(parents=True)
        subprocess.run(["git", "init", "-q"], cwd=self.tmp, check=True)
        self.source = self.tmp / "private"
        (self.source / "zombies" / "sprites" / "postapo").mkdir(parents=True)
        (self.source / "zombies" / "sprites" / "postapo" / "a.png").write_bytes(b"png")
        (self.source / "zombies" / "sprites" / "sprites.local.ron").write_text("()")
        (self.source / "licences").mkdir()
        (self.source / "licences" / "x.txt").write_text("licence")

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def run_script(self, *args, env_extra=None):
        env = {k: v for k, v in os.environ.items() if k != "ALACOD_ASSETS_DEPLOY_KEY"}
        env.update(env_extra or {})
        return subprocess.run(["bash", str(self.tmp / "scripts" / "assets-prives.sh"), *args], cwd=self.tmp, capture_output=True, text=True, env=env)

    def test_copie_idempotente_et_licences_ignorees(self):
        first = self.run_script("--source", str(self.source))
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertIn("2 copiés, 0 déjà à jour", first.stdout)
        self.assertEqual((self.tmp / "games/zombies/assets/sprites/postapo/a.png").read_bytes(), b"png")
        self.assertFalse((self.tmp / "games/licences").exists())
        second = self.run_script("--source", str(self.source))
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertIn("0 copiés, 2 déjà à jour", second.stdout)

    def test_refuse_un_chemin_non_couvert_et_ne_copie_rien_de_public(self):
        (self.source / "zombies" / "sprites" / "public.png").write_bytes(b"png")
        result = self.run_script("--source", str(self.source))
        self.assertEqual(result.returncode, 1)
        self.assertIn("aucune règle", result.stderr)
        self.assertFalse((self.tmp / "games/zombies/assets/sprites/public.png").exists())

    def test_refuse_un_chemin_couvert_mais_non_gitignore(self):
        (self.source / "zombies" / "feedback.local.ron").write_text("()")
        result = self.run_script("--source", str(self.source))
        self.assertEqual(result.returncode, 1)
        self.assertIn("pas gitignoré", result.stderr)
        self.assertFalse((self.tmp / "games/zombies/assets/feedback.local.ron").exists())

    def test_source_absente_est_une_erreur(self):
        result = self.run_script("--source", str(self.tmp / "inexistant"))
        self.assertEqual(result.returncode, 1)
        self.assertIn("introuvable", result.stderr)

    def test_ci_sans_cle_saute_sans_rien_ecrire(self):
        github_env = self.tmp / "github_env"
        result = self.run_script("--ci", env_extra={"GITHUB_ENV": str(github_env)})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("étape sautée", result.stdout)
        self.assertFalse(github_env.exists(), "le drapeau ne doit pas être posé sans clé")
        self.assertEqual(list((self.tmp / "games/zombies/assets").iterdir()), [])


class WebBuildCopyTests(unittest.TestCase):
    """D161 : `scripts/web-assets.mjs` copié dans un dépôt jetable avec une surcharge présente sur le disque."""

    def setUp(self):
        if shutil.which("node") is None:
            self.skipTest("node absent")
        self.tmp = Path(tempfile.mkdtemp(prefix="web-assets-test-"))
        (self.tmp / "scripts").mkdir()
        for name in ["web-assets.mjs", "assets-prives.patterns"]:
            shutil.copy(ROOT / "scripts" / name, self.tmp / "scripts" / name)
        assets = self.tmp / "games" / "zombies" / "assets"
        for rel in ["game.ron", "sprites/sprites.ron", "sprites/postapo/a.png", "sprites/sprites.local.ron", "ui/hud.local.ron", "sounds/sonniss/a.ogg", "sounds/shot.ogg"]:
            (assets / rel).parent.mkdir(parents=True, exist_ok=True)
            (assets / rel).write_bytes(b"x")

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def copied(self, env_extra):
        out = self.tmp / "out"
        shutil.rmtree(out, ignore_errors=True)
        env = {k: v for k, v in os.environ.items() if k != "ALACOD_WEB_PRIVATE_ASSETS"}
        env.update(env_extra)
        result = subprocess.run(["node", "scripts/web-assets.mjs", "zombies", str(out)], cwd=self.tmp, capture_output=True, text=True, env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        return sorted(str(p.relative_to(out)) for p in out.rglob("*") if p.is_file())

    def test_sans_drapeau_aucun_fichier_de_la_surcharge(self):
        self.assertEqual(self.copied({}), ["game.ron", "sounds/shot.ogg", "sprites/sprites.ron"])
        self.assertFalse((self.tmp / "out" / "sprites" / "postapo").exists(), "le dossier de la surcharge ne doit pas exister, même vide")

    def test_un_drapeau_autre_que_1_n_inclut_rien(self):
        self.assertEqual(self.copied({"ALACOD_WEB_PRIVATE_ASSETS": "true"}), ["game.ron", "sounds/shot.ogg", "sprites/sprites.ron"])

    def test_avec_drapeau_la_surcharge_est_copiee(self):
        self.assertEqual(
            self.copied({"ALACOD_WEB_PRIVATE_ASSETS": "1"}),
            ["game.ron", "sounds/shot.ogg", "sounds/sonniss/a.ogg", "sprites/postapo/a.png", "sprites/sprites.local.ron", "sprites/sprites.ron", "ui/hud.local.ron"],
        )


if __name__ == "__main__":
    unittest.main()
