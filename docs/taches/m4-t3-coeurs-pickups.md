# m4-t3-coeurs-pickups — cœurs, i-frames, sous, bombes et clés au sol (M4, V1b, C4/C5)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §12, §18,
§36, §44, §48, §50, §61 (contrats de M4-T0b). Branche `m4-t3-coeurs-pickups` depuis `origin/main`, **après le
merge de M4-T0b** et la réponse de William à **Q2** (modèle des cœurs) et **Q4** (partage en coop).
Estimation : 3 j. Section de conventions réservée : **§68**.

## À faire

1. **Couche de vie** selon Q2 (proposition de T0b : `ExtraHealth { soul }` absorbé point par point avant les
   PV) : résolveur, consommable « cœur d'âme » qui la remplit, attente `ExtraHealth(handle, n)`.
2. **I-frames après un coup** : `hit_iframes` posé sur le personnage de `isaac` (valeur en données).
3. **Sous au sol** : entrée de pool `pick: Currency(n)` (fin d'enum `LootPick`) qui pose un `ShellPickup`
   (même entité que les shells de M2-T6) ; récompense de salle et mort d'ennemi peuvent en lâcher.
4. **Bombes et clés au sol** : consommables compteurs ; plafond `max_stack` exécuté (l'objet reste au sol si
   le compteur est plein).
5. **Cœur au sol à pleine vie** : reste au sol (Isaac) ou est ramassé sans effet (comportement actuel de la
   `ration`) : la proposition est « reste au sol » pour les seuls consommables qui le déclarent
   (`refuse_when_full: true`) ; le dire au rapport.
6. **Partage en coop** selon Q4 : par joueur (comportement actuel) ou ressource d'équipe neutre ; dans les
   deux cas, ordre de ramassage par `GgrsNetId`, scénario à deux joueurs.
7. **Scénarios** : `soul_heart_absorbs`, `hit_iframes_isaac`, `coin_pickup`, `bomb_pickup_cap`,
   `heart_full_stays`, `coop_pickups_shared` (ou `_per_player`).

## Règles

Champs et variantes en fin, neutres ; les jeux existants ne bougent pas. Preuve §5 si une trace bouge.

## Hors périmètre

Bombes posées (T4), prix en vie max (T8), HUD des cœurs (T16).

## Vérification

README §4 ; rapport `docs/taches/rapports/m4-t3-coeurs-pickups.md`, conventions §68.

## Atteignabilité

Récompense de salle `combat` et mort d'ennemis de `isaac` (tables de T11d) ; boutique exigée par la grammaire.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker :
le dire au rapport. Pousser la branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et
rejouer la suite juste avant le commit `LIVRÉ m4-t3-coeurs-pickups`, qui contient le rapport.
