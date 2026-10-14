# m2-t17-generateur-v2 — générateur v2 : gabarits par objet, par salle et par boss (M2-T17, V3)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §28 (générateur v1 et
placement scripté), §35 (salles), §36 et §44 (objets, synergies), §37 et §45 (boss, arène), §47 (bot
explorateur), et `crates/scenario/src/generate.rs`. Branche `m2-t17-generateur-v2` depuis
`origin/main`. Estimation : 3 j. Section de conventions réservée : **§54**.

## Contexte

`make gen GAME=<jeu>` produit aujourd'hui des scénarios par arme (`WeaponOnTarget`) et par ennemi
(`EnemyVsStillPlayer`, `EnemyVsMovingPlayer`), opt-in par un champ `test:` dans la définition. Les
kinds M2 n'ont pas de gabarit : un objet, une salle ou un boss ajouté par M2-T14 n'est testé que si
quelqu'un écrit un scénario à la main. Les attentes nécessaires existent déjà : `RoomState`,
`HasItem`, `ItemCharge`, `Stat`, `BossPhase`, `ProfileHas`.

## À faire

1. **Objet** (`test:` optionnel dans `items/<id>.ron`) : `Template::ItemHeld`. Le joueur commence
   avec l'objet (même chemin que les objets de départ des scénarios), dans `arena.ldtk`, sans input.
   - Attentes par défaut : `HasItem` à la frame 1 ; une `Stat` par modificateur déclaré, vérifiée à
     la frame 1.
   - Pour un actif : `UseActive` à la frame 10, puis `ItemCharge` qui retombe.
   - Attentes typées en plus, comme pour les armes.
2. **Salle** (`test:` optionnel dans `rooms/<id>.ron`) : `Template::RoomClear`. Un étage d'une seule
   salle de ce type (gabarit LDtk tiré par graine fixe), un bot `explorateur` (§47), aucun input
   scripté. Attentes par défaut : `RoomState(Locked)` puis `RoomState(Cleared)` avant `frames`, et
   `PlayerAlive`. Si la salle ne verrouille pas (`locks: false`), seulement `PlayerAlive`.
3. **Boss** (`test:` optionnel sur un personnage qui a `boss:`) : `Template::BossFight`. Salle de boss,
   bot `fonceur` (comme `gungeon_boss_bot`), aucun input scripté. Attentes par défaut : chaque
   `BossPhase` atteinte avant `frames`, puis salle `Cleared`.
4. **Lint** : mêmes règles que `lint_character_test` (`frames > 0`, attentes seulement pour un
   gabarit actif) ; `RoomClear` sur un type de salle sans gabarit LDtk → refusé ; `BossFight` sur un
   personnage sans `boss:` → refusé. Fixtures.
5. **Contenu** : `test:` sur les objets, salles et le boss existants de `gungeon` et `testbed`.
   Commencer par 3 objets, 2 salles et le `gatling`, puis tous si les scénarios restent sous une
   minute chacun. Lister au rapport ce qui est couvert et ce qui ne l'est pas.
6. **`make gen`** couvre les nouveaux gabarits ; `make gen GAME=<jeu>` ×4 sans modification après
   régénération.

## Règles

Aucune trace existante ne doit bouger (champ `test:` absent = rien de généré). Les traces des
scénarios générés sont créées par cette tâche : vérifier les attentes de chacun **avant** le
bless, et noter au rapport la frame où chaque attente passe. Si un bot ne finit pas une salle ou ne
bat pas le boss, ne pas changer le bot : retirer le `test:` de ce contenu, le noter au rapport, et
l'orchestrateur en fera une dette. Pas de `f32`, `order_iter!`, `FrameEvents` (jamais `Message`).
`cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Gabarits par statut, par synergie et par arène (après M2-T14 si besoin), nouveaux profils de bot.

## Session cloud

Tâche prévue pour une session Claude Code cloud (pas la machine `orca`) : ignorer les consignes
`orca` du README (§1 : `make sweep`, ports de signaling) ; build sans `game/dev-dylib` si
`./scripts/dev-features.sh` ne le propose pas. La session ne peut pas écrire à l'orchestrateur :
**pousser la branche régulièrement** (commits WIP acceptés ; le conteneur redémarre environ toutes
les deux heures et tue les processus d'arrière-plan : lancer la suite au premier plan) et finir par
un commit `LIVRÉ m2-t17-generateur-v2` qui contient le rapport. p2p : non requis (aucun changement
de simulation) ; le dire au rapport.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`, `make gen` ×4) ; rapport
`docs/taches/rapports/m2-t17-generateur-v2.md`, conventions §54.
