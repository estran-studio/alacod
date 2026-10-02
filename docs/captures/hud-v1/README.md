# Captures de référence du HUD v1 (T2.12)

Rendu hors écran par `play_scenario --capture` (960×540, image n = frame n), profil
`headless`, rendu `--features render`. Le HUD est décrit par
`games/zombies/assets/ui/hud.ron` (voir `CLAUDE.md` § HUD et `docs/conventions.md` §15).

| Fichier | Scénario | Frame | Ce qu'on voit |
|---|---|---|---|
| `prompt_achat.png` | `tests/scenarios/shop_tour.ron` | 100 | prompt « [H] Acheter pistolet — $500 » devant l'arme murale |
| `perk_powerup.png` | `perk_powerup.ron` (ce dossier) | 260 | icône « J » (Juggernog) au-dessus de la vie ; « Double Points 26 s » et « Insta-Kill 26 s » en haut à droite |
| `a_terre.png` | `tests/scenarios/downed_revive.ron`, `--follow 1` | 200 | « À TERRE 14 s » et « Réanimation 47 % » vus par le joueur à terre |
| `ecran_fin.png` | `tests/scenarios/downed_all_lose.ron` | 1506 | « DÉFAITE », résumé de la run, boutons « Rejouer (R) » et « Lobby » |

Refaire une capture (exemple) :
```bash
cargo run -p scenario --profile headless --features render --bin play_scenario -- \
  tests/scenarios/shop_tour.ron --capture /tmp/cap --every 10
```
Sans écran ni GPU (session cloud) : `xvfb-run -a` et le Vulkan logiciel de Mesa
(`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json WGPU_BACKEND=vulkan`).

Les textes de debug (caméra en haut à gauche, « Wave 1 | PREP … » en bas, numéro de frame et
FPS en bas à droite) ne font pas partie du HUD.
