# M2-T8 — Formations simples

Branche `m2-t8-formations`. Conventions : `docs/conventions.md` §49.

## Livré

- **Garde-fou M2-T2** (commit séparé, en tête) : un dash `iframes_kind: Projectiles` sans `DashExtra`
  ne retombe plus en silence sur l'invulnérabilité de `Health` (`debug_assert`). `create_character` et le
  runner posent déjà `DashExtra` dès qu'un dash est déclaré ; le garde-fou ferme le cas restant.
- **Règle `Formation(shape: Arc(radius, spread), period, volley)`**, variante en fin d'enum `Behavior`.
- **Escouade dérivée** : `ai.squad`, `SquadSlots` recalculé à chaque frame (ressource hors rollback).
  Aucun composant ni type rollback nouveau → aucun hash ne bouge.
- **Déplacement** vers la place sur l'arc puis `Strafe` léger ; **tir alterné** par fenêtre
  `(frame / period) mod effectif == rang` (`volley` pour le tir groupé).
- **Lint** : 5 refus (rayon, ouverture, période, formation inerte, formation sans escouade) + fixtures.
- **Contenu** : testbed `escouade_tireur` ; gungeon `bullet_kin_squad`, niveau `EscouadeEW` (salle
  `escouade`), grammaire `court_escouade`, séquence `etages_court_escouade`.
- **Scénarios** : `formation_arc`, `formation_member_dies`, `formation_alternate_fire`,
  `formation_two_players`, `gungeon_squad`. Nouvelle attente : `EnemyDistance(.., target: Entity(..))`.

## Décisions

- **Pas de nouvel état rollback** : rang, effectif, place et tour de tir se déduisent de l'état
  (positions, `GgrsNetId`, frame). Le seul état « caché » est une ressource effacée et réécrite avant tout
  retour anticipé, au début du système.
- **Cible de l'escouade** = joueur debout le plus proche de son centre, ex æquo au plus petit `GgrsNetId`
  (fonction pure `nearest`, testée). Écart assumé avec « plus proche par le chemin ».
- **Gungeon** : `bullet_kin` inchangé ; la salle d'escouade est un type de salle à part, tiré par aucune
  grammaire existante, donc les étages existants ne bougent pas.

## Traces

Suite complète `make test_scenarios` sans bless sur la branche : **aucune trace existante n'a bougé** (gungeon
compris, malgré le nouveau niveau LDtk `EscouadeEW`, la grammaire et le personnage ajoutés : tout est neutre).
Seules les traces des 6 scénarios nouveaux sont enregistrées (`formation_*`, `gungeon_squad`). Pas de preuve §5
nécessaire.

## Vérification

`fmt`, `check-forbidden`, `check-rollback-registration`, `make lint` ×4, `make gen` ×4 : verts. `make test_crates` :
662 verts. P2P gungeon à deux clients : traces identiques (599 lignes). `fuzz_inputs` (`ALACOD_FUZZ=0:8`) vert sur
`formation_alternate_fire` et `formation_two_players`.

Note : `escouade_tireur_fragile` (30 PV) sert `formation_member_dies` ; `formation_arc` s'arrête à la frame 180 car
le joueur immobile (100 PV) meurt sous les flèches vers la frame 200.

## Pièges

- `GgrsNetId.0` est un `usize` (pas `u64`) ; `order_iter!` renvoie un `Vec` (pas un itérateur).
- Une fixture de lint doit nommer son fichier (`characters/grunt.ron`) dans `content_folders`, et un
  champ `Option` demande `Some(..)` en RON.
- Les arènes du testbed sont petites (≈ 330×170) : un arc de rayon 120 en sort ; `floor_a.ldtk` n'a
  qu'un civil inoffensif. La salle `EscouadeEW` de gungeon (112 px) impose un arc de 40.

## Hors périmètre

Formations complexes (lignes, encerclement dynamique), communication entre escouades, IA de boss (M2-T7).
