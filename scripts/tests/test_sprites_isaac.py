"""Murs, portes et objets du monde de `isaac` : générés, à jour, déclarés CC0 (scripts/generer-sprites-isaac.py)."""
import importlib.util
import re
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "generer-sprites-isaac.py"
spec = importlib.util.spec_from_file_location("generer_sprites_isaac", SCRIPT)
gen = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gen)

ASSETS = ROOT / "games" / "isaac" / "assets"
LISIBLE = ASSETS / "sprites" / "lisible"


class SpritesIsaacTests(unittest.TestCase):
    def test_les_fichiers_du_depot_sont_ceux_que_le_script_ecrit(self):
        files, atlases = gen.build()
        for rel, data in files.items():
            self.assertEqual((LISIBLE / rel).read_bytes(), data, f"{rel} périmé : relancer scripts/generer-sprites-isaac.py")
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
        entry = yaml[yaml.index("- id: sprites-murs-isaac"):]
        self.assertIn("licence: CC0 1.0", entry)
        for png in LISIBLE.glob("*.png"):
            self.assertIn(f"sprites/lisible/{png.name}", entry, f"{png.name} absent d'assets.yaml")

    def test_la_palette_n_est_pas_celle_de_gungeon(self):
        # Les murs d'isaac et de gungeon (même générateur) ne partagent aucun octet : palette « sous-sol » distincte.
        gungeon = (ROOT / "games" / "gungeon" / "assets" / "sprites" / "lisible" / "atlas_murs.png").read_bytes()
        self.assertNotEqual((LISIBLE / "atlas_murs.png").read_bytes(), gungeon)

    def test_la_feuille_des_murs_porte_les_tuiles_lues_par_le_moteur(self):
        # `wall_art` lit 36 tuiles de mur puis les tuiles annexes de `cells` : les piques sont l'indice 36, dans les lignes de la feuille.
        hud = (ASSETS / "ui" / "hud.ron").read_text(encoding="utf-8")
        tile = int(re.search(r"cells: \[\(layer: \"Surfaces\", value: 5, tile: (\d+)\)\]", hud).group(1))
        self.assertEqual(tile, gen.SPIKES_TILE)
        wa = gen.wall_atlas()
        self.assertGreaterEqual((wa.w // 16) * (wa.h // 16), tile + 1)

    def test_le_sol_est_dans_la_feuille_et_dans_le_hud(self):
        # S2-4 : `wall_art.floor.tiles` ne pointe que des tuiles de sol de la feuille (37 à 40), et chacune porte la couleur de fond du niveau
        # sur ses bords (aucune couture entre deux variantes voisines).
        hud = (ASSETS / "ui" / "hud.ron").read_text(encoding="utf-8")
        tiles = {int(t) for t in re.search(r"floor: Some\(\(tiles: \[([\d, ]+)\]", hud).group(1).replace(" ", "").split(",")}
        self.assertEqual(tiles, set(gen.FLOOR_TILES))
        wa = gen.wall_atlas()
        self.assertGreaterEqual((wa.w // 16) * (wa.h // 16), max(gen.FLOOR_TILES) + 1)
        base = gen.rgb(gen.FLOOR_BASE)
        for tile in gen.floor_tiles():
            edge = [tile.get(i, 0) for i in range(16)] + [tile.get(i, 15) for i in range(16)] + [tile.get(0, i) for i in range(16)] + [tile.get(15, i) for i in range(16)]
            self.assertTrue(all(px == base for px in edge))

    def test_halo_et_pointeur_sont_declares(self):
        ids = set(re.findall(r'^    "([^"]+)": \(animation:', (ASSETS / "sprites" / "sprites.ron").read_text(encoding="utf-8"), re.M))
        self.assertTrue({"halo", "aim"} <= ids)

    def test_les_portes_couvrent_l_ouverture_de_quatre_cases_dans_tous_les_etats(self):
        _, atlases = gen.build()
        for name, size in (("atlas_porte_v", (16, 64)), ("atlas_porte_h", (64, 16))):
            door = next(a for a in atlases if a.name == name)
            _, idx = door.entries[0]
            self.assertEqual(set(idx), {"Idle", "Open", "Locked", "Secret"})
            for tile in door.tiles:
                self.assertEqual((tile.w, tile.h), size)

    def test_chaque_sprite_de_nom_attendu_par_le_moteur_existe(self):
        ids = set(re.findall(r'^    "([^"]+)": \(animation:', (ASSETS / "sprites" / "sprites.ron").read_text(encoding="utf-8"), re.M))
        for needed in ("door_v", "door_h", "rock", "rock_loot", "chest", "shelf", "shell", "bomb", "portal", "item/key", "item/bomb", "item/coeur"):
            self.assertIn(needed, ids, f"sprites.ron : « {needed} » absent")


if __name__ == "__main__":
    unittest.main()
