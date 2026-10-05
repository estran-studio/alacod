# Rapport — m1-v3-bots-reanimation (prudent/fonceur relèvent un coéquipier à terre)

Branche partie de `m1-v3-bots-portail` (d7a358e).

## État en cours

Code écrit, **pas encore compilé** (attente du feu vert d'orch). Reste : compilation, tests,
mesure 2 bots × 20 graines avant/après avec le nouvel `alacod-sim` (fin de run, D43/D44),
`make test_scenarios` (scénarios où un bot `prudent`/`fonceur` tombe à terre : recalage + preuve
par inputs, liste explicite — dont `bots_four_mixed`/`clone_quad` en vagues s'ils bougent), suite.

## Fait

- Règle de `chasseur`/`acheteur` (`crate::hunter`) reprise pour `prudent`/`fonceur` : un joueur à
  terre porte un `Interactable` `Revive` ; sans ennemi visible à moins de 150 px
  (`REVIVE_SAFE_DISTANCE`), le bot debout va vers le plus proche (rectangle du collider, puis
  `GgrsNetId`) par le chemin (`BotNavigation::approach`, portée − 8) ; à portée, Interaction tenue
  seulement si le jeu sélectionnerait bien cette surface (la plus proche à portée), sinon il
  s'approche encore (`input::revive_step`). En `Floors` comme en vagues (la navigation n'est
  calculée en vagues que quand quelqu'un est à terre).
- `decide_prudent`/`decide_fonceur` : `BotView::revive` passe avant la chasse et la garde de
  position (l'esquive de `prudent` reste prioritaire) ; visée et tir sur l'ennemi le plus proche
  conservés pendant la réanimation.
- Test : `prudent_et_fonceur_relevent_un_coequipier`.
