# Rapport — m1-v3-bots-reanimation (prudent/fonceur relèvent un coéquipier à terre)

Branche partie de `m1-v3-bots-portail` (d7a358e), `origin/main` 61ac539 mergé (bots-portail et
ses traces bénies, `throne_duo_defaite` scripté).

## État en cours

Vérifié, livré. Suite complète sur la branche + main 61ac539 : `make test_scenarios` vert sauf la
trace de `throne_three_floors` (seule, à bénir, voir plus bas) ; tests des crates verts (même
unique échec : `scenarios`) ; `make lint`, `cargo fmt --check` (après `cargo fmt`), interdits,
enregistrement rollback, `make gen` testbed/zombies/throne (aucun fichier modifié), `cargo check
-p throne`, exemples : verts. Target purgé.

## Fait

- Règle de `chasseur`/`acheteur` (`crate::hunter`) reprise pour `prudent`/`fonceur` : un joueur à
  terre porte un `Interactable` `Revive` ; sans ennemi visible à moins de 150 px
  (`REVIVE_SAFE_DISTANCE`), le bot debout va vers le plus proche (rectangle du collider, puis
  `GgrsNetId`) par le chemin (`BotNavigation::approach`, portée − 8) ; à portée, Interaction tenue
  seulement si le jeu sélectionnerait bien cette surface (la plus proche à portée), sinon il
  s'approche encore (`input::revive_step`). En `Floors` comme en vagues (la navigation n'est
  calculée en vagues que quand quelqu'un est à terre).
- **Seulement en urgence** : un coéquipier n'est visé que quand son saignement finit dans moins de
  600 frames (`REVIVE_URGENT_FRAMES`, `input::revive_urgent(bleedout_at_frame, frame)`). Première
  version (relever dès la chute) : `clone_quad` (scénario de référence M0, attente « vague 5, 13
  kills à f4372 ») tombait sous la vague 5 — le bot qui relève arrête de tuer pendant que la vague
  presse. Avec l'urgence, `clone_quad` est **inchangé** (attentes et trace).
- `decide_prudent`/`decide_fonceur` : `BotView::revive` passe avant la chasse et la garde de
  position (l'esquive de `prudent` reste prioritaire) ; visée et tir sur l'ennemi le plus proche
  conservés pendant la réanimation.
- Tests : `prudent_et_fonceur_relevent_un_coequipier`, `reanimation_seulement_en_urgence`.

## Mesure (alacod-sim avec fin de run D43/D44)

`alacod-sim --game throne --bots N --profiles prudent×N --floors run --seeds 1..20 --until-floor 3
--max-frames 15000`, « avant » = d7a358e + main 4e8fe93, « après » = cette branche + main 4e8fe93.
**Mesures prises sur main 4e8fe93** (avant le merge de 61ac539) ; orch : elles restent valables
comme avant/après (61ac539 n'apporte que les traces bénies de bots-portail et un scénario
scripté, pas de changement de bot ni de simulation).

| bots | étage 3 avant | étage 3 après | morts avant → après | soft-locks |
|------|---------------|---------------|---------------------|------------|
| 2    | 13/20         | **14/20**     | 13 → 12             | 0 → 0      |
| 4    | 20/20         | 20/20         | 0 → 1 (graine 1)    | 0 → 0      |

- 2 bots : la graine 1 passe de défaite (f4339) à étage 3 ; les 6 autres défaites (graines 2, 3,
  6, 15, 16, 17) restent des défaites aux mêmes frames.
- 4 bots : seules les graines 1 (f3814 → f3971, un mort de plus) et 5 (f3677 → f4006) bougent ;
  frames moyennes 3790 → 3815. Effet neutre à quatre.

## Scénarios et traces

`make test_scenarios` (branche + main 61ac539) : **une seule trace bouge, `throne_three_floors`**,
attentes vertes (remesurées : joueur 0 à terre f2799, relevé par le joueur 1 à f4264, étage 3 à
f4874, les deux vivants ; avant : mort par saignement à f4599).

- Trace : différente de la référence à partir de la **ligne 4004 (f4003)** :
  `attendu 4003 …9eb8194ea39866e7 139, obtenu 4003 …088519debbd00801 139`.
- Preuve par inputs (`alacod-sim --game throne --bots 2 --profiles prudent,prudent --floors run
  --seeds 4..4 --max-frames 5000 --save-scenario`, binaires avant/après) : premier input différent
  du **joueur 1 à f3998** (`Up, Left, Fire` → `Up, Right, Fire`),
  soit 600 frames avant la fin du saignement du joueur 0 (f4599) — l'urgence s'ouvre ;
  joueur 0 : premier input différent à f4174. Aucun input ne change avant f3998 ; la trace suit
  5 frames plus tard (f4003).
- **Ne bougent pas** : `clone_quad` (vague 5, 13 kills à f4372, attente M0 inchangée, trace
  identique), `bots_four_mixed`, `throne_duo_defaite` (scripté depuis 61ac539), et tous les autres.

Trace de `throne_three_floors` à bénir par orch.
