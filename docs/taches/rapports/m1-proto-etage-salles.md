# Note m1-proto-etage-salles — un étage de salles verrouillées : quelle architecture pour M2 ?

**Prototype jetable** (branche `m1-proto-etage-salles`, base `0ba6040`), b0, une journée. Question
(plan M2, `docs/taches/m2-plan-brouillon.md` §6) : **(A)** une salle = un emplacement du mode
`Floors`, ou **(B)** un étage = un monde LDtk assemblé de salles. Seule cette note compte ; le
code du prototype n'est pas à merger.

## Recommandation

**Voie B**, sans hésitation. Elle s'appuie sur ce qui existe déjà (l'assembleur `Basic`, les
`RoomBounds`/`LevelId` par salle, les portes jumelles, le lien porte ⇄ champ de flux) ; le
prototype a obtenu un cycle complet **verrouillage → nettoyage → réouverture** en **193 lignes**
sans toucher à `Floors`, aux net ids ni au chargement, et sans déplacer aucune trace existante.
La voie A demanderait de réécrire `Floors` (graphe, retour en arrière, persistance de l'état des
salles quittées, portes-téléports) pour un résultat moins fidèle à Gungeon : **~10 à 12 j
estimés** contre **~4 à 5 j** pour E1 sur la voie B (prototype compris dans l'estimation), E2
étant nécessaire dans les deux cas.

## 1. Ce qui existe déjà (constat avant prototype)

- **Toute carte LDtk non-caverne passe déjà par l'assembleur `Basic`** (`map_ldtk::loader`,
  `map::generation::imp::basic`) : un gabarit = un niveau LDtk ; les connexions viennent de la
  couche `LevelConnection` ; sortie = un projet LDtk à N niveaux. Cartes multi-salles réelles :
  `avant_poste` (zombies, 9 salles), `testbed/two_rooms_door`, `testbed/corridor`.
- Par salle au chargement (hors rollback) : `RoomComponent { spawn }`, `RoomBounds`, `LevelId`
  (`add_level_components.rs`) ; les portes (rollback, `GgrsNetId`) portent le `LevelId` de leur
  salle et un `paired_door`.
- Une porte avec `Collider` est un mur pour la navigation (`rebuild_blocked_cells`, reconstruit
  à chaque changement) : **verrouiller/rouvrir = poser/retirer un collider**, le champ de flux
  suit tout seul.
- `Floors` : séquence linéaire de mondes préchargés (un monde LDtk par emplacement), transition
  par portail quand le compte **global** d'ennemis est nul, `index` croissant seulement.

## 2. Ce que le prototype a fait (voie B)

- **Carte** `testbed/gungeon_proto.ldtk` générée par script (`docs/taches/rapports/m1-proto-etage-salles.make_gungeon_proto.py`)
  depuis `two_rooms_door` : départ, combat (2 ennemis), récompense, et un **champ de niveau
  `room_kind`** — recopié tel quel par l'assembleur (`fieldInstances` du gabarit).
- **Module** `map_ldtk::game::rooms_proto` (170 lignes) + 23 lignes ailleurs :
  `RoomKindTag` (lu au chargement), `DoorShape` (collider gardé pour refermer, statique),
  ressource rollback **neutre** `RoomLocks { rooms: LevelId → Dormant | Locked | Cleared }`,
  `room_lock_system` (`RollbackSystemSet::Run`, après `floor_transition_system` — l'ordre est
  exigé par la détection d'ambiguïté) : portes ouvertes au premier tick ; salle `combat` verrouillée
  quand un joueur debout y est entré et qu'un ennemi y vit (portes de la salle et jumelles) ;
  rouverte quand plus aucun ennemi n'y est.
- **Mesure** (scénario `proto_gungeon_rooms`, synctest) : joueur scripté qui entre et tire ;
  verrouillage à **f93** (4 portes fermées), deux ennemis tués, réouverture à **f494** (4 portes
  ouvertes), `EntityCount(Enemy) = 0`, aucune désynchronisation. `buy_door`,
  `testbed_two_rooms_door_idle`, `testbed_corridor_idle`, `avant_poste_demo` : traces
  identiques (le module ne fait rien sur une carte sans `room_kind`).

## 3. Ce que le prototype a appris (coûts réels de E1 sur la voie B)

| Point | Constat | Coût estimé |
|---|---|---|
| Activation des entités par salle | Annuler la vitesse des ennemis d'une salle dormante **ne suffit pas** : `move_enemies` écrit la position directement ; des `follower` ont foncé par la porte ouverte et ont été tués depuis la salle de départ (un bot `prudent` tire à travers la porte sans entrer) | IA : sauter sélection et déplacement pour un ennemi d'une salle dormante (marqueur neutre posé par la salle, comme `MeleeHold`) — **1 j** |
| Fermer une porte sur un occupant | Deux ennemis arrivés dans l'embrasure s'y sont retrouvés **coincés** par le collider refermé | Ne fermer qu'une porte libre (sinon différer d'une frame) ou repousser — **0,5 j** |
| Types de salles | `room_kind` en champ de niveau suffit ; à porter au registre/lint (`RoomTemplate`) | **0,5 j** |
| État et attentes | `RoomLocks` neutre suffit ; attente `RoomState` et moment clé `room` | **0,5 j** |
| Récompense à la sortie, `OnRoomClear` | non fait (C4/C1 v2) | dans T2.3/T2.5 |
| Coop | verrouillage sur **un** joueur présent : un coéquipier resté dehors est séparé (Gungeon téléporte le coéquipier) | **0,5 j** (téléporter les absents à l'entrée) |

**E1 sur la voie B : ~4 à 5 j** (le plan disait 5 j), prototype réutilisable comme point de départ.

## 4. Défauts de l'assembleur trouvés (à corriger dans E2, quelle que soit la voie)

- **Choix « aléatoire » toujours le dernier** : `basic.rs` choisit le gabarit compatible et la
  connexion libre par `.skip(r).last()`, qui renvoie **toujours le dernier élément** quel que soit
  `r` : la génération n'est pas aléatoire (le prototype a dû rendre la chaîne univoque par des
  connexions décalées). Corriger change toutes les cartes générées (`avant_poste`, salles du
  testbed) : traces à rebénir avec preuve.
- `Room::is_overlapping` existe mais **n'est jamais appelé** : des salles peuvent se chevaucher.
- `context.rs` (≈ l. 200) : `continue` sans incrément si un gabarit `Spawn` n'est pas le premier
  du fichier — boucle infinie probable.
- Les salles sont renommées `Level_{i}` (l'identifiant du gabarit est perdu) ; les champs de
  niveau, eux, sont gardés (c'est ce qui porte `room_kind`).

E2 (grammaire d'étage) part donc de cet assembleur : contraintes de types (départ, boss au bout,
boutique, coffre, secret), distances, chevauchement, minicarte, transition d'étage — **~8 j** comme
au plan, plus **~1 j** pour les trois défauts ci-dessus.

## 5. Voie A estimée (non prototypée)

Ce qu'il faudrait à `Floors` pour faire un étage Gungeon d'une salle par emplacement :

| Changement | Pourquoi | Estimation |
|---|---|---|
| Graphe d'emplacements au lieu d'une séquence | portes vers plusieurs salles, retour en arrière (`index` croissant aujourd'hui) | 2 j |
| Portes = téléports avec point d'entrée | pas de continuité spatiale entre mondes empilés à la même origine | 1,5 j |
| Persistance de l'état d'une salle quittée | la transition **détruit** tout le rollback hors joueurs ; revenir dans une salle nettoyée la recréerait pleine (ennemis, coffres, objets au sol) | 3 j |
| Net ids | chaque entrée re-numérote ce qui est recréé (`GgrsNetIdFactory` croît) : traces longues, preuves plus lourdes | inclus, mais coût de preuve permanent |
| Chargement | un monde LDtk préchargé **par salle** (15 gabarits × étages) au lieu d'un par étage | 0,5 j + mémoire |
| Compte d'ennemis par salle | le portail compte les ennemis **globalement** | 0,5 j |
| Présentation | fondu et recentrage à chaque porte (là où Gungeon montre la salle voisine) | 1 j |

**~10 à 12 j**, pour un étage qui ressemble moins à Gungeon (pas de vue sur la salle voisine, pas de
balles qui traversent une porte ouverte) ; E2 resterait à faire en plus.

## 6. Ce qui reste incertain

- **Performance** : un étage assemblé de 15 à 20 salles charge tous les murs et toutes les entités
  d'un coup ; le prototype (3 petites salles) ne le mesure pas. À vérifier dès T2.10 avec le bench
  à 500 balles (une grille spatiale et un champ de flux sur l'étage entier).
- **Champ de flux** : un seul champ sur tout l'étage ; avec des portes fermées, les ennemis d'une
  salle verrouillée n'ont de chemin que dans leur salle — c'est voulu, mais le coût de
  reconstruction à chaque verrouillage n'est pas mesuré sur un grand étage.
- **Transition d'étage** (étage → étage suivant) : reste `Floors` (un monde assemblé par
  emplacement) ; c'est l'usage déjà prouvé par throne (cavernes), sans changement.
