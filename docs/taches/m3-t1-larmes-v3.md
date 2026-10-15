# m3-t1-larmes-v3 — larmes v3 : modificateurs d'objets empilés, Spectral, Split, Orbit, statuts v2 (M3, V1a, B5 v3)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §16, §19
(statuts), §41, §60 (contrats de M3-T0a) et le rapport `docs/taches/rapports/m4-t0a-contrats-larmes.md`.
Branche `m3-t1-larmes-v3` depuis `origin/main`, **après le merge de M3-T0a**. Estimation : 5 j. Section de
conventions réservée : **§66**.

## Contexte

M3-T0a a posé `ProjectileMods` sur les objets, `compose` et ses règles, `ShotSpeed`/`ShotSize`, `Luck` et les
contrats `Spectral`, `Split`, `Orbit`. Rien n'est encore lu au tir.

## À faire

1. **Tir composé** : au tir d'un joueur (et d'un familier qui hérite des larmes, si Q13 le retient), les
   modificateurs de l'arme et des `projectile` de ses objets et synergies sont composés par `compose` ;
   `on_hit` et `on_expire` des objets s'ajoutent à ceux de l'arme (ordre de §60). Une balle « ordinaire »
   (sans `Projectile`) devient composable dès qu'un objet ajoute quelque chose ; sans objet, chemin d'origine
   intact (traces inchangées).
2. **`Spectral`, `Split`, `Orbit`** exécutés (`projectile_collision_system`, `projectile_expire_system`,
   steering), lint `Unsupported` retiré. `Split` respecte `MAX_GENERATION`.
3. **Statuts v2** : `StatusDef::Poison` (comme `Burn`, `DamageKind::Poison`) et `StatusDef::Fear` (l'ennemi
   fuit sa cible : `Flee` imposé pendant la durée), en fin d'enum. Charme : **Q12** (hors tâche sauf réponse).
4. **Chance d'effet** : `on_hit` d'un objet accepte `chance: Some(base, per_luck)` ; tirage par un RNG local
   de graine `(graine de run, net id du projectile, rang)`, jamais `RngStreams` ; `chance_with_luck` de T0a.
5. **Visée** : 360° (comportement actuel) ; 4 directions façon Isaac seulement si **Q17** le décide (option
   de personnage, pas une règle globale).
6. **Scénarios** (testbed, objets d'essai) : `tears_item_homing`, `tears_stack_two_copies` (deux exemplaires
   d'un objet `Pierce(1)` : la larme traverse deux cibles), `tears_stack_three` (rebond + guidage + division),
   `tears_spectral_wall`, `tears_split`, `tears_orbit`, `status_poison`, `status_fear`, `tears_luck_chance`
   (même graine, chance 0,5 : nombre d'effets fixé par la trace). Unitaires : toutes les paires de
   modificateurs par `compose`.

## Règles

Aucune trace existante ne bouge hors des scénarios nouveaux (tout passe par des objets que le contenu
existant n'a pas) ; sinon preuve §5. Coût : mesurer `bench_bullets_room` avant/après (`make perf_review`).

## Hors périmètre

Contenu `isaac` (T11a), familiers (T6), visuels des larmes (T17), bench dédié (T15).

## Vérification

README §4 ; rapport `docs/taches/rapports/m3-t1-larmes-v3.md`, conventions §66.

## Atteignabilité

Rencontré quand T11a met les objets de larmes dans le pool trésor de `isaac` (une salle de trésor exigée par
étage) ; d'ici là, scénarios testbed seulement : dette nommée au rapport, à fermer par T11a.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker :
p2p rejoué par l'orchestrateur, le dire au rapport. Pousser la branche régulièrement. Voie rapide du README
§6 : fusionner `origin/main` et rejouer la suite juste avant le commit `LIVRÉ m3-t1-larmes-v3`, qui contient
le rapport.
