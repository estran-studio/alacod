"""Sprites de lisibilité de `gungeon` : générés, à jour, déclarés CC0 (scripts/generer-sprites-gungeon.py)."""
import importlib.util
import re
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "generer-sprites-gungeon.py"
spec = importlib.util.spec_from_file_location("generer_sprites_gungeon", SCRIPT)
gen = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gen)

ASSETS = ROOT / "games" / "gungeon" / "assets"
LISIBLE = ASSETS / "sprites" / "lisible"


class SpritesGungeonTests(unittest.TestCase):
    def test_les_fichiers_du_depot_sont_ceux_que_le_script_ecrit(self):
        files, atlases = gen.build()
        for rel, data in files.items():
            on_disk = (LISIBLE / rel).read_bytes()
            self.assertEqual(on_disk, data, f"{rel} périmé : relancer scripts/generer-sprites-gungeon.py")
        text = (ASSETS / "sprites" / "sprites.ron").read_text(encoding="utf-8")
        self.assertIn(gen.sprites_block(atlases), text)

    def test_la_generation_est_deterministe(self):
        first, _ = gen.build()
        second, _ = gen.build()
        self.assertEqual(first, second)

    def test_aucun_fichier_orphelin_dans_le_dossier_genere(self):
        files, _ = gen.build()
        on_disk = {str(p.relative_to(LISIBLE)) for p in LISIBLE.rglob("*") if p.is_file()}
        self.assertEqual(on_disk, set(files), "fichier ajouté à la main ou supprimé du script")

    def test_chaque_png_est_enregistre_cc0_dans_assets_yaml(self):
        yaml = (ASSETS / "assets.yaml").read_text(encoding="utf-8")
        entry = yaml[yaml.index("- id: sprites-lisibilite-gungeon"):]
        self.assertIn("licence: CC0 1.0", entry)
        self.assertIn("placeholder généré", entry)
        for png in LISIBLE.glob("*.png"):
            self.assertIn(f"sprites/lisible/{png.name}", entry, f"{png.name} absent d'assets.yaml")

    def test_chaque_classe_de_la_legende_a_son_sprite(self):
        ids = set(re.findall(r'^    "([^"]+)": \(animation:', (ASSETS / "sprites" / "sprites.ron").read_text(encoding="utf-8"), re.M))
        hud = (ASSETS / "ui" / "hud.ron").read_text(encoding="utf-8")
        for sprite in re.findall(r'sprite: Some\("([^"]+)"\)', hud):
            self.assertIn(sprite, ids, f"légende : sprite « {sprite} » absent de sprites.ron")

    def test_les_silhouettes_des_familles_d_ennemis_different(self):
        _, atlases = gen.build()
        persos = next(a for a in atlases if a.name == "atlas_persos")
        shapes = {}
        for sid, idx in persos.entries:
            tile = persos.tiles[idx["Idle"]]
            shapes[sid] = tuple(1 if p[3] else 0 for p in tile.px)
        self.assertEqual(len(set(shapes.values())), len(shapes), "deux classes ont la même silhouette")


if __name__ == "__main__":
    unittest.main()
