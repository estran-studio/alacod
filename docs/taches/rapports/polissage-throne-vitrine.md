# Throne visuel, voie t : portail, roi rat, armes au sol, armes en main (`polissage-throne-vitrine`)

Branche `polissage-throne-vitrine` (depuis `origin/polissage-integration` `b083ec48`). Spec : `runs/alacod-polissage/t-throne-visuel.md`. Conventions § 105 ; dettes D180–D182.
Reprise après un plantage du Mac (16 h 10) : l'état du worktree a été commité tel quel (`94245eb2`), puis vérifié et complété.

## 1. Élément → source → licence → capture

Choix : les packs payants (Aymen B, Ittai, TheLazyStone) n'ont **aucun** portail, boss ni arme énergétique/lanceur (audit `polissage-throne-visuel.md` § 1.1) ; les éléments
manquants viennent de contributions OpenGameArt **CC0 / CC BY**, donc commitées (`assets.yaml` renseigné avant l'import, licence relue sur la page de chaque contribution le 2026-11-02).
Captures : `play_scenario --capture` natif, `ALACOD_MUTE=1`, **sans** surcharge locale (repli public, commité sous `docs/captures/polissage-throne-vitrine/`) ; l'état avec la surcharge
(six armes d'Aymen) est sous `~/orca/workspaces/alacod/captures/polissage-throne-vitrine/`.

| Élément | Avant | Après (public) | Source, licence (`assets.yaml`) | Capture (repli public) |
|---|---|---|---|---|
| Portail de l'étage | carré violet 40 px | anneaux de pierre et de lumière, case 32 × 32 ×2 | LetargicDev, `portalRings2.png`, OpenGameArt « Portals », **CC0** (`letargicdev-portals`) | `portail_f00570.png` (`throne_solo`, f570, portail ouvert en haut à droite) ; `portail_f00545.png` |
| Boss roi rat | Zombie 4 ×1,4 (dérivé) | rat couronné et cape, Idle 3 + Run 3 images, ×2 (collision inchangée) | Redshrike, « Rat and Rat King overworld », **CC BY 4.0**, attribution « Art by Redshrike, for OpenGameArt.org » (`redshrike-rat-king`, D180) | `boss_roi_rat_f00060.png`, `…f00120.png` (`enemy_roi_rat_still`) |
| Armes au sol : mortier, lance-grenades, laser, plasma, traqueur, disque, arsenal (les 7 sans image) | image générique (revolver) | une image propre par arme | knekko, « Futuristic guns », **CC0** (`knekko-futuristic-guns`) | lance-grenades, mortier, laser : `armes_b_f00110.png` ; plasma, traqueur, disque : `armes_c_f00110.png` ; arsenal : `armes_d_f00110.png` |
| Armes au sol : revolver, mitraillette, fusil à pompe, canon ricochet, roquette, lance-lames (les 6 déjà couvertes par Aymen) | Aymen (surcharge) / générique en public | Knekko en public, Aymen avec la surcharge | idem ; Aymen par `repack-throne.py` | `armes_*_f00110.png` (public), surcharge : `armes_a_f00110.png`, `armes_c_f00110.png` |
| Armes en main (13) | 3 images pour 13 (`pistol`, `machine_gun`, `shotgun`) | **une image par arme** (`held/<id>`) | idem | `armes_*_f00019.png` (visée à gauche/droite, arme en main) |

Scénarios de capture (jetables, hors `tests/scenarios`, ne passent pas par `make test_scenarios`) : `armes_a` (revolver, mitraillette, fusil à pompe, canon ricochet), `armes_b` (lance-grenades, roquette,
mortier, laser), `armes_c` (plasma, traqueur, lance-lames, disque), `armes_d` (arsenal) ; chaque joueur porte une arme (f19, visée), la lâche à f25, s'éloigne (f110). Les 13 armes sont donc vues en main **et** au sol.

## 2. Code touché

Un seul fichier Rust, présentation : `crates/game/src/ui/weapon_visuals.rs` (`attach_weapon_visuals`) cherche `held/<id de l'arme>` avant `sprite_config.name`. Sans entrée `held/…` le comportement est
celui d'avant (zombies, gungeon, isaac, testbed) ; `sprite_config.name` (dans le checksum) n'est pas modifié. Fichier hors de la propriété de la voie : signalé au coordinateur par `status` (2 nov).
Reste : `games/throne/assets/{sprites/sprites.ron, sprites/vitrine/*, assets.yaml}`, `scripts/repack-throne-cc0.py` (nouveau), `scripts/repack-throne.py` (`held/<id>` pour six armes d'Aymen ; `roi_rat` retiré de la surcharge),
`docs/conventions.md` § 105, `docs/taches/dettes.md` D180–D182. Dépôt privé `alacod-assets` : `throne/sprites/sprites.local.ron` + 12 `held_*` (commit `82083da`).

## 3. Preuves d'exécution

- Aucune trace ne bouge : aucun scénario n'a besoin de `BLESS` (voir le résultat de `make check` ci-dessous) ; aucune fonction du `GgrsSchedule` n'est touchée (`weapon_visuals.rs` : `Update`, présentation).
- Captures : natif, 960 × 540, avec et sans surcharge (liste au § 1).
- `make check` : voir § 4.

## 4. `make check`

Cibles de `make check` lancées une à une (cible partagée `.target-mac`, `env -u CI`, `ALACOD_MUTE=1`) :

| Cible | Résultat |
|---|---|
| `format`, `check_forbidden`, `check_rollback_registration`, `check_sim_boundary`, `lint` | exit 0 (lint : « games/isaac : aucune erreur … », idem pour les autres jeux) |
| `test` (`test_crates` + suite `scenarios` : 5 passés, 12 ignorés, 969 s) | exit 0, **sans `BLESS`** : aucune trace n'a bougé |
| `test_tools` | exit 2 : les 3 échecs connus du Mac (`test_claude_say.py`, Python 3.9.6), autorisés par la spec |

Premier passage de `make test` : 4 échecs de `content` (chemins `…/alacod/i1/…` compilés dans un binaire laissé par une autre voie dans la cible partagée) ; `touch crates/content/src/lib.rs` a recompilé le crate, puis vert.

## 5. Ce qui reste

| Reste | Estimation |
|---|---|
| D180 : écran ou fichier de crédits de throne avant toute publication (attribution Redshrike) | 0,25 j |
| D181 : portail animé (moteur : `attach_world_sprites` ne joue qu'une case) et violet si William préfère la teinte d'avant | 0,25 j |
| D182 : décision de style des armes (tout Knekko, ou tout Aymen via le dépôt privé) et poignée alignée sur la main | 0,25 j |
| Armes tenues à l'oblique : aucune rotation propre par arme (le moteur retourne l'image selon la direction) | non mesuré |
