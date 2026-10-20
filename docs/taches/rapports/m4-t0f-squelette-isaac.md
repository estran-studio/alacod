# Rapport M3-T0f — squelette du jeu isaac (branche `m4-t0f-squelette-isaac`)

(Tâche renumérotée M3-T0f ; branche et rapport gardent le nom M4-T0f.)

## Livré
- `games/isaac` : crate + `main.rs` (copie de gungeon), manifeste `game.ron` (mode `Floors`, étage `sous_sol`, grammaire 5–7 salles, boss à ≥ 2 de distance), personnage `pleureur` (6 PV, sans dash, arme `larmes`), ennemis `suiveur` / `mouche`, boss `gardien` (2 phases : `eventail` puis `couronne`), salles `depart`/`combat`/`boss`, objets `key`/`blank`/`ration`, loot, économie/ui/caméra/sons copiés, `assets.yaml` (squelette, sprites Kenney CC0 réutilisés).
- Cartes LDtk générées par `docs/taches/rapports/m4-t0f-squelette-isaac.make_salles.py` (salles 15×9, ouvertures de 3 cases centrées).
- Scénarios : `isaac_boots`, `isaac_floor_seed_1` (bot explorateur : boss verrouillé, phase 1, boss mort, victoire), et `tests/scenarios/generated/isaac/*` (`make gen GAME=isaac`).
- Test de grammaire `isaac_etages_de_grammaire` (`crates/map_ldtk/src/generation/mod.rs`, `ALACOD_FLOOR_SEEDS`), `isaac` ajouté au lint (Makefile), à la cible `make isaac`, au README et au test `embedded.rs`. Conventions §65.

## Écarts / points d'attention
- **Boss à une seule ouverture** : le planificateur exige un boss en cul-de-sac ; au lieu d'un gabarit à 4 ouvertures, quatre gabarits `BossN/E/S/W`.
- **Dégâts de mêlée ennemi** : lus dans `EnemyAiConfig` (`attack_damage` défaut 10, `attack_cooldown_frames` défaut 60), pas dans l'arme `Melee("contact")` ; renseignés à 1 / 120 dans les fiches ennemies (§65).
- Équilibrage (suiveur 10 PV, mouche 4, gardien 40) calibré pour que le bot finisse l'étage ; attentes de `isaac_floor_seed_1` recalées en conséquence.
- Arme `arsenal` (générateur) sans propriétaire : warning de lint possible, voulu.
- `assets.yaml` : squelette provisoire (en-tête neuf, contenu de gungeon).
- Aucune liste CI `.github` à modifier trouvée ; seules les boucles Makefile/README/test embedded ont changé.
- Aucune trace existante déplacée ; `make gen` des 4 autres jeux sans diff.

## Vérifications (toutes vertes)
`test_scenarios` (+ `par SHARDS=4`), `test_crates`, `make lint`, `cargo fmt --check`, `check_sim_boundary`, `make gen` ×5.
Non vérifié : **clippy** (absent du nightly) ; **p2p isaac à 2 clients** : à rejouer par l'orchestrateur.
Branche fusionnée avec la branche T0c (pas encore dans main) : merger T0c d'abord.
