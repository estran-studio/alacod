# Rapport revue-sortie-m2 — ce qui manque pour sortir de M2

Audit et liste de tâches, **aucun code de jeu**. Session cloud (Linux x86-64, 4 cœurs, nightly
`2026-10-14`), branche `revue-sortie-m2` depuis `origin/main` `7172660`. Mesures de données dans
`docs/taches/rapports/revue-sortie-m2/` (JSON des campagnes, sortie du lint).

## 1. En bref

- **M2 n'est pas prête à sortir.** Les mécaniques sont toutes mergées sauf le profil et le hub
  (M2-T12, M2-T13) ; ce qui manque, c'est qu'une partie ordinaire les rencontre, qu'un bot la finisse,
  et qu'un humain l'ait vue.
- **Constat le plus grave, mesuré ici** : la campagne à 2 bots sur `etages_deux` (20 graines) donne
  **6 desyncs de synctest, 11 soft-locks, 3 morts, 0 partie finie**. Les campagnes committées
  (macOS arm64, avant T14b/T14c) donnaient 18/20 sur les mêmes graines, sans desync. Le résultat est
  **identique sur `2e87834` et sur `main`** : ce n'est pas une régression récente du contenu, c'est un
  écart entre la mesure committée et ce poste, à expliquer avant toute sortie (§2.2).
- **20 armes sur 21 ne sont données par rien** : un joueur n'a que le `pistolet` (lint : 20
  avertissements, dette D62 côté armes).
- Bilan : sur 14 critères, **5 faits, 6 partiels, 3 manquants**. Chemin critique : environ 10 jours
  de session cloud, une demi-journée de William et le poste d'orchestration (p2p).
- **P2P : non mesuré ici** (pas de Docker dans la session cloud, une seule machine). Tout ce qui
  touche au réseau réel reste « non vérifié ».

## 2. Mesures sur `main` (`7172660`)

### 2.1 `make lint GAME=gungeon`

`alacod lint games/gungeon` : **0 erreur, 20 avertissements** (11 personnages, 21 armes, 1 arme de
corps à corps, 3 cartes). Les 20 sont du même type, « arme X : aucun personnage, mur ni pool ne la
donne » : les 18 armes de M2-T14a, plus `arsenal` et `arsenal_lourd` (contenu de test, D63 pas encore
appliquée à gungeon). Aucun objet, ennemi ni salle sans source. Liste : `revue-sortie-m2/lint-gungeon.txt`.
Les autres jeux n'ont pas été relancés (hors du périmètre demandé).

### 2.2 Campagne `alacod-sim`, séquence par défaut (`etages_deux`)

Commande (binaire `headless`, sans `dev-dylib`, comme la CI) :

```
alacod-sim --game gungeon --bots 2 --profiles explorateur,explorateur --floors etages_deux \
  --seeds 1..20 --until-floor 1 --max-frames 20000 --json …
```

| Configuration (20 graines) | Fini | Mort | Soft-lock | Desync | Durée |
|---|---|---|---|---|---|
| `etages_deux`, 2 bots (**la demandée**) | 0 | 3 (1, 13, 20) | 11 (3, 4, 6, 7, 9, 10, 11, 14, 15, 16, 19) | **6** (2, 5, 8, 12, 17, 18) | 177 s |
| même, sur `2e87834` (avant T14b/T14c/T18, après D65) | 0 | 3 | 11 | 6 | 164 s |
| `etages_deux`, 1 bot | 0 | 10 | 10 | 0 | 99 s |
| `etages_court_deux`, 2 bots | 9 | 0 | 11 | 0 | 88 s |
| *Référence committée* `e2` (macOS arm64, `1a85d51b`), mêmes graines | 18 | 2 | 0 | 0 | — |

Lecture :

1. **Desync** (graine 2, rejouée avec `--log`) : divergence autour de la frame 57-58, dans l'arène
   verrouillée (vague de deux `bullet_kin`) ; le journal montre des `Entity despawned` en `LoadWorld`
   et `AdvanceWorld`, signe d'un despawn d'entité rollback (règle `despawn_rollback`, `CLAUDE.md` §9).
   **Cause non établie** : c'est la première tâche de la liste.
2. **Soft-locks** : zéro dégât, zéro salle nettoyée, 1 201 frames (le plafond d'immobilité), et sur
   plusieurs graines les joueurs s'arrêtent à la même case (graines 7 et 11 : (-3,1 ; -118,3)) avec
   des ennemis « en vue » à 25 px. Le bot ne tire pas, ou la salle n'est pas activée, ou la navigation
   diffère : non établi.
3. **Pourquoi l'écart avec les campagnes committées ?** Non établi. Éléments : (a) même résultat
   avant et après T14b/T14c, donc pas le contenu ; (b) les scénarios de la suite rejouent leur trace
   ici (`gungeon_two_floors`, `bot_explorer_two`, `shop_seeded`, `gungeon_boss_bot`, `dash_charges`,
   `gungeon_floor_seed_1` : tous verts, 6 réussis), donc la simulation scriptée est la même ; (c) D73
   (x86 ≠ arm64) vient d'être fermée en fausse alerte, donc le déterminisme entre architectures n'est
   pas en cause *à graine fixe*, mais le comportement des **bots** (décisions, navigation) n'a pas été
   comparé entre architectures ; (d) le `rust-toolchain` est un `nightly` non épinglé (le poste a
   installé celui du 2026-10-14). À trancher par `m2-campagne-linux-desync`.
4. La mesure ne couvre pas 200 graines (20 demandées) ni 4 joueurs ; elle suffit pour dire que le
   critère « bots sans soft-lock ni desync » **n'est pas tenu sur ce poste**.

Réserve : `kills` vaut 0 partout, y compris dans la référence committée : en mode `Floors`, la colonne
ne compte pas les ennemis d'étage ; ce n'est pas un signe d'échec en soi.

### 2.3 Scénarios `gungeon` et ce qu'ils couvrent

180 scénarios dans `tests/scenarios/` + 55 générés pour gungeon (`tests/scenarios/generated/gungeon/` :
22 `weapon_*`, 16 `item_*`, 14 `enemy_*`, 2 `room_*`, 1 `boss_*`). **17 scénarios écrits à la main
jouent le jeu gungeon** :

| Scénarios (`game: gungeon`) | Couverture |
|---|---|
| `gungeon_start`, `gungeon_clear`, `gungeon_boss`, `gungeon_boss_bot` | étage squelette T0d : départ, nettoyage, boss, bot contre le boss |
| `gungeon_floor_start`, `gungeon_floor_seed_{1,23,123}`, `gungeon_two_floors` | étages assemblés par grammaire, transition d'étage (synctest) |
| `gungeon_squad` | escouade en arc (T8), grammaire `court_escouade` seulement |
| `shop_seeded` | boutique : trois présentoirs tirés par la graine (T6) |
| `dash_charges` | charges de roulade |
| `bot_explorer_floor`, `bot_explorer_two` | bot explorateur sur `court` (1 et 2 joueurs) |
| `bench_floor` | perf d'un étage (plancher 40 fps, rollback 18 ms) |
| `ui_inventory`, `ui_pause_menu` | écrans (carte `etage_1`) |

**Trou important** : la plupart des mécaniques de M2 sont prouvées par des scénarios du **testbed**
(`game: testbed` : `blank_*`, `chest_locked`, `effect_on_*`, `dash_*`, `pit_*`, `boss_*`,
`rooms_*`, `item_*` sauf les générés, `synergy_*`, `profile_run_end`, `room_reward`, `shop_buy`,
`shop_refused`, `shop_tour`, `bench_bullets_room`). Elles marchent, mais **aucun scénario gungeon ne
montre un coffre ouvert, un blank, un achat ou l'arène**. `gungeon_full_run` (§7 de `taches.md`) n'existe
pas : c'est la tâche 6.

## 3. Critères de sortie un par un

Sources : plan §9.8, ligne M2 du tableau §6, brouillon M2 §5, contrat de sortie d'un jalon
(`docs/etat-engine.md`), règle d'atteignabilité du 2026-10-14. **Fait** = mergé, avec preuve, et
rencontré par une partie ordinaire ou par un scénario gungeon ; **partiel** = mergé mais pas rencontré
ou pas prouvé en partie ordinaire ; **manquant** = absent ou échec mesuré.

| # | Critère | État | Preuve ou trou |
|---|---|---|---|
| A | Étages de salles typées et verrouillées | **fait** | E1 (`rooms_*`, `room_state`), T10 (grammaire, 1 000 graines), `gungeon_floor_seed_*`, `gungeon_two_floors` ; `etages_deux` est la partie par défaut (D59) |
| B | Bullet hell (patterns, rebonds, formations) | **fait** | T1 (`Spiral`/`Fan`/`Burst`), T14c (patterns v2), `bench_bullets_room` (535 balles, plancher 40 fps, rollback 26 ms), T19 (télégraphes) ; mesure de bench non rejouée ici |
| C | Roulade à i-frames, charges, variantes | **fait** | T2 : `dash_charges` (gungeon), `dash_iframes`, `dash_jump_bullets`, `dash_shove`… (testbed) |
| D | Blanks | **fait** | T1 : `blank_clears_bullets`, `blank_empty`, `blank_floor_refill` ; le personnage démarre avec 2 `blank` ; anneau T19, HUD T18 |
| E | Objets passifs et actifs, synergies | **fait** | T0b/T4/T14b : 20 objets (lint : tous ont une source), 6 synergies, 16 `item_*` générés + `synergy_pair`/`synergy_lost` ; `gilet` pose l'armure (D61) |
| F | Coffres et clés | **partiel** | T5 mergé (`chest_locked` en testbed, `room_reward`) ; jamais joué dans une partie gungeon : le bot meurt ou se bloque avant. Source des armes absente (critère N) |
| G | Boutique | **partiel** | T6 : `shop_seeded` (gungeon), `shop_buy`, `shop_refused`, `shop_tour` ; l'achat n'est joué par aucune partie de bot (rapport T6) ; pas de sprite de présentoir ni de douille |
| H | Boss à phases et arènes | **partiel** | T7 + T14c : boss `gatling` à trois phases, `gungeon_boss*`, `boss_*` ; l'arène n'est jouée par aucun scénario gungeon (D65 : le bot la joue depuis bot-explorateur-v2, `etages_deux`, mais voir critère L) ; HUD du boss en T18 |
| I | Hub et déblocages | **manquant** | M2-T12 (profil relu par la partie suivante) et M2-T13 (hub, entrée en run) : fiches prêtes, non lancées ; `meta::Profile` est écrit en fin de run mais **jamais lu par la simulation** |
| J | 1 à 4 joueurs en ligne | **partiel** | synctest à 1, 2 et 4 (scénarios `two_*`, `four_*`, p2p en mémoire) ; **p2p réel jamais joué sur gungeon** (D59 en réel, transition d'étage) ; `p2p_smoke` ne couvre que zombies ; réseau dégradé et plateformes non mesurés (`docs/etat-engine.md`, points 3 à 5) ; **non mesuré ici (pas de Docker)** |
| K | Scénarios verts, lint, bench dans les budgets | **partiel** | lint gungeon 0 erreur ; 6 scénarios gungeon rejoués ici, verts, traces identiques ; **suite complète non rejouée par cette revue** (1 à 2 h) ; bench non lancé au calme ; nightly CI jamais vue verte (`docs/etat-engine.md`) |
| L | Bots : 200 graines sans soft-lock ni desync | **manquant** | échec mesuré (§2.2) : 6 desyncs, 11 soft-locks sur 20 graines ; D64 (équilibrage des salles `combat` : e2 140/200 sur la mesure committée), D68 (soft-lock de réanimation, graine 90) ouverts |
| M | Vidéos publiées, revue humaine, notes fermées | **manquant** | aucun digest ni vidéo de M2 dans `docs/digests/` ; `session-test-manuelle` jamais jouée sur gungeon ; revue humaine de M1 (`m1-revue-humaine`) toujours attendue ; `tests/review-notes/` ne contient que des notes de M1 |
| N | Contenu en RON : 20 armes, 20 objets, 10 ennemis, 1 boss, 15 gabarits | **partiel** | 21 armes, 20 objets, 10 ennemis (+ `cible`), 1 boss, **12 gabarits** (`Depart`, `CombatEW/NS/T/Croix`, `Recompense`, `Boutique`, `BossW/N`, `ArenaEW/Croix`, `EscouadeEW`) sur 15 ; **20 armes sur 21 sans source** (D62) ; `escouade` hors de la grammaire `etage` ; `test:` sur les définitions (générés verts) |

Décompte : fait A à E (5), partiel F, G, H, J, K, N (6), manquant I, L, M (3).

**Atteignabilité** (règle du 2026-10-14) : objets, ennemis, salles, boutique, arène, récompense sont
dans la grammaire `etage` ; sont encore hors partie ordinaire : les armes (D62), la salle `escouade`
(T14d), le profil et le hub (T12, T13).

## 4. Tâches restantes

Ajoutées à `docs/taches.md` §7 (« Reste à faire pour sortir de M2 »), dans l'ordre. **B** bloque la
sortie, **P** est du polissage.

| # | | Tâche | Fiche | Estimation | Cloud |
|---|---|---|---|---|---|
| 1 | B | `m2-campagne-linux-desync` : cause du desync d'arène et des soft-locks, campagne refaite (200 graines, 1 et 2 bots) | **nouvelle** `m2-campagne-linux-desync.md` | 3 j | oui |
| 2 | B | M2-T14d gabarits + D64 | `m2-t14-contenu-gungeon.md` (§ T14d) | 3 j | oui |
| 3 | B | `m2-armes-sources` : sources des armes (D62) | **nouvelle** `m2-armes-sources.md` | 2 j | oui |
| 4 | B | M2-T12 profil et déblocages | `m2-t12-profil-deblocages.md` | 3 j | oui |
| 5 | B | M2-T13 hub | `m2-t13-hub.md` | 3 j | oui |
| 6 | B | `m2-full-run` : `gungeon_full_run`, bench `grand`, 4 joueurs | **nouvelle** `m2-full-run.md` | 2 j | oui |
| 7 | B | `m2-p2p-run-complete` : run complète en ligne à 2 puis 4, D59 en réel, réseau dégradé | **nouvelle** `m2-p2p-run-complete.md` | 1,5 j | **non** (Docker) |
| 8 | B | Revue humaine puis `m2-cloture-videos-digest` | **nouvelle** `m2-cloture-videos-digest.md` | 1 j + William | partiel |
| 9 | P | D71, D72 | `dettes-ui-d71-d72.md` | 1,5 j | oui |
| 10 | P | D66/D67 | `lisibilite-throne-d66-d67.md` | 2 j | oui |
| 11 | P | Sprites et sons propres, équilibrage fin | — (direction artistique) | — | non |

Ordre et dépendances : 1, 3, 9, 10 en parallèle tout de suite ; 2 démarre en parallèle sur les
gabarits mais attend 1 pour mesurer D64 ; 4 attend 3 (tables de butin) ; 5 attend 4 ; 6 attend 1 à 5 ;
7 et 8 en dernier. Sections de conventions réservées : §51 (T12), §58 (tâche 1), §59 (tâche 3).
Les deux premières lignes sont ce qui change le plus la confiance dans le reste : tant qu'un bot ne
finit pas `etages_deux` sans desync, aucune mécanique de M2 n'est prouvée en partie ordinaire.

## 5. Questions pour William (validation humaine)

1. **Vidéos et ressenti** : à jouer quand les tâches 1 à 3 sont mergées (sinon la partie meurt au
   premier combat ou ne donne qu'un pistolet) : `cargo run -p gungeon` seul puis à deux, protocole de
   `session-test-manuelle.md`. D'accord pour attendre, ou veux-tu une première passe maintenant sur
   l'étage squelette (`gungeon_clear`, `gungeon_boss`) ?
2. **Équilibrage (D64)** : quelle difficulté vise le premier étage pour un humain seul : le bot meurt
   dans la moitié des parties à 2 bots sur la mesure committée ; l'objectif de T14d (moins de 15 % de
   morts à 2 bots, moins de 50 % à 1) te convient-il ?
3. **Sons** : gungeon a 10 fichiers (copiés de `throne`) ; veux-tu une passe son propre à M2 (tir par
   arme, blank, coffre, boutique, phases du boss) avant la sortie, ou après ?
4. **Sprites** : sprites placeholders partout (pas de présentoir, de douille, de clé) ; la sortie de
   M2 attend-elle les sprites ou le jalon est-il « mécaniques jouables » ?
5. **Partie en ligne** : à rejouer sur deux machines (x86 et arm64) une fois la tâche 1 faite ; qui la
   lance (toi sur le Mac et orca) ? Quelles conditions de réseau veux-tu comme cible (RTT, perte) ?
6. **Revue humaine de M1** (`m1-revue-humaine`) : toujours attendue ; elle bloque « notes du jalon
   précédent fermées » du contrat de sortie. À faire avant ou avec celle de M2 ?
7. **Toolchain** : épingler le nightly par date (l'écart de mesure entre macOS et ce poste pourrait en
   venir) : décision ?

## 6. Non vérifié, non fait

- **P2P** (matchbox, deux machines, réseau dégradé, plateformes) : non mesuré, pas de Docker.
- Suite de scénarios complète, `make test_crates`, bench strict, `fuzz_inputs`, `make gen` : non
  rejoués (seuls 6 scénarios gungeon, le lint et les campagnes ci-dessus ont été exécutés).
- Campagnes sur 20 graines et non 200 ; 2 bots (plus deux variantes de contrôle) et non 4.
- Cause du desync et des soft-locks : non établie (c'est la tâche 1).
- Les autres jeux (zombies, throne, testbed) : lint non relancé.
- Aucun code de jeu modifié ; seuls des fichiers sous `docs/` ont été ajoutés ou modifiés.

## 7. Reproduire

Cloud vierge : `apt-get install pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
libx11-dev` ; `cargo build -p scenario --bin alacod-sim --profile headless --no-default-features`
(16 min à froid, 4 cœurs) ; commande du §2.2 ; lint : `cargo run -q -p content --bin alacod --profile
headless -- lint games/gungeon`.
