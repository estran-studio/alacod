# Rapport m3-t11c-ennemis (b1)

Conventions : `docs/conventions.md` §76.3. Branche `m3-t11c-ennemis` depuis `origin/main` `bfc71a02`, fusionnée avec `origin/main` `a2bf564a` (M3-T11b). Données seulement : aucun code moteur.

## 1. Fait
- Cinq ennemis nouveaux (`rampant` charge, `spectre` `Phasing` (ignore fenêtres et fosses, pas les murs : comportement de `Chase` Ground dans isaac), `cracheur` tireur, `tourelle_croix` immobile, `bourdon` volant tireur) en plus de `suiveur` et `mouche` : sept en tout, un `test:` chacun (10 scénarios `enemy_*` générés).
- `sight_range` posé sur les trois tireurs (`aggro_range` bas, vue longue : cible seulement en ligne de vue) ; aucun sur le `spectre` (`aggro_range` 500 couvre la salle ; il ne traverse pas les murs : `Phasing` ignore seulement les obstacles non `Wall`, dette).
- Trois ennemis à champions : `suiveur`/`explosif` (`Explode`), `mouche`/`essaim` (`SpawnCharacters`), `tourelle_croix`/`salve` (`SpawnPattern`), `chance` en expression par étage, `tint` ; `champions: [(…, kill: true, weapon: Some("larmes"))]` : 3 scénarios `champion_*` (la sonde de mort passe : explosion, deux mouches, couronne).
- Boss `gardien` : trois phases, renforts `rampant`+`cracheur` (phase 1) et `spectre`+`bourdon` (phase 2), `spirale` (patterns v2), `boss_drop` du pool `boss` inchangé.
- Placement : `Combat` + `tourelle_croix` ; le reste par le boss (`docs/taches/rapports/m3-t11c-ennemis.make_placement.py`).
- Lint isaac : « aucun ennemi … `variants` » disparaît (30 → 25 avertissements, restent T11d/T14).

## 1 bis. Écart de périmètre assumé
La fiche demandait 10 ennemis et 2 boss ; livré 7 ennemis et 1 boss. Le reste en T11d (gabarits de combat variés, salle `arene`). Mesure indicative à 1 bot : 60 % de morts, 34 % de victoires, sous la cible Q14 : à rééquilibrer en T11d. `crachat` passe à 1 PV (isaac : 1 PV = demi-cœur, le HUD tronque une fraction). Dette moteur D88 (tirage des variantes) écrite dans `dettes.md`.

## 2. Équilibrage : ce qui a été essayé
Un seul gabarit `Combat` (aucun gabarit ajouté : T11d), donc le même lot d'ennemis dans toute salle de combat. Campagnes de 10 à 16 graines (`explorateur`, `--floors sous_sol --until-floor 2`) :
| Configuration | Résultat |
|---|---|
| `main` + variantes seules | 10 victoires, 4 morts, 2 soft-locks (16) |
| + 5 ennemis en combat, premier boss | 50 graines : 50 morts, 0 kill |
| + `rampant` seul / `cracheur` seul / `tourelle` seule (boss dur) | 11 morts, 15 morts, 8 morts sur 16 |
| + 5 ennemis en combat, boss doux | 16 morts sur 16 |
| + `tourelle` en combat, renforts du boss, boss doux | 9 morts, 6 victoires, 1 soft-lock sur 16 |
L'`explorateur` ne survit pas à quatre ennemis simultanés (cœurs et sous : T11d) : d'où trois ennemis par salle et les renforts au boss.

## 3. Campagne de santé (final, 50 graines)
`alacod-sim --game isaac --bots 1 --profiles explorateur --seeds 1..50 --floors sous_sol --until-floor 2 --max-frames 12000` (880 s) :
**30 morts (60 %), 17 victoires (34 %), 3 soft-locks (9, 42, 43), 0 desync.** Graines des victoires : 2 3 4 6 7 8 14 17 19 21 22 23 26 31 34 44 48.
Soft-locks : 9 et 43 existent à l'identique sur `main` (vérifié, même contenu isaac de `origin/main`), 42 est de la même famille (plafond « phase NotStarted », ennemis hors de vue, `damage_taken: 0`, aucune salle de combat en cours) : navigation de l'`explorateur`, hors périmètre (T14). Indicatif : T14 fera 200 graines.

## 4. Graines (atteignabilité)
- Combat (`suiveur`, `mouche`, `tourelle_croix`) et boss (`rampant`, `cracheur` phase 1 ; `spectre`, `bourdon` phase 2) : graine **2** (victoire ; lignes `boss_reinforcements … id=<ennemi>` dans `--log`).
- Variantes : **le tirage ne donne aucun champion pour les graines 1 à 1279** (premier tirage du congruentiel ≥ 0,277 pour les net ids 100 à 199 ; chances d'isaac ≤ 0,27 aux deux premiers étages ; 1..50 et 1000..1150 essayées : zéro champion), donc jamais dans une campagne `--seeds 1..200`. Graine **1730** : `--seeds 1730..1730 --log` journalise 4 `explosif`, 4 `essaim`, 3 `salve`, 18 `champion_split`, 18 `explosion` (relevé par `map_probe` à f5 : 2 `explosif`, 2 `essaim`, au moins un `salve` à l'étage 0). Dette de tirage (§72, non corrigé : déplacerait les traces) au conventions §76.3.

## 5. Traces bougées (preuve §5)
21 traces isaac non générées et 3 générées (`boss_gardien`, `room_combat`, `weapon_arsenal`) (`curse_*` ×4, `devil_*` ×3, `grid_*` ×9, `hit_iframes_isaac`, `isaac_boots`, `isaac_floor_seed_1`, `isaac_reliquaire`, `isaac_synergy_garde`) + générées bougées : `boss_gardien`, `room_combat`, `weapon_arsenal` ; nouvelles : `champion_*` ×3 et `enemy_*` des cinq ennemis nouveaux (`enemy_suiveur_*` et `enemy_mouche_*` n'ont pas bougé : arène du gabarit d'armes, sans `Combat`). Dumps `ALACOD_DUMP_TRACE` + `scripts/trace-diff.py`, `main` (`a2bf564a`) contre branche, sur `isaac_boots` et `grid_treasure_key` : première frame qui diffère = 0 ; `FloorState.enemies_placed` 11 → 16 et 7 → 10 (une `tourelle_croix` de plus par salle de combat), les `GgrsNetId` suivants sont décalés, aucune autre différence structurelle. Cause : le placement de `tourelle_croix` dans `Combat` pour les traces de la grille ; `weapon_arsenal` (gabarit d'armes, sans `Combat`) bouge dès f0 par la table de projectiles de l'`arsenal` (entrée `crachat` ajoutée, `weapons/weapons.ron`), hors de la preuve §5 par dump (non faite sur cette trace). Cinq scénarios ont leurs attentes recalées (`grid_floor_seed_1`, `grid_floor_seed_123`, `grid_floor_seed_23`, `grid_post_boss`, `isaac_floor_seed_1` : 16 ou 13 ennemis, boss verrouillé vers f2021 au lieu de f1340 pour la graine 1, le bot meurt au second étage vers f3039). Aucune trace d'un autre jeu.

## 6. Vérifié (sur la fusion avec `origin/main` `a2bf564a`)
- `make gen GAME=isaac` : toutes les attentes vertes, y compris `champion_*` (`kill: true`) et `boss_gardien` (phase 0 → 1, 1 → 2, salle nettoyée).
- `make test_scenarios_par SHARDS=3` **sans BLESS** (après bénédiction des 21 traces isaac et recalage des 5 scénarios) : 3 tranches ok, 217 s.
- `make test_crates` : 1 167 passés, 0 échec.
- `make lint` ×5 : aucune erreur ; isaac 25 avertissements (30 → 25 avec T11b, « aucun ennemi à variants » disparu), gungeon 1, testbed 107 (inchangés).
- `make gen` ×5 : code 0, `git status` propre ensuite. `cargo fmt --all -- --check` : ok.
- p2p `scripts/p2p-stress.sh isaac tests/scenarios/p2p_smoke.ron 3000` (MATCHBOX_PORT=3556, projet b1) : **2 177 frames resimulées** (client 0), 0 desync, 2 999 lignes identiques.
- Bench non lancé (machine partagée).

## 7. Dettes
- `Phasing` n'ignore que les obstacles non `Wall` (fenêtres, barricades, eau, fosses) : isaac n'en pose pas, le `spectre` suit donc le champ de flux comme un `Chase` Ground.
- Un seul gabarit `Combat` : trois ennemis fixes par salle, pas de variété par salle (T11d) ; pas de salle à vagues (`arene`) dans `isaac`.
- Tirage des variantes groupé pour les petites graines (aucun champion pour 1..1279) : T14 doit prendre des graines ≥ 1700 (ex. 1730) ou faire corriger le tirage (déplace les traces de tout contenu à variantes).
- Soft-locks de l'`explorateur` (graines 9, 42, 43) : navigation, T14.
- L'`explorateur` ne survit pas à quatre ennemis simultanés (60 % de morts avec trois) : cœurs, sous et récompenses sont T11d ; chiffres indicatifs.
- Revue indépendante : voir §8.

## 8. Relecture indépendante (agent `reviewer`)
Aucun P1. P2 corrigés : (1) bande du tirage des variantes (graines 1..1279 sans champion, pas 1..2047 ; graine d'atteignabilité 1730 au lieu de 64381, vérifiée en `alacod-sim --log`) ; (2) `Phasing` ne traverse ni murs ni rochers (commentaires, §76.3, rapport) ; (3) `weapon_arsenal.trace` (cause : entrée `crachat` de l'`arsenal`) ajoutée au rapport, liste des traces corrigée. Vérifié par le relecteur : script de placement identique à l'octet à `salles.ldtk`, chances ≤ 1 à l'étage 5, pas de cascade `SpawnCharacters`, `couronne_unique` à `every: 0`.

## 9. Revue 2 : fusion avec `origin/main` `2d30a1c1` (T16 HUD isaac) et corrections
- Conflit `docs/taches.md` (deux lignes de journal) résolu en gardant les deux.
- `crachat` 0,5 → 1 PV (1 PV = demi-cœur d'isaac, le HUD tronque une fraction). Tout le jeu `isaac` rebéni dès f0 (la table de projectiles de l'`arsenal` est hachée) : 22 traces non générées (dont `isaac_hud_items` et `isaac_banner_blind` de T16) et les générées `boss_gardien`, `room_combat`, `weapon_arsenal`, `enemy_cracheur_*`, `enemy_bourdon_*`, `enemy_tourelle_croix_*`, `champion_tourelle_croix_salve`. Aucune attente à recaler : le HUD ne change pas.
- Preuve §5 pour `isaac_hud_items` et `isaac_banner_blind` (dumps `main` `2d30a1c1` contre branche) : première frame qui diffère = 0, `FloorState.enemies_placed` 11 → 16, net ids décalés ; même cause que les autres (une `tourelle_croix` par salle de combat).
- D88 écrite dans `docs/taches/dettes.md` (tirage des variantes) ; §76.3 corrigé : la graine par défaut `123456` est gardée par le restart, **aucun champion au premier étage avec elle** ; atteignable par graine choisie (1730).
- Écart de périmètre : 7 ennemis et 1 boss livrés sur 10 et 2 (T11d : gabarits variés, `arene`) ; 60 % de morts / 34 % de victoires à 1 bot, sous la cible Q14.
