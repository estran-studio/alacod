# M1 `throne` — digest de fin de vague 2

Digest final de la vague 2 de M1 (rédaction : m1-integration-scenarios, puis m1-cloture-videos-digest,
agent local b0). Chiffres du journal (`docs/taches.md` §9 et §10) sauf mention contraire ; aucun nouveau
calcul dans cette clôture.

## En bref

- Le clone joue la run de `throne` : trois cavernes générées par graine (trois cavernes **différentes**
  depuis « une caverne = un asset »), portail à l'étage vidé, boss `roi_rat` au troisième étage, rads,
  niveaux et mutations, horloge d'étage, butin par munition, HUD, écran de mutation, restart en ligne.
- **Critère des bots atteint** (`main` `ed8a274`, 2026-10-06) : **198/200 à 2 bots, 0 soft-lock,
  0 desync** (2 défaites au troisième étage), **200/200 à 4 bots**. La veille, sur `61ac539` :
  149/200, 12 soft-locks, 39 défaites. Entre les deux : D48 (ennemis et portail hors champ), D51
  (dispersion des tirs ignorée, bug moteur), les correctifs de bots (soft-locks, armes, réanimation).
- **Revers** : depuis D51, le troisième étage est devenu facile pour les bots (0 défaite sur 50 graines
  à 2 bots, puis 2 sur 200) : **rééquilibrage à décider avec William**.
- **Bench** : `bench_horde` était tombé à 36 fps au calme (plancher 38) ; D53 trouve la cause (snap de
  la salle de spawn qui fait entrer le bench dans le guidage de récupération) et la corrige sans changer
  la simulation : 58,7 / 59,6 fps au calme, critère du bench atteint.
- Reste pour sortir de M1 : la revue humaine (section « Pour la revue humaine ») et les notes de M0.

## Ce que le clone sait faire

Une ligne par chantier du plan (`docs/taches.md` §6) livré pendant M1, avec sa tâche (journal).

| Chantier | Tâche(s) | Ce que ça donne |
|---|---|---|
| Contrats combat et IA | T1.0a | armes déplacées dans `combat`, enums squelettes (`ProjectileModifier`, `Pattern`, `StatusDef`, `Behavior`…), crate `behaviors` |
| Contrats monde et run, cavernes | T1.0b, T1.6 | `world::CellGrid` (rollback), générateur de cavernes par automate cellulaire et graine, terrain destructible, flow field |
| Contenu `throne` | T1.0c, T1.11, m1-throne-gen-et-d40 | `games/throne` : un pilote, douze armes sur cinq munitions, dix ennemis, mutations, trois cavernes, butin par munition (D40) |
| B5 v1 projectiles composables | T1.1 | `Bounce`, `Pierce`, `Size`, `Lifetime`, `Homing`, `Gravity`, `on_hit`, `on_expire` ; explosions comme projectile à durée nulle |
| B5 v1 patterns et tir ennemi | T1.2 | `Emitter` rollback, `Aimed`/`Spread`/`Ring`/`Sequence`/`Telegraph`, attaque `Shoot(pattern)` |
| B3 statuts | T1.3 | `Burn`, `Slow`, `Stun`, `Freeze`, empilement, visuel dérivé |
| D1 behaviors composables | T1.4 | `Chase`, `KeepDistance`, `Strafe`, `Charge`, `Shoot`, `Melee`, `Flee`, `Wander` par priorité |
| D2 variantes et élites | T1.5 | `variants` tirées par flux, tags (`champion`, `rapide`) |
| E4 v1 surfaces | T1.7 | tags de cellules et modificateurs de vitesse |
| F1 mode `Floors` | T1.8 | étages dans le `GgrsSchedule`, portail, boucle infinie au dernier niveau |
| F2 horloges et difficulté | T1.9 | `Clock` d'étage et de run, difficulté par étage et par temps |
| C1 v1 / C4 v1 effets, jauges, mutations | T1.10 | `Effect { on, if, do }`, rads → niveau → mutation |
| Lint et attentes de M1 | T1.12, T1.15 | audit du lint des nouveaux kinds, attentes `BulletCount` … `CellState` |
| Générateur v1 | T1.13, m1-throne-gen-et-d40 | gabarits par ennemi (immobile, mobile), placement scripté ; `make gen GAME=throne` vert (37 scénarios) |
| Bots v1 | T1.14, m1-v3-bots-pathfinding | `prudent` esquive et navigue par le champ, `alacod-sim --until-floor` |
| I2 feedback v1 | T1.17 | hit stop, secousse, flash, télégraphe au sol, chiffres ; `FeedbackLog` (preuve sans écran) |
| Écran de mutation | T1.16 | choix à trois cartes, input scriptable (journal) |
| HUD throne | T1.18 | rads, niveau, munitions par type, statuts, cibles du HUD lintées |
| Dettes | m1-dettes-lot-1, m1-d39, m1-d36-et-analyse-depart, m1-assembleur-d45-d47, m1-dettes-doc-lot-2, m1-d26-doublons-generes | D26, D31, D33, D35, D36, D37, D39, D45 à D47 fermées ; D40 à moitié (`grunt`) |
| Scénarios du clone, boss, une caverne = un asset | m1-integration-scenarios | `throne_solo`, `throne_duo`, `throne_quad`, boss `roi_rat` ; chaque caverne chargée depuis son propre asset (`cave://`) |
| Navigation par profil et gabarit | m1-navigation-profils-tailles, m1-d41-spawns-degages | champ de flux par profil et par gabarit (petit, grand) ; points d'apparition dégagés selon le corps en jeu (D41) |
| Diagnostic de soft-lock | m1-d42-softlock-diagnostic, m1-d43-d44-fin-de-partie | relevé D42 sur le vrai champ, `alacod-sim` s'arrête à la fin de run, défaite déclarée |
| Ennemis jamais hors champ, portail atteignable | m1-d48-ennemis-hors-champ | règles de passage partagées (`world::nav`) ; points d'ennemis et ancre de portail atteints par le champ du gabarit ; `alacod-sim --log` (D49) |
| Dispersion des tirs | m1-d51-dispersion-ignoree | **bug moteur** : `spread` n'était jamais appliqué aux tirs simples (±0,5 rad pour toutes les armes) ; ressenti de toutes les armes changé, `zombies` compris |
| Restart en ligne | m1-restart-p2p | « Rejouer » à la fin d'une partie en ligne avec les mêmes pairs (D14) |
| Bots v1 suite | m1-v3-bots-portail, m1-v3-bots-reanimation, m1-v3-bots-softlocks, m1-v3-bots-armes | portail piloté en vitesse, réanimation du coéquipier, zéro soft-lock, choix et ramassage d'armes ; anticipation de la cible mesurée mais **non livrée** (m1-v3-bots-lead) |
| Performance | m1-d53-bench-horde | guidage de récupération des zombies sans tests de collision inutiles (`bench_horde` 40 → 59 fps au calme) |

## Les chiffres

### Tests

Le journal fait foi (`docs/taches.md` §10) : à la dernière vérification sur l'état fusionné
(m1-d26-doublons-generes), 602 tests de crates, la suite des scénarios verte, lint des trois jeux,
fmt, scripts, `make gen` des trois jeux sans modification, exemples, p2p N=2 identique (`6e297852…`).

### Historique : 20 graines au moment du brouillon (m1-integration-scenarios)

```
cargo run -q -p scenario --profile headless --bin alacod-sim -- --game throne --bots N \
  --profiles prudent,… --floors run --seeds 1..20 --until-floor 3 --max-frames 15000
```

Joué sur la branche m1-integration-scenarios (état final : correctif cave://, boss 20 × 20,
butin relevé, réserves ×2, tourelle en dernier). « Fini » = troisième étage passé ; SL =
soft-lock détecté (1 200 frames sans kill ni étage). Résultats complets :
[`m1-fin-de-vague-2.sim.json`](m1-fin-de-vague-2.sim.json).

| Réglage | 1 bot | 2 bots | 4 bots |
|---|---|---|---|
| avant le correctif (journal : **trois fois `niveau_1`**) | — | 20/20 | — |
| correctif + boss 20 × 20 + butin relevé | 0/20 (7 SL, 13 morts) | 1/20 (19 SL) | 11/20 (9 SL) |
| + réserves de départ ×2 | 0/20 (10 SL, 10 morts) | 1/20 (19 SL) | 11/20 (9 SL) |
| + tourelle en dernier (**final**) | 0/20 (6 SL, 14 morts) | **3/20** (16 SL, 1 défaite) | **11/20** (9 SL) |

**Desync : 0** dans toutes les séries. Les soft-locks laissent presque toujours 1 ou 2 ennemis
sur les points les plus éloignés du départ (tourelle fixe, brute lente, tireurs) que le bot
`prudent` ne va pas chercher : limite des bots, pas des données (les réserves doublées ne
changent rien graine par graine). En solo, deux soft-locks au portail ouvert de l'étage 0
(graines 3 et 16) : défaut de bots aussi. sim fps moyen : 65 (1 bot), 52 (2), 40 (4).

### 200 graines

```
alacod-sim --game throne --bots N --profiles prudent,… --floors run --seeds 1..200 \
  --until-floor 3 --max-frames 15000 --progress --json
```

| Date, `main` | Bots | Finies (3 étages) | Desync | Soft-locks | Défaites | Frames (min / méd. / max) |
|---|---:|---:|---:|---:|---:|---|
| 2026-10-05, `61ac539` | 2 | 149/200 | 0 | 12 | 39 | 3 307 / 5 515 / 8 326 |
| 2026-10-06, `ed8a274` | 2 | **198/200** | **0** | **0** | 2 (graines 73, 100) | 2 430 / 3 361 / 4 950 |
| 2026-10-06, `ed8a274` | 4 | **200/200** | **0** | **0** | 0 | 1 745 / 2 385 / 3 298 |

- Le 2026-10-05 : les 39 défaites étaient toutes au troisième étage (boss `roi_rat`), première mise à
  terre par les tireurs ou le boss puis survivant seul (digest `m1-200-graines-throne.md`, b0) ; les 12
  soft-locks (relevé D42) ont été classés par rejeu (m1-v3-bots-softlocks, b1) : recul dans la roche
  devant un ennemi caché, à sec contre le boss ou une tourelle, portail non pris, ennemi né hors de son
  champ (D48).
- Le 2026-10-06 : 11 runs finies avec un mort relevé à 2 bots ; aucun mort à 4 bots.
- Données hors dépôt : `alacod_tasks/m1-200-throne/` et `alacod_tasks/m1-200-throne/ed8a274/`.

### Bench

Bench strict au calme (orch, 2026-10-06, `a8b813e`) : `bench_bullets` 99 à 117 fps (plancher 70),
`bench_cave` 161 à 167 (plancher 40), **`bench_horde` 36 fps, sous son plancher de 38** (67 à 81 au
calme pendant M0). Cause et correctif : m1-d53-bench-horde (bissection jusqu'à `8785220`, guidage de
récupération) ; mesure au calme le 2026-10-06 (charge 1,4 à 1,9, avant/après en alternance) :
`bench_horde` 39,8 / 40,8 → **58,7 / 59,6** fps (plancher 38), `bench_bullets` ≈ 135, `bench_cave`
≈ 195, `bots_four_mixed` ≈ 107, inchangés. **Critère du bench atteint** (journal, m1-d53-bench-horde).

## Vidéos

Rendues sur `146d09c` (`make videos SCENARIO=throne_solo,throne_three_floors,throne_quad`, puis
`make views SCENARIO=throne_quad`, 960 × 540, une image toutes les deux frames). Les vidéos de
moins de 5 Mo sont copiées dans `docs/digests/videos/` avec leurs moments clés :

| Vidéo | Durée | Taille | Source |
|---|---:|---:|---|
| [`throne_solo.mp4`](videos/throne_solo.mp4) | 61,7 s | 2,8 Mo | `target/videos/146d09c/throne_solo.mp4` |
| [`throne_three_floors.mp4`](videos/throne_three_floors.mp4) (duo) | 86,7 s | 4,3 Mo | `target/videos/146d09c/throne_three_floors.mp4` |
| [`throne_quad.mp4`](videos/throne_quad.mp4) | 56,7 s | 3,3 Mo | `target/videos/146d09c/throne_quad.mp4` |
| `montage.mp4` | — | 6,9 Mo | **lien seulement** (> 5 Mo) : `target/videos/146d09c/montage.mp4` |
| `throne_quad.vues.mp4` (quatre vues `--follow`) | 56,7 s | 14,3 Mo | **lien seulement** : `target/videos/146d09c/throne_quad.vues.mp4`, `make views SCENARIO=throne_quad` |

À voir en priorité : le troisième étage du quatuor (le roi des rats tire en couronne depuis le
coin le plus éloigné, mort vers f2558, nouveau boss au rechargement f3312) ; la mort du bot seul
dans la troisième caverne (f3640, cercles de télégraphe et chiffres de dégâts de T1.17) ; les
trois cavernes de tailles différentes du duo (48 × 32, 56 × 40, 64 × 44), qui étaient la même
avant le correctif.

## Ce qui manque pour M1

Critères de sortie du plan (`docs/plan-engine.md` §9.8) :

| Critère | État |
|---|---|
| Lint et tests verts sur `main` | **oui** (journal, dernière vérification sur l'état fusionné) |
| Tous les scénarios du clone en synctest à 2 et à 4 | **oui** : `throne_three_floors` (2), `throne_quad` (4), `throne_solo` (1), `clone_*` |
| Les bots finissent le clone sur 200 graines sans softlock ni desync | **oui** (2026-10-06, `ed8a274`) : 198/200 à 2 bots, 200/200 à 4 bots, 0 soft-lock, 0 desync |
| Bench dans les budgets | **oui, au calme le 2026-10-06 après D53** : `bench_horde` 58,7 / 59,6 fps (plancher 38), `bench_bullets` ≈ 135, `bench_cave` ≈ 195, `bots_four_mixed` ≈ 107 (journal, m1-d53-bench-horde) |
| Vidéos publiées | **en partie** : rendues d'après D51 et copiées dans le dépôt (section « Vidéos ») ; pas de publication automatique |
| Doc des conventions à jour | **oui** : relecture d'ensemble (m1-relecture-conventions), suites dans m1-dettes-doc-lot-2, chaque tâche depuis |
| Notes du jalon précédent fermées | **revue humaine de M0 en attente de William** |

Autres manques :
- **Rééquilibrage du troisième étage à décider** avec William : 2 défaites sur 200 à 2 bots depuis
  D51 (39 sur 200 avant). Leviers préparés (rapport m0-200-graines-test-map) : munitions garanties
  (D50), un tireur de moins dans `niveau_3`, pente de difficulté.
- **D50** (pénurie de munitions, graine 76) : prouvée avant D51, **non reproduite sur la graine 76 après
  D51** ; à rejuger.
- **D4** : phases de boss reportées à M2 ; `roi_rat` n'a qu'une liste de behaviors.
- **D40b** : `grunt` du testbed sans attaque (décision de William).
- Anticipation de la cible par les bots : mesurée, non livrée (m1-v3-bots-lead).


## Pour la revue humaine

À jouer, pas seulement à regarder. Commande (solo, local) : `make throne` (= `cargo run -p throne
--features native -- --local-port 7000 --players localhost`) ; le clone `zombies` : `make zombies`.

1. **Ressenti des armes après D51** : chaque arme tire enfin avec sa propre dispersion (revolver et
   laser précis, mitraillette 0,15 rad, rafale large). Les douze armes de throne se distinguent-elles ?
   Les armes de `zombies` aussi ont changé (même correctif).
2. **Difficulté du troisième étage** (`niveau_3`, boss `roi_rat`) : trop facile depuis D51 selon les
   bots (2 défaites sur 200) ; à juger en jouant, avant de choisir un levier (munitions garanties, un
   tireur de moins, pente de difficulté).
3. **Écran de mutation** (à chaque niveau) : touches 1, 2, 3 ; lisibilité des trois cartes, durée avant
   le choix d'office.
4. **HUD** : rads, niveau, munitions par type, statuts ; lisible pendant un combat ?
5. **Restart** : touche `R` ou bouton de fin de partie ; en ligne, « Rejouer » avec les mêmes pairs.
6. **Feedback** : hit stop, secousse, flash, télégraphe au sol des chargeurs et des tourelles,
   chiffres de dégâts : trop, pas assez ?
7. **Cavernes** : trois tailles, portail au barycentre des points des joueurs (ramené sur une case
   atteignable depuis D48) ; un ennemi qu'on ne trouve pas, un portail qu'on ne voit pas ?
