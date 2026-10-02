# m0-v8 — Carte réelle du clone zombies (voie V6, assets seulement)

Branche : `m0-v8-carte-zombies` · Voie V6 (`docs/taches.md` §11) · Aucun code d'engine.

## Objectif

Donner au clone zombies une **vraie carte de jeu** (8 à 12 salles, boucle classique :
salle de spawn → branches avec portes de coûts croissants → salle ouverte « à tenir »
avec fenêtres → salle Juggernog → armes murales aux points de passage), assemblée à
partir de **tilesets libres**. Le layout s'inspire librement des cartes zombies
classiques (structure type « Nacht der Untoten ») — **pour tests, démos et preuve de
concept uniquement**. Les cartes finales de 1837 seront dessinées par William et ne
reprendront pas ces layouts ; les assets, eux, doivent rester entièrement libres
(règle §11 : rien qui vienne des jeux de référence).

## Contraintes (règles absolues)

- **Assets seulement.** Aucune modification de code d'engine, aucun `.trace`, aucune
  modification de `docs/taches.md`.
- **Nouveaux fichiers uniquement**, au futur emplacement : carte dans
  `games/zombies/assets/maps/`, configs de spritesheets et tilesets à côté des
  existantes. Deux exceptions additives, explicitement autorisées :
  - créer `games/zombies/assets/assets.yaml` (registre des licences, voir §11) et y
    enregistrer **aussi** les packs déjà présents (dont `ZombieShooter` : identifié,
    source réelle, licence, modifications) ;
  - ajouter dans `games/zombies/assets/game.ron` l'entrée additive
    `(path: "maps", kind: "Map")` — rien d'autre dans ce fichier.
- **Interdit** : changer `start_map` (l'orchestrateur basculera la carte par défaut
  après validation), modifier un fichier existant autrement que ci-dessus, utiliser
  un asset sans licence enregistrée, un fichier de plusieurs mégaoctets sans en
  parler.
- **Conventions** : `docs/conventions.md` §1 (cartes LDtk, entités et champs — la
  porte s'écrit `price`/`electrify`, le reste est posé par le générateur), §2
  (planches et animations). LDtk 1.5.3, tuiles de 16 px, niveau unique comme
  `test_map.ldtk` (pas de multi-niveaux).
- **Tileset** : réutilise d'abord le tileset déjà câblé de `test_map.ldtk` (zéro
  risque de câblage). Un tileset libre supplémentaire (CC0, ex. Kenney) est bienvenu
  pour la variété, à condition d'être câblé selon §2 et vérifié par `map_preview`.

## Étapes

1. **Licences** : créer `assets.yaml`, enregistrer les packs existants puis les
   nouveaux (id, fichier, source avec l'URL réellement consultée, auteur, licence,
   modifications).
2. **Layout** : concevoir la carte en JSON LDtk (même structure que
   `exemples/test_map.ldtk` : couches `Walls`, `Entities` ; entités
   `DoorHorizontal/Vertical` avec prix croissants, fenêtres, `PlayerSpawn` index
   0..3, `ZombieSpawn`, `WeaponLocation` ×3-4, `SodaLocation` ×1-2, `CrateLocation`).
   Toutes les salles doivent être joignables, les prix croître du spawn vers les
   salles éloignées, la salle à tenir avoir 3-4 fenêtres.
3. **Câblage** : tileset + spritesheets ; `make map_preview` rend la carte.
4. **Validation locale** : `make lint` (la carte passe le lint du manifeste),
   captures de `map_preview`, puis un scénario de démo sur la carte
   (`map: "maps/<nom>.ldtk"`, 1 joueur, ~2000 frames, attente `WaveAtLeast(wave: 5)`)
   **livré sans trace** — l'orchestrateur blesse à la vérification.
5. **Rapport** (`docs/taches/rapports/m0-v8-carte-zombies.md`, format README §7).

## Acceptation (vérifiée par l'orchestrateur sur l'état fusionné)

- `make lint` vert ; `assets.yaml` complet (existant + nouveau).
- `map_preview` rend la carte (captures dans le rapport).
- Le scénario de démo atteint la vague 5 (trace blessée par l'orchestrateur).
- Après ajout de `--map` à `alacod-sim` (orchestrateur, hors de cette tâche) :
  20 graines sans desync ni softlock sur la nouvelle carte (le pathfinding des bots
  de m0-v7 en est le banc d'essai).
- Aucune trace existante modifiée ; les 61 scénarios restent verts.

## Hors périmètre

Bascule de `start_map`, `--map` d'`alacod-sim`, vidéos officielles, équilibrage des
vagues sur la nouvelle carte : à l'orchestrateur / aux tâches suivantes.
