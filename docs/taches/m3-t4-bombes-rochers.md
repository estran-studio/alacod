# m3-t4-bombes-rochers — bombes posées, rochers, piques, mur secret (M3, V1d, E3/E4 v2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (champ de flux, despawn différé), `docs/taches.md` §8.1,
`docs/conventions.md` §21, §26, §46, §62 (contrats de M3-T0c) et le rapport
`docs/taches/rapports/m4-t0c-contrats-monde.md`. Branche `m3-t4-bombes-rochers` depuis `origin/main`, **après
le merge de M3-T0c**. Estimation : 4 j. Section de conventions réservée : **§69**.

## À faire

1. **Bombe** : appui sur `Bomb` (front montant, comme le blank) avec au moins une `bomb` → bombe posée aux
   pieds, mèche en frames, puis `Explode` (dégâts à tous les personnages à PV du rayon, joueurs compris :
   tir ami de l'explosion = **Q4**/donnée `FriendlyFire`), crédit au poseur. `FrameEvents<Exploded>` neutre,
   moment clé `explosion`.
2. **Baril sur `Explode`** : `barrel_explosion_system` réutilise la règle de T0c ; ses traces
   (`barrel_chain`, gungeon) doivent rester identiques (preuve §5 si non).
3. **Rochers cassés** : `destroys: true` casse les rochers du rayon (`destroy_terrain`), murs et champ de
   flux à jour (même chemin que les cavernes) ; rocher à butin : pose un objet tiré par `loot_seed` avec la
   clé `rock:<salle>:<x>:<y>`.
4. **Mur secret** : une ouverture `Secret` touchée par une explosion s'ouvre (état rollback de T0c) ; les deux
   salles sont alors reliées dans le champ de flux.
5. **Piques** : `hazard` exécuté (période, dégâts) ; le dash ne les franchit pas sauf donnée.
6. **Scénarios** (testbed) : `bomb_place_explode`, `bomb_breaks_rock`, `bomb_hurts_owner` (si la règle le
   veut), `bomb_chain_barrel`, `rock_loot`, `secret_wall_bomb`, `spikes_damage`, `rock_flow_field` (un ennemi
   contourne puis traverse après l'explosion).

## Règles

Despawn des bombes par `despawn_rollback()` ; ordre des explosions d'une frame par `GgrsNetId` ; cases
cassées dans un ordre trié. `fuzz_inputs` (`ALACOD_FUZZ=0:8`) avec un personnage à bombes. Les jeux sans
`bomb` ne bougent pas.

## Hors périmètre

Placement du mur secret par la grammaire (T5), bombes du bot (T14), visuels (T17).

## Vérification

README §4 plus `fuzz_inputs` ; rapport `docs/taches/rapports/m3-t4-bombes-rochers.md`, conventions §69.

## Atteignabilité

Bombes de départ du personnage de `isaac` et récompenses ; rochers et piques dans les gabarits `combat` de
T11d ; mur secret placé par T5. Graine nommée au rapport de T11d.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker :
p2p rejoué par l'orchestrateur, le dire au rapport. Pousser la branche régulièrement. Voie rapide du README
§6 : fusionner `origin/main` et rejouer la suite juste avant le commit `LIVRÉ m3-t4-bombes-rochers`, qui
contient le rapport.
