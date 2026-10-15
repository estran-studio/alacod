# m4-t5-grammaire-grille — étage en grille, salle secrète, trésor à clé, salle post-boss (M4, V1d, E2 v2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1, `docs/conventions.md` §35, §40,
§62 (contrats de M4-T0c), le rapport `docs/taches/rapports/m2-t10-grammaire-etage.md`, et le code
`crates/map/src/generation/floor.rs`, `crates/map/src/generation/imp/floor.rs`,
`crates/map_ldtk/src/generation/to.rs`. Branche `m4-t5-grammaire-grille` depuis `origin/main`, **après le
merge de M4-T0c** et la réponse de William à **Q8** (taille et formes des salles). Estimation : 5 j,
**risque** (planificateur nouveau, boucles). Section de conventions réservée : **§70**.

## Contexte

Le planificateur de M2-T10 fait pousser un arbre de gabarits par connexions alignées. Isaac pose ses salles
sur une grille : toutes de même taille (ou multiples), une porte par côté partagé, des boucles possibles, des
salles spéciales en cul-de-sac et une salle secrète voisine de plusieurs salles.

## À faire

1. **Planificateur `Grid`** (`FloorLayout::Grid { cell, max }`) : croissance depuis la salle de départ, une
   nouvelle case n'a qu'un voisin déjà posé (règle d'Isaac) sauf boucle explicitement permise (donnée
   `loops: u32`, défaut 0) ; boss sur le cul-de-sac le plus loin ; types requis sur des culs-de-sac ; graine
   par emplacement (§40). Unitaires sur 1 000 graines : connexité, types présents, pas de chevauchement,
   aucune porte vers le vide.
2. **Portes** : toute arête entre deux salles posées est une porte ; les ouvertures inutilisées murées
   (`close_unused_openings`). Si le format de plan (`FloorPlan`) suppose un arbre (un seul parent), l'étendre
   à un graphe et garder `Graph` identique (même plan, même hash de disposition pour gungeon).
3. **Salle secrète** (`secrete`, une par étage si requise) : case vide adjacente à au moins deux salles (pas
   au boss), ouvertures `Secret` (T0c) vers ses voisines.
4. **Trésor à clé** : `door_lock: Key(1)` sur le type `tresor` à partir de l'étage donné en données
   (**Q6**).
5. **Salle post-boss** (`diable`) : case optionnelle accrochée au boss, tirée avec une chance en donnée
   (graine de l'emplacement) ; son contenu est T8.
6. **Minicarte** : la minicarte de M2-T18 lit déjà `RoomStates` ; vérifier qu'elle dessine une grille sans
   changement (sinon dette au rapport, T16 la reprend). La salle secrète n'apparaît qu'ouverte.
7. **Contenu `isaac`** : grammaire `sous_sol` en `Grid` (T0f l'avait en `Graph`) ; les traces `isaac`
   bougent : preuve §5, cause dans le commit.
8. **Scénarios** : `grid_floor_seed_{1,23,123}`, `grid_secret_room` (placée, fermée), `grid_treasure_key`,
   `grid_post_boss` (graine où elle apparaît). Perf : un étage de 12 à 15 salles en moins de 2× le temps de
   chargement de `grand` (gungeon).

## Règles

`Graph` inchangé : traces gungeon identiques, sans `BLESS`. Déterminisme du planificateur : `BTreeMap`/tri,
RNG local par emplacement, tentatives bornées.

## Hors périmètre

Contenu de la salle du diable (T8), ouverture du mur secret (T4), curse « étage agrandi » (T9), formes en L.

## Vérification

README §4 ; rapport `docs/taches/rapports/m4-t5-grammaire-grille.md`, conventions §70.

## Atteignabilité

La séquence par défaut de `isaac` (`entry.floors`) passe en `Grid` dans cette tâche : toute partie ordinaire
la rencontre ; secret, trésor à clé, post-boss : graines nommées au rapport.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker :
p2p `isaac` (transition d'étage) rejoué par l'orchestrateur, le dire au rapport. Pousser la branche
régulièrement. Voie rapide du README §6 : fusionner `origin/main` et rejouer la suite juste avant le commit
`LIVRÉ m4-t5-grammaire-grille`, qui contient le rapport.
