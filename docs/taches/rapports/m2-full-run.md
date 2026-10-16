# m2-full-run — `gungeon_full_run` : la run de M2 de bout en bout

Session b2 (VPS Debian x86-64, 8 cœurs, nightly du repo), branche `m2-full-run` depuis `origin/main`
`0402a071` (M2-T14d/D64, M2-T12, M2-T13, `m2-armes-sources`, M3 T0a–T0d, T1, T2, T4, T6, T7, T13). Section de
conventions : sous-section « Run complète de M2 » de `docs/conventions.md` §57.

## 1. Fait

1. **Moments clés** (`crates/scenario/src/events.rs`) : `chest` (coffre ouvert, `ChestOpened`), `arena` (arène
   tenue : une salle dont le type porte une arène passe de verrouillée à nettoyée), `unlock` (déblocages du profil
   lus par la partie), `hub` (entrée en run depuis le hub, daté de la première frame de la run). Le portail
   (`portal`), l'étage (`floor`) et l'achat en boutique (`purchase`, raison `shop`) existaient. Attente
   `EventAfter` (moment clé survenu après une frame) pour prouver le second étage, car `Event` se contente d'une
   occurrence du premier.
2. **`Scenario::shopping`** : le bot `explorateur` ouvre les coffres et achète sans code côté appelant
   (`ExplorerOptions::shopping`, D62).
3. **Le runner suit la relance du hub** (`HubSetup { stay: false, inputs }`) : deux phases dans une boucle (hub,
   puis run dont les frames repartent de 0), pistes et bots échangés à la porte, `play` et `capture` aussi.
4. **`gungeon_full_run`** (2 bots, graine 10, `etages_deux`, hub, profil à trois déblocages, 7 700 frames) et
   **`gungeon_full_run_four`** (4 bots, même graine, 7 000 frames, synctest).
5. **`fuzz_inputs`** : mode `ALACOD_FUZZ_SCENARIO=<nom>` (inputs aléatoires à la place des bots d'un scénario,
   pistes du hub conservées).
6. **`bench_floor`** (grammaire `grand`) : plancher et plafond de rollback mesurés (§4). Budgets des deux scénarios.
7. **`ALACOD_EVENTS_JSON=<dossier>`** (`tests/scenarios.rs`) : `events.json` sans rendu ; ceux des deux scénarios sont
   dans `docs/taches/rapports/m2-full-run/` (page de revue). **La vidéo n'a pas pu être rendue** (§5).
8. **Deux défauts du moteur trouvés et corrigés** (§2), plus un détail de déterminisme (`HashMap` → `BTreeMap`).

## 2. Défauts trouvés en le faisant (aucune trace existante ne bouge)

1. **Hub → run : toutes les portes restaient fermées.** Le hub est une salle typée (`room_kind: depart`) ; sa
   session posait `RoomStates::doors_opened`, que `cleanup_rollback_world_system` ne remettait pas à zéro. Dans la
   run lancée par la porte, aucun bot ni joueur ne sortait de la salle de départ (sonde `RoomState(doors_closed)` :
   **39 portes fermées, contre 3 sans hub**, bots immobiles, `explore_step` sans route). Même défaut pour le restart
   (touche R) d'une partie gungeon : les salles nettoyées de la partie précédente restaient nettoyées.
   Correction : `RoomStates`, `ArenaClocks`, `DoorLocks` remis à zéro à la sortie d'`InGame` ; assertions dans
   `crates/scenario/tests/hub.rs`. Reproduit avant correction (sonde ci-dessus, code de `origin/main`).
2. **4 joueurs sur gungeon : panique « pas de point de spawn pour le joueur 2 ».** Le gabarit `Depart` n'a que deux
   `PlayerSpawn`. Ajouter deux points au gabarit (essayé) déplace le barycentre du portail et change la numérotation
   des entités : toutes les traces gungeon (boss 205 → 207 sur la graine 10). Choix : `player_spawn_point`
   (`map_ldtk::game::local`), un joueur sans point propre prend celui de rang `handle % n` décalé de 24 unités vers
   le bas par tour, seulement là où le chargement paniquait ; test unitaire. (`HashMap` des spawns remplacé par un
   `BTreeMap`, deux entrées de `determinism-allowlist.json` retirées.)

Hors du périmètre mais à savoir : à l'étage suivant, `floors.rs` laisse « en place » un joueur sans `PlayerSpawn`
(joueurs 2 et 3 à 4 joueurs) : ils arrivent là où ils étaient, ce qui suffit ici (les quatre gagnent).

## 3. Mécaniques attestées (graine 10)

À 2 joueurs, frames de la run (`gungeon_full_run`) ; chaque ligne a une attente ET un moment clé dans
`events.json`. Les moments clés tirés de `FrameEvents` portent la frame d'émission (une de moins que la frame lue
après l'update : `by_frame` = frame + 1).

| Mécanique | Frame | Attente |
|---|---|---|
| Entrée depuis le hub (M2-T13) | marchand f80 (visite f120), porte f175 du hub, `hub` f1 | `HubVisit(marchand)`, `HubExit(joueur 0, f175)`, `Event(hub)` |
| Déblocage lu depuis un profil (M2-T12) | f1 | `RunUnlocks(tresor_trefle, coffre_riche, etendard_royal)`, `Event(unlock)` |
| Salle verrouillée puis nettoyée | f42 → f139 | `Event(room)` « verrouillée » puis « nettoyée » |
| Blank utilisé | f70, f82 (les deux joueurs) | `Event(blank)` |
| Objet ramassé | clé f157, rancune f3022 | `Event(item_pickup)`, `Consumable(key)`, `HasItem(rancune)` |
| Achat en boutique | f2825, f2850, f2887 (gilet, roulade) | `Event(purchase, shop)`, `HasItem(gilet)`, `HasItem(roulade)` |
| Coffre ouvert, clé consommée | f2928 (`arc`) | `Consumable(key)` 1 (f2920) puis 0 (f2930), `Event(chest)` |
| Arène tenue | f1890 → f2640 (750 frames) | `Event(arena)`, `RoomState(arene, Cleared)` |
| Boss et ses trois phases | verrou f3406, phases f3439 et f3487 | `BossPhase(NetId(205), 2)`, `Event(boss_phase)` |
| Portail | f3489 | `Event(portal)` |
| Second étage | f3691 ; coffre f4711, boutique f5249, arène f6172-f6922, boss f7325 (phases f7471, f7532) | `FloorIndex(1)`, `EventAfter` ×4, `BossPhase(NetId(694), 2)` |
| Victoire, profil | f7545 | `RunState(Ended(Victory))`, `ProfileHas(first_victory)` |

À 4 joueurs (`gungeon_full_run_four`) : même parcours, victoire f6876, quatre joueurs en vie ; clé du joueur 1
consommée par le coffre de f2671 (`Consumable` 1 → 0).

**Graine retenue : 10.** Campagne `alacod-sim --shopping` (2 bots, `etages_deux`, graines 1 à 64, 15 000 frames,
sans hub) : 48 victoires, 13 soft-locks (bot, D82), 3 défaites, 0 desync. Les graines 13 et 41 citées par le
rapport T14d : 41 gagne (f6781), 13 meurt (f7417). La 10 est une graine où tout arrive et où les phases 0 → 1 → 2
du boss tombent aux deux étages ; elle gagne à 2 et à 4 joueurs. Rejouées avec le hub et le profil à 2 joueurs,
les graines 1, 2, 9 et 10 gagnent (f6484, f6176, f7385, f7545) ; à 4 joueurs les quatre gagnent aussi (essai fait
avec des points de spawn ajoutés au gabarit, puis refait avec le repli de spawn pour la 10 seule). Choix de la 10
pour la richesse des événements, pas pour la durée (la 2 est la plus courte).

## 4. Vérifié (résultats réels)

Machine : VPS Debian 8 cœurs, partagé avec l'orchestrateur (charge ambiante de 1,6 à 2,4 quand je ne lançais
rien ; les campagnes et la suite sont au premier plan). `CI=true` dans l'environnement : pas de `dev-dylib`.

| Commande | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` sur la branche (avant fusion), sans BLESS des traces existantes | **470 scénarios verts**, 1 123 s ; aucune `.trace` existante modifiée (`git status`), seules les deux nouvelles sont ajoutées |
| même commande **sur la fusion avec `origin/main` `a9432ed9`** (M3-T2 comprise) | **484 scénarios verts**, 1 318 s |
| même commande **sur la fusion finale avec `origin/main` `0a719425`** (outillage-p2p-stress comprise ; seul conflit : le journal de `docs/taches.md`) | **484 scénarios verts**, 1 044 s, aucune trace existante modifiée |
| `make test_crates` | avant fusion : 955 réussis, 0 échec, 13 ignorés (70 suites) ; sur la fusion : **977 réussis**, 0 échec, 13 ignorés |
| `make lint` ×5 (zombies, testbed, throne, gungeon, isaac) | 0 erreur, sur la branche et sur la fusion |
| `make gen GAME=…` ×5 | sortie 0 et **aucun diff** (`git status` propre hors mes fichiers), sur la branche et sur la fusion |
| `cargo fmt --all -- --check`, `check-forbidden.sh`, `check-rollback-registration.sh`, `make check_sim_boundary` | OK ; frontière : 11 crates de simulation, 5 en dette D69 |
| `ALACOD_FUZZ=0:8 ALACOD_FUZZ_SCENARIO=gungeon_full_run` puis `…_four` (3 600 frames, synctest) | 8 graines chacun, **aucune divergence**, joueurs en vie |
| `make bench` (strict, un processus, au calme relatif) | **échoue sur 27 scénarios** (29 dépassements) qui ne tiennent pas leur budget **déjà** (voir ci-dessous) ; mes trois scénarios passent |
| p2p à deux clients, stress à rollbacks réels (`scripts/p2p-stress.sh zombies /tmp/inputs.ron 3000`, signaling Docker local port 3566), **sur la fusion `0a719425`** | `p2p-stress : OK` : codes de sortie 0 et 0, 0 `DesyncDetected`, **3 909 frames resimulées** (client 0), traces identiques sur 2 999 lignes. Un premier essai sans stress (600 frames, sans script) : traces identiques, 0 frame resimulée |
| `./scripts/p2p-restart.sh` (restart en ligne : touche `cleanup_rollback_world_system`), **sur la fusion `0a719425`** | partie 1 et partie 2 : traces identiques (599 lignes) ; partie 2 ≠ partie 1 ; local : partie 2 = partie 1 |

**Bench `grand`** (`bench_floor`, étage de 15 à 20 salles, 2 bots `fonceur`, 3 000 frames) : trois relevés isolés
44,1 / 44,1 / 46,3 sim fps et pire rollback 25,3 à 26,6 ms ; dans `make bench` 48,1 fps et 24,8 ms ; 322 entités,
43 ennemis, instantané de 87 KiB. Budget `tests/budgets.ron` : `min_sim_fps: 20`, `max_rollback_ms: 50` (la moitié et
le double, comme le reste du fichier). **L'ancien plafond de 18 ms (poste M4, 2026-10-12) n'était pas tenable sur ce
poste** (il échouait en strict à 24,8 ms). Mesuré avec une charge ambiante de 2 à 2,4 (un processus de moi, le reste
à l'orchestrateur) : « au calme » au sens du poste partagé, pas sur une machine vide. Budgets des deux nouveaux
scénarios : 53,5 et 49,1 sim fps, rollback 20,2 et 20,7 ms dans `make bench` ; planchers 20 fps, plafonds 40 ms.

**Les 27 scénarios (29 dépassements) de `make bench` ne sont pas les miens, 6 reproduits sur `origin/main`
`a9432ed9`** (worktree jetable, même target, un processus, `ALACOD_BENCH_STRICT=1`) : `idle` 87,9 fps (plancher 110 ;
88,7 sur ma branche), `two_players_idle` 70,1 (80 ; 79,2), `four_players_idle` 80,1 (100 ; 66,5), `bot_explorer_two`
81,6 (100 ; 70,9), `testbed_follower` 90,6 (170 ; 89,4), `bench_cave` 72,7 fps et 14,6 ms de rollback (100 et 5 ;
78,6 et 13,6). Les planchers ont été fixés sur d'autres machines (macOS arm64, cloud 4 cœurs) ; les 21 autres
(`ammo_burst`, `bench_bullets*`, `testbed_*`, `weapons_workout`…) n'ont pas été rejoués sur `main`. Écarts de ±15 %
d'un passage à l'autre. `scripts/scenario-metrics.py` n'a pas tourné (`make bench` s'arrête à l'échec).

**Relecture indépendante** (agent `reviewer`, lecture seule, deux passes) : trois constats P2 à la première passe,
corrigés — réglages de joueur perdus après le hub (refus explicite), `play`/`capture` qui s'arrêtaient au hub,
seconde arène non distinguée de la première (`EventAfter`) — puis un quatrième à la seconde (la capture écrivait les
images du hub), corrigé. Aucun BLOQUANT.

## 5. Non fait / incertain / non vérifié

- **Vidéo : non faite.** `scripts/scenario-video` panique à l'ouverture de la fenêtre (ni X11 ni Wayland, pas d'`Xvfb`,
  pas de root ; `lavapipe` est installé). `events.json` des deux scénarios produits sans rendu
  (`ALACOD_EVENTS_JSON`, `docs/taches/rapports/m2-full-run/`). La modification de `capture`/`play` pour le hub suivi
  est **compilée mais jamais exécutée** (pas de capture possible ici).
- **p2p de la run complète en ligne** (hub, deux étages, D59 en réel) : non fait, c'est `m2-p2p-run-complete`. Le
  stress à rollbacks réels n'a été joué que sur **zombies** (3 909 frames resimulées, §4) et le restart en ligne
  (`p2p-restart.sh`) ; aucun p2p sur gungeon.
- `make bench` n'est pas vert sur ce poste (27 scénarios, 29 dépassements hérités, §4) ; seuls 6 de ces 27 ont été
  rejoués sur `main`.
- Le fuzz d'inputs aléatoires ne traverse pas les étages (joueurs en vie, couverture non mesurée).
- [INFERENCE] `ExplorerVisits` (état du bot, hors simulation) n'est pas remis à zéro au restart : un restart (R)
  d'une partie de bots n'est pas couvert, non examiné.
- 4 joueurs : pas de point de spawn propre pour les joueurs 2 et 3 (repli décalé), et à l'étage suivant ils restent
  « en place » (`floors.rs`) ; pas de scénario à 4 joueurs de l'étage 2 au départ (ils gagnent, c'est tout).

## 6. Dettes, questions

- **Budgets de `make bench`** : 27 scénarios (29 dépassements) ne tiennent pas leur plancher ou leur plafond sur le
  VPS Debian (relevés faits sur macOS arm64 / cloud 4 cœurs). Une tâche de recalibrage par machine (ou des budgets par
  poste) manque ; sinon `make bench` ne peut pas servir de critère de sortie ici.
- **Gabarit `Depart` à quatre `PlayerSpawn`** : propre, mais déplace le barycentre du portail et la numérotation
  des entités de toutes les traces gungeon (essayé : boss 205 → 207). À faire avec un bless justifié si l'on veut des
  points de départ propres ; le repli de `player_spawn_point` suffit tant que ce n'est pas fait.
- **`Event`** ne filtre pas par frame (l'attente cherche dans tout l'historique) et les moments clés tirés de
  `FrameEvents` ont une frame de retard sur la lecture : `EventAfter` règle le premier, pas le second (documenté).
- Hub suivi : attentes continues et réglages de joueur refusés ; pas de rejeu depuis un enregistrement avec hub.
- **D82** : 13 soft-locks sur 64 graines à 2 bots (cycle limite du bot devant une ouverture de deux cases).
- Remote : le push en SSH échoue (« Host key verification failed », `github.com` absent de `known_hosts`) ; poussé
  en HTTPS (identifiants `gh`), `origin` remis en HTTPS. Pas changé `known_hosts` (décision de confiance à toi).

## 7. Atteignabilité (M2, `docs/taches.md` §7)

`gungeon_full_run` rencontre : salles verrouillées, objets, coffre à clé, boutique, blank, arène, boss à trois
phases, portail, second étage, déblocage lu du profil, entrée par le hub. Commande d'une partie ordinaire :
`alacod-sim --game gungeon --bots 2 --profiles explorateur,explorateur --floors etages_deux --until-floor 2
--seeds 10..10 --shopping`.
