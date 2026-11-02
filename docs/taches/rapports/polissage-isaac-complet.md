# polissage-isaac-complet — voie i, semaine 2 (web, S2-4 visuel, S2-5 sons)

Branche `polissage-isaac-complet` (WIP `6d13358d` repris, `origin/polissage-integration` fusionnée). Conventions : `docs/conventions.md` § 106.
Dettes : D183 (sons sans écoute), D184 (personnages hors pack), D185 (rendu jugé sans capture). S2-6 (map, équilibrage) : abandonné cette semaine.
Règles d'économie respectées : aucun navigateur ni headless, aucune campagne, aucun `BLESS`, aucun son (`ALACOD_MUTE=1`), aucun achat, zombies intact.

## Web (mardi)

| Point | Issue | Preuve |
|---|---|---|
| Listes blanches `isaac` | fait | `scripts/build-web-games.mjs:9`, `prepare-cloudflare.mjs:31`, `web-assets.mjs:49`, `website/functions/builds/[[path]].js:9`, `website/static/loader.js:63` |
| Catalogue du site | fait | carte `isaac` (M3, « Roguelike · Salles ») dans `website/src/lib/game/catalog.ts` ; page « Jouer » : E bombe, Espace objet actif, Q blank |
| Drapeaux `solo: true` `online: false` | non applicable au dépôt | à poser dans le `releases.json` du build local du coordinateur (consigne de la spec) |
| Lint isaac | fait | `alacod lint games/isaac` : « aucune erreur (24 personnages, 3 armes, 2 armes de corps à corps, 0 vagues, 3 cartes) », avec et sans surcharge locale |
| Compilation wasm | fait au WIP | `cargo check` wasm vert au commit `91003428` ; le build servi est celui du coordinateur (aucun build lancé ici) |
| Manette (S1-1) | vérifié à la lecture | `get_input_map` (§ 93) : déplacement stick gauche, visée stick droit, tir RT, objet actif LT, bombe A, blank croix haut, recharger X, interagir Y, mêlée B, arme RB, lâcher croix bas. Seule `Modifier` (dash arrière) n'a pas de bouton : isaac n'a ni dash ni sprint (`dash_distance: "0.0"`), donc **aucune action d'isaac sans bouton**. Non essayé en jeu (D160) |

## S2-4 — visuel

Sources : Aymen B (« UPDATE 2_ZombiePack »), Ittai Manero (« Zombie Apocalypse Tileset »), déjà achetés ; originaux du projet pour le sol, le halo et le
pointeur (CC0). Le pack privé est une surcharge locale gitignorée (`sprites/postapo/`, `sprites/sprites.local.ron`), copiée dans le dépôt privé
`alacod-assets` (`isaac/`, commit `d7debd1`, 68 fichiers avec les sons) ; sans elle, tuiles Kenney (repli public).

| Élément | Avant | Après (public) | Après (avec le pack privé) |
|---|---|---|---|
| Sol | uni `#7B6A5F` | quatre tuiles de terre 37–40 (grain, caillou, fissure ; bords unis, aucune couture) | idem |
| Ramassables | aucun signe | halo pulsant (`halo`) | idem |
| Ennemis qui visent | rien | pointeur rouge (`aim`) | idem |
| Couleur du joueur | aucune | `player_colors` bleu, vert, jaune, violet | idem |
| Légende (F7) | absente | `legend` de `hud.ron` (silhouettes tirées de `sprites.ron`) | idem, silhouettes du pack |
| `player` (héros, pleureur) | Kenney | Kenney | Aymen B `player` |
| `suiveur`, `rampant`, `cracheur`, `bourdon` (0,75), `spectre` | Kenney | Kenney | Aymen B `zombie2`, `zombie1`, `zombie3`, `zombie3`, `bandit` |
| `gardien` (boss, ×2,2) | Kenney | Kenney | Aymen B `zombie4` |
| `araignee`, `mouche`, `follet`, `arroseur`, `tourelle_croix` | Kenney | Kenney | Ittai : enfant, oiseau, oiseau, assis, tourelle |
| `colosse` (boss) | Kenney | Kenney | Ittai : gros zombie |
| `cible`, `cible_fragile`, `cible_mordante` | Kenney | Kenney | Ittai : canette, épouvantail, maigre |
| Familiers, `garde`, `ombre`, `sentinelle` | Kenney | Kenney | Kenney (D184) |

États animés : Idle et Run pour toutes les figures du pack ; Dying (Aymen B seulement ; Ittai n'a pas de mort animée). Aucun pixel dessiné pour le pack
(découpe, collage, échelle entière ou demi : `scripts/repack-isaac.py`). **Non vérifié** : aucune capture, aucun navigateur (règle d'économie) ; la partie du soir
de William fait foi (D185). Tests : `scripts/tests/test_sprites_isaac.py` (10 verts, dont sol dans la feuille et dans le HUD, halo et pointeur déclarés).

## S2-5 — sons

Code (présentation, `Update`, hors `GgrsSchedule`, hors hash) : `SOUND_KEYS` gagne `shell` et `active` ; préfixe `pickup_<objet>` (lint : l'objet doit exister) ;
`feedback_sounds.rs` : `play_purchase_and_pickup_sounds` joue `pickup_<objet>` puis `pickup`, `shell` pour un `CurrencyEvent` positif de motif `pickup` ;
`play_active_sound` dérive `active` de la charge de `Inventory::active` qui retombe à 0. Tests : `content` 141 verts (`cles_de_ramassage_par_objet_et_sons_de_sous_et_d_actif`), `game`
(`objet_actif_utilise_quand_sa_charge_retombe_a_zero`) vert.

| Événement | Clé | Public (commité) | Privé (Sonniss, `feedback.local.ron`) |
|---|---|---|---|
| Mèche de bombe | `bomb` | Kenney `impactTin_medium_001` | Interface Pop High Short |
| Explosion (bombe, rochers) | `explosion` | Snake's 762x54r, −12 demi-tons | Epic Stock Media Anime Game, Explosion Small Blast |
| Sous ramassés | `shell` | Kenney `impactMetal_light_000` | Foley Coin Flip Single Fast |
| Objet actif utilisé | `active` | Kenney `impactBell_heavy_002` | Epic Stock Media, Power Up Bright Positive 05 |
| Clé / bombe / cœur / cœur d'âme ramassés | `pickup_key`, `pickup_bomb`, `pickup_coeur`, `pickup_coeur_ame` | Kenney `impactMetal_medium_000`, `impactMetal_heavy_000`, `impactSoft_medium_000`, `impactGlass_light_000` | aucun (D183) |
| Autre objet ramassé, power-up | `pickup` | Kenney `impactGlass_light_002` | Game Entry Happy Short |
| Boss apparaît / phase / mort | `boss_spawn`, `boss_phase`, `boss_death` | Kenney `impactMetal_heavy_001`, `impactMining_001`, `impactPlate_heavy_004` | T Rex, Jumpscare Vocal, Werewolf Growl |
| Étage suivant | `floor` | Kenney `impactPlate_medium_003` | Transition Braam Slow Dark Creepy |
| Salle nettoyée | `room_cleared` | Kenney `impactBell_heavy_004` | Interface Plucks Happy |
| Achat | `purchase` | Kenney `impactMetal_light_003` | Ting Coins |
| Mort du joueur | `death_player` | Kenney `impactSoft_heavy_003` | Pain Vocal Dying |
| Mort d'un ennemi par famille | `death_<id>` (19 ennemis) | `death` (inchangé) | bêtes, humains, spectres, brutes (boss compris), machines |
| Menus | `menu_navigate`, `menu_confirm`, `menu_back` | Kenney | Cinematic Sound Design |

Le contenu d'isaac n'a que **deux** personnages boss (`gardien`, `colosse`, pas neuf) : les clés `boss_*` sont globales, la mort d'un boss joue aussi son `death_<id>` (brute).
Licences d'abord : `assets.yaml` (`sound-snake-authentic-gun-sounds-isaac`, `sound-impact-kenney-isaac`, `sound-sonniss-gdc-2026-isaac`, `aymen-b-zombiepack-isaac`,
`ittai-zombie-apocalypse-isaac`). **Non vérifié à l'oreille** (aucun son sur le Mac) : choix faits sur le nom et la durée, à juger par William (D183).

## Vérifications

| Commande | Résultat |
|---|---|
| `alacod lint games/isaac` | aucune erreur |
| `cargo test -p content` / `cargo test -p game feedback` | verts (141 ; 29 ; sans échec) |
| `python3 -m unittest scripts.tests.test_sprites_isaac` | 10 verts |
| `make check` | `format` vert ; `test_tools` : 47 verts, 3 échecs de `test_claude_say` (Mac, exclus par la spec, sans lien : script de voix) ; puis `check_forbidden check_rollback_registration check_sim_boundary test lint` verts d'un trait (35 min, `test` = `test_crates` + `test_scenarios` sans `BLESS`, dernière ligne `games/isaac : aucune erreur (24 personnages, …)`) |
| Traces | `test_scenarios` complet vert sans `BLESS` : aucune trace n'a bougé ; les fichiers touchés sont de la présentation et des données non lues par `GgrsSchedule` ; aucun `BLESS`, aucun `schedule_order_diff` (simulation intacte) |

## Reste

- Écoute des sons et réglage des volumes (D183) ; échelles des figures du pack à juger sur la partie du soir (D185).
- Familiers et trois ennemis sans figure du pack (D184), sol par étage.
- S2-6 (map, équilibrage D93, D125 avec rendu) : non fait, abandonné cette semaine.
