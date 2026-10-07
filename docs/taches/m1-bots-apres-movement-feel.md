# m1-bots-apres-movement-feel — bots réglés pour la nouvelle course (graine 19, approche du portail)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b1, branche `m1-bots-apres-movement-feel` créée
depuis `origin/m1-fusion-revue-m0-suite`, puis rebasée sur `main` dès que la fusion y est). Voie V3 bots, 1 j.
Compilations : `CARGO_BUILD_JOBS=2`, `df -h /home` ≥ 20 Go avant chaque compilation, une sim et un dump à la fois.

## Constat (b0, m1-fusion-revue-m0-suite, 2026-10-07)

`movement-feel` passe l'accélération des joueurs de 150 à 3000 px/s² (course nerveuse, dash à i-frames). Mesures de b0,
même méthode avant (main `836d3cf`) / après (fusion) :
- `zombies` 20 graines, 4 acheteurs : 20/20 → 20/20, plus rapide ;
- `throne` 4 bots : 20/20 → 20/20 ;
- `throne` 2 bots : 20/20 → **19/20, un soft-lock** (graine 19 : bloquée à l'étage 2 jusqu'à f3050 ; relevé D42 : dernier
  rat à 29,5 PV caché à 173 px des bots, « point visé par le flow field qui chevauche un mur physique avec son
  collider » ; sur main la même graine finit à f4321), et 3 graines avec un mort (5, 8, 10) au lieu de 0.
- Dans les scénarios à bots, **les portails s'ouvrent aux mêmes frames mais les bots y entrent 120 à 190 frames plus tard**
  : l'approche du portail (`decide::approach_portal`, `steer`, vitesse de croisière) était réglée pour l'ancienne course.

## Décisions (fixées ici)

1. **Graine 19 d'abord** : rejouer, dater, dire pourquoi les bots ne vont pas chercher le rat et pourquoi le rat est
   coincé (ennemi contre un mur hors de vue : `step_blocked_by` libère un ennemi incrusté, mais un ennemi qui vise un point
   inatteignable reste-t-il sur place ?). Côté bots (`crates/bots`) ou côté moteur : corriger ce qui est prouvé ; un défaut
   moteur hors bots → dette avec la graine, pas de correctif sans accord d'orch.
2. **Pilotage des bots** : recaler pour la nouvelle course les constantes de `steer`/`approach_portal` (vitesse de croisière,
   zone de freinage) et la poursuite d'un ennemi caché, pour que les bots n'oscillent ni ne dépassent leur cible avec une
   accélération 20 fois plus forte. Tests unitaires mis à jour.
3. **Mesure avant/après** sur l'état fusionné : `throne` 20 graines à 2 et 4 bots, `zombies` 20 graines à 4 acheteurs ;
   critère : 0 soft-lock, défaites non augmentées par rapport à main d'avant la fusion (0 sur 1..20 à 2 bots), zombies 20/20.
4. Traces des scénarios à bots qui bougent : preuve par inputs ; `clone_*` doivent garder leurs attentes M0.

## Livrer

Suite complète, crates, lint ×3, fmt, scripts, `make gen` ×3, exemples ; rapport `docs/taches/rapports/m1-bots-apres-movement-feel.md` ;
`git push -u origin m1-bots-apres-movement-feel` ; `SendMessage` à `orch [f1df5b]` : `LIVRÉ m1-bots-apres-movement-feel <sha> :
<graine 19, portail, mesures avant → après>`.
