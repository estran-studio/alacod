# T-3b — Suites d'intégration : jeu court étiqueté, attentes fines du modèle, gestes depuis l'entrée réelle

Jalon servi : **S2, dim 22 nov** (plan `docs/taches/plan-1837-cli.md` § 6, lignes « T-3 » et « Changement d'ordre du 16 nov »). Branche `wquintal/t-3b-suites-integration`, partie de
`main` `86af8f61` (P-1d mergé). Conventions : § 123 de `docs/conventions.md` ; dette D219. Journaux : `/home/debian/orca/runs/alpha/logs/` (VPS).

Le composant de P-1d (`game::model3d::playback::ModelPlayback`, `decide_poses`, l'invariant `animation_vivante`, le lint `MissingPose`) est **réutilisé tel quel** : aucune ligne de
`crates/game/src/model3d/**` n'a changé (les régressions provoquées y sont temporaires et défaites, § 3). Aucun type rollback, aucun système de `GgrsSchedule` touché ; aucune trace existante
modifiée (les seules traces nouvelles sont celles des trois scénarios `tireur_pantin_*`).

## 1. Ce qui est livré, point par point

| Point (spec T-3b) | Issue | Preuve |
|---|---|---|
| 3. Jeu court étiqueté, budget écrit | fait | `Scenario::tags` (`tags: ["rapide"]`), `ALACOD_SCENARIO_TAG`, budget `ALACOD_SUITE_MAX_FRAMES` (somme des `frames`, en temps de jeu, déterministe) dans `crates/scenario/tests/scenarios.rs` ; `make check_integration` (`SUITE_MAX_FRAMES=4800`, 80 s de jeu) remplace `check_animation` dans `check_rapide` ; mesuré : 13 scénarios, 2 985 frames (50 s de jeu), 28,9 s d'horloge (journal du contrôle, § 6) |
| 3. Même option dans `alacod test` | fait | `alacod test [jeu] --tag rapide` → `alacod-gen --tag rapide` : seulement les `tests/scenarios/*.ron` étiquetés du jeu, sans génération, traces exigées (aucun étiqueté : `alacod-gen` sort en code 2 avec un message, chemin non joué). Preuve sur une copie jetable de 1837 : § 5 |
| 3. La nuit | fait | la suite de nuit joue tous les scénarios, étiquetés compris ; le binaire `gestes_entree_reelle` est dans `test_crates` |
| 1. Entrées réelles par geste | fait | `crates/scenario/tests/gestes_entree_reelle.rs` : 4 tests (pantin au pistolet, pantin à l'arme à gestes) × (souris + clavier, visée par `CursorMoved` ; manette, visée par l'axe du stick droit). Chemin : leafwing (`InputManagerPlugin<PlayerAction>`, `get_input_map`), `collect_device_inputs`, `read_local_inputs`, `record_local_inputs`, `InputRecorder::to_scenario`, rejoué par le runner. Gestes : visée (`ModelYaw`), tir (`Clip shoot`), épaulé (`Clip shoulder` + `ClipSpeed`), rechargement (`Clip reload` + `ClipSpeed`/`ClipProgress`), coup de mêlée (`Clip strike`), pas (`Clip roll`). 4/4 verts ; joués par `check_integration` et `test_crates` |
| 2. Mécaniques entre personnages | fait | existant réutilisé et étiqueté `rapide` : `tireur_poste_cycle` (cycle, salve, recul), `tireur_poste_touche` (coup reçu → salve perdue), `tireur_poste_suit` (visée qui suit), `renvoi_secteur` (balle renvoyée par le coup), `wisp_revive_by_mate` (relève à deux), `wisp_mislead_diverts` (leurre), `key_carrier_coop` (clé d'équipe), `repli_couvert_egalite` (couvert) ; ajouté : `TelegraphAim` (le cône suit) et les scénarios `tireur_pantin_cycle`, `tireur_pantin_suit`, `tireur_pantin_touche` (cycle du tireur face au joueur clip par clip, cône et corps qui suivent, coup reçu joué et salve perdue) |
| 4. Attentes fines sur `ModelPlayback` | fait | `Clip`, `ClipProgress`, `ClipSpeed`, `ModelYaw` (`crates/combat/src/weapons/expectations.rs`, évaluées par `crates/scenario/src/model_expect.rs`) ; une entrée par ligne de la revue observée de P-1d : § 4 |
| Chaque attente nouvelle rouge sur régression provoquée | fait | § 3 |
| `make check_sim_boundary`, contrôle rapide découpé | fait, vert | § 6 |

## 2. Testbed ajouté (contenu neutre)

- `characters/tireur_pantin.ron` : le `tireur_poste` (mêmes réglages de simulation) dessiné par le modèle `tireur_pantin` de `models/models.ron` (même `.glb` que le pantin ; poses de l'arme
  `fireball_gun` de sa règle `Shoot` et `hit`). Un personnage a pour id son `asset_name_ref` (lint `DuplicateId`) : d'où une entrée de modèle à part plutôt que `pantin`.
- `scripts/gen-pantin.py` : clip `hit` (0,5 s, buste rejeté en arrière) ; `anime.glb`/`anime.json` régénérés (le manifeste ne gagne que `hit` ; durées des autres clips inchangées).
- `models.ron` : poses de `arme_a_gestes` sur le pantin (les mêmes clips que le pistolet) pour l'épaulé des tests d'entrée réelle.

## 3. Rouge sur régression provoquée, vert après

Deux constructions rouges sur `d8d04277`, régressions temporaires défaites par `git checkout` (aucune n'est commitée) ; vert : `check_integration` et `gestes_entree_reelle` sur la même tête.

| Régression provoquée (fichier) | Attente qui échoue (message du runner) | Journal |
|---|---|---|
| cap du corps jamais mis à jour (`playback.rs`, `heading…filter(\|_\| false)`) | `ModelYaw` : `tireur_pantin_cycle` f45 et f148 « cap du corps 0.0°, 141.1° attendu » ; `tireur_pantin_suit` f35, f58 ; gestes : f40 (gauche), f184 (haut), f28 (droite) | `t-3b-rouge-r1.log` |
| visée de l'émetteur figée au départ de la salve (`cycle.rs`, `track_target && false`) | `TelegraphAim` : `tireur_pantin_suit` f35 « le cône vise à 7.1° du joueur 0 », f45 23.6°, f55 36.9°, f61 38.0° | `t-3b-rouge-r1.log` |
| geste joué une fois au rythme du manifeste, sans étirement (`playback.rs`, `Timing::At` → `time + dt`) | `ClipSpeed` : `tireur_pantin_cycle` f100 « vitesse de `reload` 1.000 < min 1.15 » ; gestes f90 (pistolet) ; `ClipProgress` f140 « progression de `reload` 0.806 < min 0.9 » | `t-3b-rouge-r1.log` |
| dernière image du tir non tenue (`playback.rs`, `since_shot` non borné) | `ClipProgress` f81 « progression de `gun-idle` 0.961 < min 0.99 » ; `ClipSpeed` f81 « vitesse de `gun-idle` 1.000 > max 0.01 » (et l'invariant `animation_vivante`) | `t-3b-rouge-r1.log` |
| bouton de mêlée perdu (`input_devices.rs`) | `Clip` gestes f148 « clip joué `gun-idle`, `strike` attendu » (souris et manette) | `t-3b-rouge-r1.log` |
| coup reçu non lu par la pose (`playback.rs`, `input.hit` effacé) | `Clip` `tireur_pantin_touche` f50 « clip joué `gun-idle`, `hit` attendu » ; `ClipProgress` f71. Preuve de la **présentation** du coup reçu seulement : l'annulation de la salve en simulation est gardée par `BulletCount` (attente existante de `tireur_poste_touche`, recopiée), pas par une attente nouvelle, et n'a pas été mutée ici | `t-3b-rouge-r2.log` |
| épaulé lu sur la seule détente (`playback.rs`, sans `raise_ends`, le défaut D199) | `Clip` gestes à l'arme à gestes f40 « clip joué `gun-idle`, `shoulder` attendu » (souris et manette) | `t-3b-rouge-r2.log` |
| visée au curseur perdue (`input_devices.rs`, conversion écran → monde) | `ModelYaw` souris f40 « cap du corps 180.0°, -90.0° attendu » ; arme à gestes f28 | `t-3b-rouge-r2.log` |
| visée au stick droit perdue (`input_devices.rs`, stick ignoré) | `ModelYaw` manette f40, arme à gestes f28 | `t-3b-rouge-r2.log` |

## 4. La revue observée de P-1d, ligne par ligne

| Ligne de la revue (P-1d § 4) | Attente (scénario ou test, frame) |
|---|---|
| Fantassin, attente (`fusil-idle` en boucle) | `tireur_pantin_cycle` : `Clip gun-idle once:false` + `ClipSpeed` 0,99–1,01 (f15) |
| Fantassin, épauler (une fois) | `Clip shoulder once:true`, `ClipProgress` 0,4–0,7 (f26) ; seconde salve f158 |
| Fantassin, en joue (boucle, cône vers le joueur) | `Clip aim once:false`, `TelegraphAim` ≤ 1°, `ModelYaw` vers le joueur (f45) ; `tireur_pantin_suit` (le joueur bouge : f35-f61) |
| Fantassin, tir et recul (une fois, tenu) | `Clip shoot once:true`, `ClipProgress` 0,3–0,6 (f70) ; tenu : `ClipProgress` ≥ 0,99 et `ClipSpeed` ≤ 0,01 (f81) ; recul : `EnemyDistance` de `tireur_poste_cycle` |
| Fantassin, rechargement (une fois, étiré) | `Clip reload once:true`, `ClipSpeed` 1,15–1,25 (clip de 1,2 s sur 1 s, f100), `ClipProgress` ≥ 0,9 (f140) |
| Fantassin, retour au poste (allure, face au joueur) | `Clip gun-walk once:false`, `ModelYaw` vers le joueur (f148) |
| Fantassin, touché (non observé au jeu par P-1d) | `tireur_pantin_touche` : `Clip hit once:true` (f50), `ClipProgress` ≥ 0,85 (f71), salve perdue (`BulletCount` 0, f65), cycle qui reprend (f89, f110, f133) |
| Fantassin, mort | sans objet au banc : aucune attente (D219) |
| Frère, attente / marche | gestes : la marche vers le haut avant le pas ; `model_pantin` sous `animation_vivante` |
| Frère, épauler (étiré sur le geste d'arme) | gestes à l'arme à gestes : `Clip shoulder` + `ClipSpeed` 0,95–1,05 (9 frames de `raise_frames` sur le clip de 0,15 s, f40) |
| Frère, tir | gestes : `Clip shoot once:true` (pistolet f52, arme à gestes f52) ; l'écart (a) de P-1d (durée du clip au lieu du geste d'arme) n'est **pas** épinglé (D219) |
| Frère, rechargement | gestes : `Clip reload` + `ClipSpeed` 1,15–1,25 (pistolet : 1 s pour le clip de 1,2 s, f90) ; arme à gestes `ClipSpeed` 0,95–1,05 (f90), `ClipProgress` ≥ 0,9 (f133) |
| Frère, coup de hache | gestes : `Clip strike once:true` (f148) ; fenêtre de frappe : `frappe_moment` |
| Frère, gigue | gestes : `Clip roll once:true` (f192) ; l'écart (b) (premier pas sur le deuxième clip) n'est **pas** épinglé (D219) |
| Frère, visée (le corps regarde la visée) | gestes : `ModelYaw` gauche (f40), haut (f184), droite (f28), depuis le curseur et depuis le stick |

## 5. `alacod test --tag rapide` sur 1837

Copie jetable détachée de 1837 `origin/main` `3940825` (`/home/debian/orca/workspaces/1837/t-3b-jetable`, retirée ensuite ; le checkout du coordinateur 1837 n'est pas touché), cinq de ses scénarios
étiquetés `tags: ["rapide"]` **dans la copie seulement**, engine `05994d06` (`--engine`) : `logs/t-3b-alacod-test-tag-05994d06.log`.

```
ok     scenarios/exterieur_couvert  300 frames, 0 coups, attentes ok / 8, trace ok
ok     scenarios/fantassin_cycle    480 frames, 0 coups, attentes ok / 11, trace ok
ok     scenarios/geste_hache_renvoie 200 frames, 0 coups, attentes ok / 4, trace ok
ok     scenarios/geste_tir          150 frames, 0 coups, attentes ok / 13, trace ok
ok     scenarios/m5_releve          600 frames, 0 coups, attentes ok / 6, trace ok
5 scénario(s) joué(s), 5 trace(s) comparée(s), 0 échec(s)
```

`--list --tag rapide` liste les cinq. **À faire par le coordinateur 1837** (contenu de 1837, pas dans l'engine) : ajouter `tags: ["rapide"],` à ces cinq scénarios (1 730 frames, 29 s de jeu :
couvert, cycle du fantassin, renvoi à la hache, tir du Frère, relève) — `exterieur_cle` (1 500 frames) reste hors du jeu court — et jouer `alacod test --tag rapide` avant chaque merge de 1837.

## 6. Contrôle rapide (forme découpée, décision du coordinateur du 20 nov)

`05994d06`, journal `logs/t-3b-suites-integration-check-rapide-05994d06.log` (scripts `logs/t-3b-check-rapide-decoupe.sh` et `logs/t-3b-test-crates-decoupe.sh`, cible partagée
`target-partage`) : `make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_integration` code 0 (471 s ; jeu court : 13 scénarios, 2 985 frames,
50 s de jeu, 28,9 s d'horloge ; `gestes_entree_reelle` 4/4) ; `test_crates` découpé, mêmes options : `--lib --bins` 1 166 passed, `--doc` 0, puis les 56 tests d'intégration de
`cargo metadata` un par un (binaire supprimé après chacun) : 57/56 joués (lib/bins/doc compris), 0 échec, aucun code non nul ; 1 675 s ; total 2 147 s.

Traces : aucune trace existante modifiée (le jeu court rejoue les traces des dix scénarios étiquetés, identiques) ; `schedule_order_diff` non joué : aucun système de simulation ajouté ni déplacé, aucun type
rollback (le runner et les tests seuls ont changé côté code).

## 7. Revue de 071ce9a4 (coordinateur, `logs/t-3b-revue-071ce9a4.md`, et revue interne) : corrections

| Point | Correction | Rouge avant / vert après |
|---|---|---|
| P1 (bloquant) : `alacod test --tag` sautait la génération mais purgeait `tests/generated` (`remove_stale` avec `keep` vide) | `alacod-gen` : la purge exige une génération réellement faite (`!only_written`) | binaire de `05994d06` sur une copie jetable de 1837 : « suppression de …/weapon_temoin.ron », binaire corrigé : le témoin survit (`logs/t-3b-revue-p1-remove-stale.log`) ; test permanent `test_tag_plays_only_tagged_written_scenarios_and_keeps_generated_ones` (`crates/content/tests/cli.rs`, ignoré, dans `make test_alacod_external`) : rouge garde défaite (`logs/t-3b-revue-corrections-rouge.log`, témoin `NotFound`), vert après (`logs/t-3b-revue-corrections.log`, 1 passed). Le checkout du coordinateur 1837 n'a jamais eu de `tests/generated` et reste propre (`git status` vide) ; seule la copie jetable a perdu le témoin posé pour la preuve |
| P2 : `InputRecorder::to_scenario` remettait `tags` à vide | `RecordedSettings::tags`, copié par `from_scenario`, reporté par `to_scenario` (vide pour une partie jouée) | `recording::tests::to_scenario_reporte_les_reglages_recus_a_la_construction` (assertion `tags == ["rapide"]`) : rouge sans le report (`left: []`), vert après |
| Revue interne : une tranche (`ALACOD_SCENARIO_SHARD`/`_SLICE`) sans scénario étiqueté échouait « aucun scénario étiqueté » | l'échec ne vaut plus que sans tranche | correction de lecture, non rejouée en rouge |
| Mineur 3 : R2 ne mute que la présentation du coup reçu | dit au § 3 ; mutation dédiée boucle ↔ une fois ajoutée (R3 : `looping` inversé dans `step`) | R3 : chaque `Clip … once:` des trois `tireur_pantin_*` échoue (« `shoulder` boucle, joué une fois attendu », « `aim` joué une fois, une boucle attendue »…, `logs/t-3b-revue-corrections.log`) ; vert après |

Fichiers changés par ces corrections : `alacod-gen.rs`, `recording.rs`, `scenarios.rs` (test), `cli.rs` (test), `Makefile` (`test_alacod_external` joue aussi le test `--tag`), rapport. Suivant la consigne du
coordinateur : tests ciblés + `check_integration`, pas de contrôle découpé complet (aucun fichier de simulation ni d'application hors de ces crates/tests).

## 8. Ce qui reste

- D219 : harnais qui remplace winit pour le curseur, gestes joués sur le pantin seulement, scénarios de 1837 à étiqueter par le coordinateur 1837 (§ 5), écarts (a) et (b) du Frère non épinglés,
  mort d'un personnage à modèle sans attente.
