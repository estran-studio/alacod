# plan-assets — plan des assets des trois clones (sons, sprites, tilesets, polices) sous licence libre

Décision de William (2026-10-14, dette **D32**) : le son de tir n'est qu'un symptôme ; il faut un
**plan** et une session dédiée aux assets. Tâche de **recherche et de plan** d'abord (adaptée à une
session cloud : peu de compilation), puis un premier lot importé.

Lire d'abord `CLAUDE.md`, `docs/conventions.md` (sections sprites, HUD §15, feedback §31, sons),
les trois registres `games/{zombies,throne,gungeon}/assets/assets.yaml` (format, licences,
`statut: placeholder`), `docs/plan-engine.md` (voie V6, portage console §7 : licences compatibles
avec une sortie commerciale et console) et `crates/game/src/feedback.rs` (sons, palliatif
`SHOT_SOUND_MAX`). Branche `plan-assets` depuis `main`.

## Partie 1 — inventaire et besoins (livrable : `docs/plan-assets.md`)

1. Inventaire de ce qui existe par jeu : fichier, usage réel dans le code/RON, licence, statut
   (placeholder ou définitif), problèmes connus (D32 : tir soutenu de 17,7 s coupé en fondu ; sprites
   partagés entre jeux ; ennemis et boss de `gungeon` sans sprite).
2. Besoins par jeu, classés par priorité de jeu (ce qui gêne le plus la lecture d'une partie
   d'abord) : sons (tir par arme, impact, rechargement, dash, blank, coffre, mort, musique de salle
   et de boss), sprites (joueur, ennemis, boss, balles, objets, coffres, props de M2-T11), tilesets,
   UI/HUD, polices.
3. Contraintes de licence : seulement **CC0** ou **CC BY** (attribution tenue dans `assets.yaml`) ;
   refuser NC, ND, « usage personnel », licences de packs payants non redistribuables ; noter ce qui
   resterait compatible avec une sortie console.
4. Sources candidates évaluées (Kenney, OpenGameArt, itch.io en CC0/CC BY, freesound CC0/CC BY,
   Sonniss GDC si la licence le permet, polices OFL) : pour chaque besoin, 1 à 3 candidats avec lien,
   licence vérifiée sur la page de l'auteur, style (cohérence pixel-art 16/32 px des tilesets actuels).
5. Plan de remplacement en lots (un lot = un jeu ou une famille d'assets, livrable seul), avec ce
   qu'il faut changer côté code/RON (noms de fichiers, atlas, `assets.yaml`, conventions).

## Partie 2 — premier lot : sons de tir (ferme D32)

Un échantillon de **coup unique** par famille d'arme (pistolet, fusil à pompe, mitraillette au
moins), sous CC0/CC BY, normalisé (format `.ogg`, volume), déclaré dans `assets.yaml` ; `feedback.ron`
des jeux concernés pointe dessus ; retrait du palliatif `SHOT_SOUND_MAX` s'il n'a plus de raison
d'être. Présentation seulement : aucune trace ne bouge (`make test_scenarios` vert sans bless).

## Règles

Ne jamais committer un asset dont la licence n'est pas vérifiée et écrite dans `assets.yaml` (source,
auteur, licence avec lien, modifications). Pas de fichier binaire au-delà du premier lot dans cette
tâche. Rapport `docs/taches/rapports/plan-assets.md` ; finir par un commit
`LIVRÉ plan-assets` (session cloud : pousser régulièrement, ne jamais pousser sur main).
