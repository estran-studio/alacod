# Rapport : dettes-outillage-d15-d34-d60-d63 (session cloud, 4 coeurs)

## 1. Fait
- **D15** : la grille de collision ne gagne rien, elle coûte. Interrupteur temporaire non livré (`query_aabb`
  balaie toutes les entrées, mêmes résultats triés par `net_id`), 3 passes alternées, profil `headless` :
  `bench_bullets` 146 fps avec grille / 161 sans (×0,91) ; `bench_horde` 46,8 / 55,2 (×0,85) ;
  `bench_bullets_room` 85 / 100 (×0,85). Réserve : le balayage n'est pas le code d'avant T2.1 (l'insertion dans
  la grille reste payée), une seule machine. Aucun code livré. Suite possible : retirer la grille (décision de William).
- **D34** : écart du CORDIC mesuré sur ]−4π, 4π] : ≤ 5e-5 sur [−3π/2, 3π/2], 1e-4 dès 5,24 rad, 1e-3 dès 5,78,
  7e-3 en 2π, panique au-delà de 2π+0,03. Il touche du contenu réel (dernier rayon d'une couronne de 24 : 6,02 rad,
  norme ≠ 1). `direction_of` replie dans [−3π/2, 3π/2] ; les angles déjà exacts restent identiques au bit près
  (un premier essai dans ]−π, π] déplaçait plus de 40 traces, abandonné). Tests : `bevy_fixed`
  (`cordic_error_is_bounded_…`), `combat` (`direction_of_is_accurate_on_wide_range_d34`).
  Preuve §5 : dumps `main` vs branche sur `enemy_ring`, `pattern_spiral`, `enemy_telegraph` : la première frame qui
  diffère est celle de la première couronne, seules des `Bullet` de rayons > 3π/2 changent
  (ex. velocity (1, −0,00694) → (1,00002, −0,00006)), tout le reste est identique. 34 traces blessées.
  **Non replié (dette restante)** : les tirages d'angle ∈ [0, 2π) de `calculate_spawn_position`
  (`waves/systems.rs`), `enemy/spawning.rs` et `Wander` (`rules.rs`) : position d'apparition et direction d'errance
  avec un écart ≤ 7e-3 pour ~16 % des tirages ; les replier déplacerait d'autres traces, hors de la fiche.
- **D60** : `scripts/scenario-video` (`render`, `views`, `compare`) résout `tests/scenarios/<nom>.ron`, puis
  `generated/*/<nom>.ron`, `jeu/nom` accepté, ambiguïté = erreur qui liste les chemins ; absence de GPU Vulkan =
  arrêt avant compilation avec message ; `DRY_RUN=1` prouve la résolution (testé : simple, généré, `jeu/nom`,
  ambigu `item_bottes`, inconnu). Non testé : une capture réelle (pas de GPU ici).
- **D63** : `test_only: true` (optionnel, défaut `false`) sur personnage et arme (`config:`), lu par `content`
  seulement ; `content::reach` l'ignore (et un personnage marqué n'est pas une source d'armes). Marqués : `cible`,
  `mannequin` (throne), `cible` (gungeon). Fixtures `reach_test_only_character/weapon`, conventions §53.
  Restent deux avertissements, à décider :
  - `zombie_scaled` (zombies) : personnage de preuve de l'équilibrage par nombre de joueurs (F5), joué par
    `equilibrage_joueurs_duo` et `equilibrage_joueurs_quad` (carte `exemples/zombie_scaled.ldtk`) ; proposition :
    le marquer `test_only`.
  - `arsenal` (throne, gungeon) : arme des ennemis (leurs `Shoot`/`SpawnPattern` la citent ; aussi scénario généré
    `weapon_arsenal`) ; ce n'est pas du contenu de test : proposition = l'analyse lit les armes citées par les règles
    des ennemis atteints, pas de marqueur.
- **Question ouverte M2-T17** : le délai de 6 frames n'est pas un défaut. Cause mesurée : `input_delay: 5` de la
  session GGRS locale (`args/mod.rs`), appliqué à tous les boutons ; `item_use_active_system` agit à la frame
  simulée 15 (sonde `eprintln`, retirée) = 10 + 5, et l'attente « frame 16 » lit l'état après cette frame. Aucun
  anti-rebond ni ordre de systèmes en cause. `Fire` a le même délai (masqué par la cadence et les attentes larges).

## 2. Vérifié (fusion avec `origin/main` ec69ebc, commit a1e0f77)
- `make test_scenarios_par` (4 tranches, sans BLESS) : 288 scénarios verts, 0 trace différente (383 s). Pas
  `make test_scenarios` monothread (même binaire, même comparaison de traces).
- `make test_crates` : 692 réussis / 0 échec / 10 ignorés. `cargo fmt --all -- --check` : propre.
- `alacod lint` des 4 jeux : aucune erreur. `check-rollback-registration.sh` : OK. `check-forbidden.sh` : 6
  avertissements (identiques à ceux annoncés pour main, aucun ajouté par mes fichiers).
- `make gen` ×4 : non lancé (aucune arme modifiée).

## 3. Non vérifié
- p2p à deux clients et restart en ligne (pas de Docker) : **à rejouer par l'orchestrateur, D34 change la simulation**.
- Bench strict au calme, capture vidéo avec GPU.

## 4. Dettes / questions
- Retirer ou garder la grille de collision (D15) ; replier les tirages d'angle de spawn et d'errance (D34) ;
  `zombie_scaled` et `arsenal` (D63).

## 5. Atteignabilité
D34 : `enemy_ring`, `pattern_spiral`, `throne_*`, `gungeon_floor_*` rencontrent le repli. D63 : lint des jeux livrés.
