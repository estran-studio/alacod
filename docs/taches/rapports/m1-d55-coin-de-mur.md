# Rapport — m1-d55-coin-de-mur (D55 : un ennemi accroché à un coin de mur ne glisse jamais)

Branche partie de `main` 7f9db06 (origin/main ae87ae0 ne change ni `crates` ni `games`).

## Diagnostic (ce que D55 était vraiment)

Graine 19 de throne, 2 bots `prudent`, soft-lock à f3050 (reproduit tel quel sur main, 3050 frames,
inputs sauvegardés par `--save-scenario`). Le rat #247 est en (166, 601), son point visé (192, 601).

1. **Premier défaut : `steering_point`.** Pour la case (11, 37), le mur du dessous (cases (11..12, 36))
   repousse le point de +9 en y, et le coin bloqué en diagonale en haut à gauche (10, 38) le repousse
   de −8 en y : les deux se compensent, le point tombe à y = 601. Le collider du rat (20×20, décalage
   y −6, donc bas à `y − 16`) y recouvre le mur de 7 px. Le point visé est dans le mur : ce n'est pas
   un glissement raté, la cible est inatteignable. Corrigé (voir ci-dessous) mais **insuffisant**.
2. **Cause racine : une chicane.** Entre la case (10, 37) et la case (11, 37), un coin bloqué est en
   haut à gauche (plafond à y = 608 pour x < 176) et l'autre en bas à droite (sol à y = 592 pour
   x ≥ 176). Le corps qui franchit x = 176 doit tenir entre les deux : 16 px (une case) pour 20 px de
   haut. Il est géométriquement impossible de passer, quel que soit le pilotage. Le flow field
   (petits gabarits, couloir de deux cases) traversait pourtant ce pas.
3. **Effet sur la génération.** Une fois le pas exclu du champ, le rat reste dans sa poche, avec son
   spawner : la poche est hors du champ des joueurs. L'accessibilité des points de caverne
   (`world::nav`, D48) doit appliquer la même règle, sinon le spawner naît dans une poche inaccessible
   (c'est ce que j'ai vu en n'ayant corrigé que le champ : soft-lock à f2320, rat « hors de son champ
   de flux »).

D39 (`slide_axes`) n'y change rien : il n'y a rien à glisser, le mouvement voulu est bloqué par
une vraie collision de deux coins.

## Correctif (quatre commits)

- `navigation.rs::steering_point` : la poussée d'un coin diagonal sur un axe est ignorée quand la case
  orthogonale du côté vers lequel elle pousse est bloquée (le mur de ce côté a déjà repoussé le point
  de tout le corps). Tests `steering_corner_tests` (cas de la graine 19 : bas du collider à ≥ 592 ;
  coin seul : poussées inchangées).
- `world::nav::chicane_step(blocked, x, y, dx, dy)` : un pas orthogonal est une chicane si, sur l'axe
  perpendiculaire, la case de départ est bloquée d'un côté et celle d'arrivée du côté opposé. Appliquée
  aux gabarits **petits** (un gabarit grand écarte déjà ces cases par `blocked_for`) :
  - dans l'expansion du flow field (`build_flow_field`) ;
  - dans `nav_distances` (accessibilité des points de caverne), donc dans la génération.
  Le test de parité existant (`champ_de_flux_et_points_de_caverne_meme_accessibilite`, 3 niveaux ×
  20 graines) reste vert : les deux calculs suivent la même règle. Tests : `chicane_entre_deux_coins_opposes`
  (game), `chicane_de_deux_coins_opposes_coupe_la_poche` (world).
- Bless de 9 traces (commit dédié).

Choix : on corrige la **navigation** (ne jamais envoyer un corps où il ne tient pas) et non le
glissement. Alternative écartée : glissement le long du coin quand l'axe voulu est bloqué par un
recouvrement de quelques px : ici le recouvrement vaut 7 px et la sortie par le haut est interdite
par le plafond.

## Preuve §5 (traces)

`make test_scenarios` complet sur la branche avant bless : **9 traces bougent**, tout le reste est
identique (zombies, testbed, clones, vagues, scénarios générés). Dump `.full` contre `origin/main`
(`scripts/trace-diff.py`), un scénario par appel :

| scénario | première frame qui diffère | seul type différent à cette frame |
|---|---|---|
| `bench_cave` | 0 | `FlowFieldCache` (la carte caverne contient une chicane) |
| `explode_wall` | 0 | `FlowFieldCache` |
| `throne_three_floors` | 0 | `FlowFieldCache` |
| `throne_floor_1` | 511 | `FlowFieldCache` (passage à l'étage suivant : champ recalculé) |
| `throne_progression` | 511 | même hash que `throne_floor_1` (même carte jusqu'à f511) |
| `throne_solo` | 511 | idem |
| `throne_quad` | 440 | `FlowFieldCache` |
| `throne_mutation_choice` | 935 | `FlowFieldCache` |
| `throne_softlock_recul` | 1845 | `FlowFieldCache` |

À chaque première frame divergente, seul le `FlowFieldCache` diffère (le champ perd les pas de
chicane) ; les autres écarts des frames suivantes en découlent (trajectoires). Pas de nouveau type
rollback (rien dans la parité des types vides). `throne_progression` et `throne_solo` n'ont pas été
dumpés : même trace de référence que `throne_floor_1` jusqu'à f511, même hash obtenu. Blessées sur
cette branche (commit « bless : 9 traces ») ; à refaire par l'orchestrateur au merge.

## Mesures (`alacod-sim`, binaires avant = origin/main, après = branche)

| | avant | après |
|---|---|---|
| throne 2 bots `prudent`, graines 1..20, étage 3 | 19/20 | **20/20** |
| soft-locks / desync | 1 (graine 19, f3050) / 0 | **0 / 0** |
| graine 19 | soft-lock étage 1, f3050 | étage 3 à **f3536** |
| autres graines throne | — | frames et morts identiques graine par graine |
| zombies 4 acheteurs, 1..20, vague 5 | 20/20, 0 desync, 0 soft-lock | **identique graine par graine** (frames, kills) |

Attention : le correctif de bots (`m1-bots-apres-movement-feel`) n'est pas dans cette base ; la graine 19
est donc débloquée par le moteur seul (la poche n'existe plus), sans le correctif de b1.
Un premier jeu de mesures « après » était faux (le binaire copié avait été recompilé par le build de
`main-wt` dans le même target) ; refait avec le binaire de la branche.

## Vérification (§4)

- `make test_scenarios` : exécuté en entier avant bless (≈ 113 min, lent en machine partagée), 9 échecs
  de trace uniquement ; les 9 ont été blessés un par un (`BLESS=1 SCENARIO=<nom>`, code de sortie 0).
  Après bless, le test `scenarios` (tous les scénarios, traces comparées) est vert dans la suite
  `cargo test` ci-dessous (« 3 passed » en 5 635 s).
- `cargo test -p scenario -p run -p combat -p game -p content -p map_ldtk -p sim_core -p stats -p bots
  -p effects -p world --no-fail-fast` (profil headless) : **0 échec** sur tous les binaires de test
  (dont `expectations` 45, `world` 16, `game` 103, `content` 82+84, `scenarios` 3 passed / 7 ignorés).
  Un premier passage avait été corrompu par ma propre purge du target (rlibs supprimées en cours de
  test) ; c'est le second, propre, qui compte.
- `make lint` : « aucune erreur » (games/throne ; zombies et testbed aussi).
- `cargo fmt --all -- --check` : rien. `check-forbidden.sh` : 4 avertissements préexistants.
  `check-rollback-registration.sh` : OK.

## Non fait / non vérifié

- p2p à deux clients (Docker) et bench au calme : non faits.
- Scénario figé « ennemi + coin de mur » : pas de scénario permanent. Les inputs de la graine 19 (3 050
  frames, ~5 min) sont liés à l'ancienne carte (la carte et la poche changent avec le correctif) ; le
  verrou de non-régression est porté par les tests unitaires (`steering_corner_tests`,
  `chicane_*`) et par le test de parité champ de flux / caverne. Le repro figé avant correctif est
  `alacod-sim --game throne --bots 2 --profiles prudent,prudent --floors run --seeds 19..19 --until-floor 3
  --max-frames 12000 --save-scenario <dir>` sur main : fin « soft-lock » à f3050.
- `throne_progression` et `throne_solo` : pas de dump `.full` (voir plus haut).

## Dettes

- Un corps de 20 px de haut qui doit passer entre un coin de plafond et un coin de sol espacés d'une
  case est maintenant évité par le champ ; les ennemis plus petits (≤ 16 px) ne seraient pas concernés
  par la règle mais il n'y en a pas aujourd'hui (`SMALL_AGENT_MAX = 20`). Si un gabarit ≤ 16 px apparaît,
  `chicane_step` devra dépendre du gabarit (aujourd'hui : tous les « petits »).
- Défauts de l'assembleur de salles (E2) : cette chicane vient du générateur de cavernes, pas des salles.
