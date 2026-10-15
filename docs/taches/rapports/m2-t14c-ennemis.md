# Rapport m2-t14c-ennemis (M2-T14 partie c, §52)

## 1. Fait
- Six ennemis (`shotgun_kin`, `veteran_kin`, `bandana_kin`, `tourelle_kin`, `fuyard_kin`, `lanceur_kin`), un
  `test:` chacun (12 scénarios générés `enemy_*`, `make gen GAME=gungeon`). Variantes §25 sur trois d'entre eux.
- Quatre patterns nommés (`pompe`, `rafale`, `croix`, `grenade_lente`), arme ennemie `arsenal_lourd`
  (`test:` + scénario `weapon_arsenal_lourd`). `arsenal` n'a pas été touché.
- Placement dans les gabarits existants (`CombatEW`, `CombatNS`, `CombatCroix`) et les vagues de l'arène, sans
  nouveau gabarit (script `m2-t14c-ennemis.make_placement.py`). `CombatT` et `EscouadeEW` inchangés.
- Boss `gatling` : phase 1 = couronne puis spirale (répétition 150). Détails en `docs/conventions.md` §52.

## 2. Vérifié (résultats réels)
- `make test_scenarios` : sans BLESS, 3 traces différentes, toutes les attentes vertes ; après `BLESS=1` : réussi.
- `make test_crates` : tout vert. `make lint` ×4 : 0 erreur. `cargo fmt --check` : OK.
- `make gen` ×4 : zombies, testbed et throne sans diff ; gungeon : 26 fichiers nouveaux, les générés existants
  inchangés.
- `alacod lint gungeon` : 12 avertissements (11 avant) ; aucun nouvel avertissement d'**ennemi** (`cible` seul).
  Le nouveau est l'arme `arsenal_lourd` (même cas qu'`arsenal`, dette D63).

## 3. Preuve §5 : traces qui bougent (toutes `gungeon`)
- `bench_floor` (dès la frame 0) : l'étage à grammaire `etage` contient plus d'ennemis (296 → 357 entités).
- `bot_explorer_floor` (f559), `bot_explorer_two` (f557) : entrée du bot dans la phase 1 du boss ; la spirale
  ajoute des balles (44 → 49 entités). Leurs attentes passent toujours.
- `gungeon_boss`, `gungeon_boss_bot`, `gungeon_floor_*`, `gungeon_two_floors` : inchangés (le boss meurt avant la
  spirale). Un renfort `lanceur_kin` en rage les déplaçait et tuait le bot : retiré.
- Aucune trace d'un autre jeu ne bouge.

## 4. Non fait / dettes
- **p2p** (Docker) : non rejoué ici, à faire par l'orchestrateur ; bench strict non lancé.
- `make test_scenarios_par` non utilisé (suite en série, ~48 min).
- Les ennemis de l'arène et de `CombatEW/NS/Croix` n'ont pas de scénario gungeon dédié qui les rencontre (seulement
  le gabarit `still`/`moving`) : à ajouter par T14d avec les gabarits.
- Pas d'escouade nouvelle (`veteran_kin` en escouade) : le type de salle `escouade` n'est tiré que par la
  grammaire `court_escouade` ; le brancher dans `etage` est T14d.
- Manque moteur : pas de renfort de boss par phase 1 sans casser les scénarios scriptés ; pas de projectile
  guidé pour un lanceur de missiles.
- Sprites : aucun (placeholders, comme `bullet_kin`).
