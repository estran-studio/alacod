# M-5 — Le cœur à deux : PV, feu follet, relève

**Jalon servi** : S2, **dim 22 (la porte)** ; cible jeu **19 nov** (« le joueur mort devient un feu follet et son coéquipier peut le relever ») et **lun 16** (« la vie, les dégâts, les
munitions et leur affichage »). Plan : `docs/taches/plan-1837-cli.md` § 6, ligne M-5 (sa preuve fait foi). Design : `1867_lore/design/jeu.md` § 11. Conventions : `docs/conventions.md` **§ 113**.

- Engine : branche `wquintal/m-5-coeur-a-deux` (partie d'`origin/main` `0b484d15`), **code `cce86ac0`** (inclut le merge d'`origin/main` `30f9d1e6`, conflits de docs seulement) ; la tête de la branche ajoute ce rapport mis à jour et rien d'autre (docs seules).
- 1837 : branche `m-5-coeur` (partie d'`origin/main` `61563bf`), tête `89dc3719` (RON, scénarios de preuve, protocole du soir).
- Journal du contrôle rapide : `~/orca/runs/alpha/logs/m-5-coeur-a-deux-check-rapide-cce86ac0.log` (`make check_rapide` vert, exit 0).

## Ce qui est livré, point par point

| # | Point du lot | Issue | Preuve |
|---|---|---|---|
| 1 | Relevé avant tout code (`status`, dix lignes `fichier:ligne`) | **fait** | message `msg_bbfb3ac75c7e`, accepté par le coordinateur ; plan retenu tel quel (un mode en données de `Downed`, pas de second système) |
| 2 | Engine générique : mode feu follet, point de reprise, relève, défaite à deux, lint, schéma | **fait** | `sim_core::wisp::{WispDef, WispLure}` (champ `wisp` de `CharacterConfig`, `describe`/schéma par `CharacterFileSchema`), `combat::downed::{Wisp, RespawnPoint, is_decoy}`, `character::health::rollback_wisp_respawn`, leurre dans `enemy::ai::navigation` (`SourceReach`) ; lint `wisp.*` et `respawn_range` (`crates/content/tests/m5_wisp_fixtures.rs`, 4 fixtures) |
| 3 | Présentation : feu follet visible, HUD (PV, munitions, état feu follet, relève) | **fait** (rendu à l'œil : **non vérifié**) | `game::wisp_visuals` (gizmos), source HUD `wisp` (`ui::hud_model`, test `wisp_source_texts`), captures § « Captures » ; `health` et `ammo` déjà lues, vérifiées par `HudText` dans `wisp_revive_by_mate` |
| 4 | 1837 : RON (mode du Frère, durée de relève, point de reprise dans la salle des bancs), `alacod lint` vert | **fait** | `assets/characters/player.ron` (bloc `wisp`, `revive_frames`), `assets/characters/feu-de-camp.ron` (`respawn_range`), `assets/maps/salle.ldtk` (case (19, 14), par `scripts/poser-decor.py --salle`), `assets/ui/hud.ron` (widget `wisp`) ; `alacod lint` : **aucune erreur** (21 personnages) ; les deux scénarios `docs/preuves/m-5/{releve,reprise}.ron` rejoués en headless **sans échec** |
| 5 | Preuve : scénarios neutres à deux joueurs, un à un joueur, traces neuves seulement, `schedule_order_diff`, `check_rapide`, capture à deux joueurs, protocole du soir | **fait** | voir ci-dessous |
| 6 | § 113, D201–D204, rapport, LIVRÉ | **fait** | `docs/conventions.md` § 113, `docs/taches/dettes.md`, ce rapport, `docs/taches.md` |

### Le mode, en bref (détail : § 113)

Un personnage qui déclare `wisp: Some((speed_mult, lure, bleedout_frames, respawn_frames, respawn_health))` et qui tombe à 0 PV **en coop** (règle inchangée : un autre joueur debout, sinon `Death`) reçoit
`Downed` **et** `Wisp` (composant rollback **neutre**). Tout ce que `Downed` interdit déjà l'est donc (tir, mêlée, dash, interaction, dégâts ignorés) ; il marche à `speed_mult`. **Aucun champ n'est ajouté à
`Downed`** : son hash, donc toute trace sans feu follet, ne bouge pas.

- **Leurre** (`Attract(range)` / `Mislead(range)`, ou `None`) : le feu follet devient une source du Dijkstra multi-source du champ de flux, bornée à sa portée (coût de chemin) ; `Mislead` décale en plus les joueurs
  debout de `portée + 1` : dans la portée, les ennemis ne vont que vers la lumière. `enemy_target_selection` et `update_enemy_targets` la rangent parmi les cibles ; l'attaque ignore toujours un joueur à terre
  (les égarés se massent sur la lumière sans la frapper).
- **Relève** : par un coéquipier (interaction tenue, `revive_frames` du personnage, `handle_revive_interaction` retire aussi `Wisp`) ou **au point de reprise** (`respawn_frames` consécutives dans la portée d'un
  `RespawnPoint`, `Health = max × respawn_health`).
- **Point de reprise** : `combat::downed::RespawnPoint`, porté par un **personnage** neutre immobile qui déclare `respawn_range` (D201 : le premier essai, une entité LDtk dédiée et des placements
  `respawn_points`, **marchait dans les scénarios mais pas dans le jeu** : la génération de carte ne recopie que les types d'entité qu'elle connaît ; vu en rejouant la salle de 1837, la balise n'existait pas ;
  tout cela a été retiré au profit du personnage, qui passe par `CharacterSpawn` et `characters` sans chemin nouveau).
- **Les deux tombés** : la règle existante (`rollback_check_defeat`, tout joueur `Downed` ou `Death`) ; le second à tomber n'a personne pour le relever, il meurt, `Run` passe en `Defeat`.

### Scénarios (testbed, `testbed/arene_vide.ldtk`, personnages neutres `wisp_*`, `respawn_beacon`)

| Scénario | Ce qu'il prouve | Joueurs |
|---|---|---|
| `wisp_revive_by_mate` | mort → feu follet (`PlayerDowned`, HUD « FEU FOLLET », PV 0) → relève par le coéquipier (« Relève », `PlayerRevived`) → debout à 30 PV, texte vide | 2 |
| `wisp_respawn_point` | le feu follet **marche** (position à f150) jusqu'à la balise, « Reprise », se relève seul à 50 PV, sans coéquipier | 2 |
| `wisp_both_down_defeat` | le second joueur, seul debout après la chute du premier, meurt : `Defeat` | 2 |
| `wisp_solo_unchanged` | un seul joueur à mode feu follet : la mort reste la mort (`PlayerDead`, `Defeat`, HUD vide) | 1 |
| `wisp_plain_ignored`, `wisp_attract_pulls`, `wisp_attract_out_of_range`, `wisp_mislead_diverts`, `wisp_mislead_still` | un suiveur : sans leurre il va au joueur debout ; `Attract` le tire vers la lumière (la plus proche gagne) ; la **portée est une donnée** (40 unités : ignoré) ; `Mislead` le détourne même plus près du joueur debout | 2 |

Traces : **huit neuves**, aucune autre. 1837 : `docs/preuves/m-5/releve.ron` et `reprise.ron` (le vrai contenu : les salves des fantassins font tomber le Frère à 40 PV ; le partenaire porte une armure d'essai) rejoués
**sans échec** par le runner de l'engine (`PlayerDowned`, `HudText wisp`, `PlayerRevived`, santé rendue 12 PV et 20 PV).

### Traces existantes et ordre de la simulation

- **Aucune trace existante ne bouge** : la suite complète (tous les scénarios de `tests/scenarios/**`, quatre tranches `ALACOD_SCENARIO_SLICE=k/4`, binaire `scenarios` du profil `headless` sans `BLESS`) est verte sur le code
  `cce86ac0`. Les `downed_*`, `four_players_*`, `two_players_*` y sont.
- **`make schedule_order_diff`** : fait **à la main avec deux builds**, base `0b484d15` et tête, dumps `schedule_order` (`--no-default-features`, comme `test_crates`) : le script du dépôt n'a pas été utilisé tel quel (il compile
  la base dans un target à part, 20 Go de plus alors que le disque du studio était à 32 Go libres ; et un target **partagé** entre base et tête fait passer la base compilée pour la tête, D133, que j'ai rencontré et
  corrigé en nettoyant les crates du workspace). Sorties : `~/orca/runs/alpha/logs/m-5-coeur-a-deux-schedule-order/{base,head}`. Les fichiers diffèrent par du **renumérotage** (le tri topologique de bevy ne
  suit pas l'ordre d'insertion, `CLAUDE.md`), comparés **sans les numéros** : les types rollback gagnent exactement `Wisp`, `RespawnPoint` (avant les cinq de clôture), les systèmes gagnent `rollback_wisp_respawn`, un
  `apply_deferred` et la plomberie d'instantané des deux types ; rien d'autre, pour les cinq jeux.

### Captures (deux joueurs, hors écran, `play_scenario --capture --follow 0`)

`docs/taches/rapports/m-5-coeur-a-deux/` (testbed : `revive-f125-feu-follet`, `revive-f170-releve`, `revive-f299-releve-fini`, `respawn-f140-marche`, `respawn-f200-reprise`, `respawn-f319-releve`) et, dans 1837,
`docs/preuves/m-5/` (`releve-f270-feu-follet`, `releve-f350-relevee-a-30`, `releve-f520-debout`, `reprise-f330-marche`, `reprise-f430-au-feu`, `reprise-f560-debout`). On y voit la lumière chaude qui pulse, l'arc bleu de la relève,
l'anneau et l'arc doré de la reprise, « FEU FOLLET / Relève 30 % » et « FEU FOLLET / Reprise 72 % », la barre de vie (`0/40`, puis `12/40`) et les munitions (`1 | 12`). Le corps du Frère reste dessiné sous la lumière (D204).

## Revue du coordinateur (b2d74aa1) : corrigée

- **P1** (le feu follet coupait le passage du champ des joueurs debout) : le champ des sources debout est calculé **seul**, puis la zone de chaque feu follet (Dijkstra borné à sa portée) est superposée ; une case lui
  revient seulement si son coût depuis lui (décalage `Mislead` compris) est plus bas. Test `wisp_reach_tests::un_feu_follet_ne_coupe_pas_le_passage_vers_le_joueur_debout` (couloir de trois cases, un couloir d'une case est
  infranchissable pour un corps de 20 px) : **échoue sur l'ancien algorithme** (« case 2 sans direction », vérifié en rejouant le test sur `a7c19e14`), passe maintenant. Un scénario équivalent n'a pas été retenu : une carte à
  cloison jouée en scénario menait au même point d'arrivée (le feu follet bloque physiquement le passage) avec l'ancien et le nouveau code, donc ne prouvait rien.
- **P2** (sortie anticipée sans comparer la portée) : `FlowFieldCache::target_reach`, comparé, haché seulement s'il n'est pas vide (`TargetReach`, test de hash vide) ; scénario `wisp_mislead_still` (personne ne change de case :
  **échoue sans le correctif**, vu avant correction, passe après). Les traces `wisp_attract_*`/`wisp_mislead_diverts` (neuves) ont été rebénies ; aucune trace existante ne bouge.
- **P3** : sans objet, l'entité LDtk et son tri n'existent plus (D201).
- Le diff de l'ordre est refait **contre `origin/main` `30f9d1e6`** (dumps à cibles nettoyées entre la base et la tête, `.../m-5-coeur-a-deux-schedule-order/{base,head}`) : mêmes ajouts qu'avant (types `Wisp`, `RespawnPoint` à la place 61-62, avant les cinq de clôture ; système
  `rollback_wisp_respawn` ; un `apply_deferred` ; plomberie d'instantané), rien d'autre. Suite complète (quatre tranches) et `make check_rapide` rejoués sur `cce86ac0` : verts.

## Écarts et décisions à connaître

1. **HUD : une source nouvelle `wisp`, pas la source `downed` étendue** (le plan acceptait « `downed` étendue »). `downed` n'est calculée que dans le rendu (`ui::hud`), jamais dans `HudSnapshot` : une attente
   `HudText` ne peut pas la lire en headless. `wisp` vit dans `hud_model` (lisible par `HudText`, preuve par les scénarios) ; `downed` ne montre plus de compte à rebours pour un feu follet sans saignement (`bleedout_at_frame` vaut `u32::MAX`).
2. **Le point de reprise est un personnage** (voir plus haut, D201), pas une entité de carte ni un placement ; la ligne du lot « point de reprise (entité de carte ou de contenu) » est donc tenue par le contenu.
3. **Mesure faite à la main, pas par le script** : `schedule_order_diff` (voir plus haut).
4. **Le tir ami du partenaire ne blesse pas** dans 1837 (`friendly_fire` par défaut `Never`) : le protocole du soir fait tomber le Frère par les fantassins ; les scénarios de l'engine forcent `Always` sur l'arme.
5. **Tous les nombres de 1837 sont PROVISOIRES** (`game-feel.md` n'en donne aucun) : relève 3 s, leurre `Mislead` à 10 m de chemin, 2 s au feu, 30 % de vie rendue à la relève par le coéquipier, 50 % à la reprise ; marqués `PROVISOIRE` dans les RON.

## Dettes ouvertes (`docs/taches/dettes.md`)

- **D201** : le point de reprise est un personnage, pas une entité de carte (collider, santé et `GgrsNetId` de personnage ; entité dédiée = une dizaine de fichiers de génération).
- **D202** : un ennemi se fige près de **tout** joueur, y compris à terre (`move_enemies` : `distance_to_nearest_player` sans filtre `Downed`) ; trouvé en écrivant `wisp_plain_ignored`, hors lot (bougerait les traces `Downed`).
- **D203** : la portée du leurre est un coût de chemin du champ partagé (reconstruit toutes les 30 frames : retard jusqu'à 0,5 s) ; le champ « legacy » ignore les feux follets.
- **D204** : présentation minimale (gizmos, le corps reste sous la lumière, pas de son) ; pas de relève immédiate par un objet (« goutte d'eau de Pâques »).

## Dernières lignes de `make check_rapide` (`~/orca/runs/alpha/logs/m-5-coeur-a-deux-check-rapide-cce86ac0.log`)

```
games/zombies : aucune erreur (6 personnages, 4 armes, 6 armes de corps à corps, 1 vagues, 6 cartes)
games/testbed : aucune erreur (50 personnages, 27 armes, 9 armes de corps à corps, 0 vagues, 46 cartes)
games/throne : aucune erreur (14 personnages, 13 armes, 4 armes de corps à corps, 0 vagues, 2 cartes)
games/gungeon : aucune erreur (11 personnages, 21 armes, 1 armes de corps à corps, 0 vagues, 5 cartes)
games/isaac : aucune erreur (24 personnages, 3 armes, 2 armes de corps à corps, 0 vagues, 3 cartes)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
✓ Contrôle rapide passé (suite de scénarios : la nuit)
```

## Ce qui reste

- **Coordinateur 1837** : réépingler le `rev` de l'engine sur le `main` qui contient M-5 (le `Cargo.toml` de 1837 pointe `e6b5ce68`, qui ignore `wisp` et `respawn_range` en silence : le jeu ne montrerait rien).
- **Non vérifié** : la partie à deux **en ligne** et dans **Chrome** (VPN d'entreprise du Mac, comme M-2) ; l'effet du leurre sur les **fantassins postés** (le banc de l'engine le prouve avec un suiveur mobile) ; le rendu à l'œil.
- **Le soir** (`1837/docs/protocole-soir-m-5.md`, dix lignes) : régler les quatre nombres provisoires.
- Dettes D201–D204 ci-dessus.
