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
6. **`bench_floor`** (grammaire `grand`) : plancher et plafond de rollback mesurés (§3). Budgets des deux scénarios.
7. **Deux défauts du moteur trouvés et corrigés** (§2).

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

TBD

## 5. Non fait / incertain / non vérifié

TBD

## 6. Dettes, questions

TBD

## 7. Atteignabilité (M2, `docs/taches.md` §7)

`gungeon_full_run` rencontre : salles verrouillées, objets, coffre à clé, boutique, blank, arène, boss à trois
phases, portail, second étage, déblocage lu du profil, entrée par le hub. Commande d'une partie ordinaire :
`alacod-sim --game gungeon --bots 2 --profiles explorateur,explorateur --floors etages_deux --until-floor 2
--seeds 10..10 --shopping`.
