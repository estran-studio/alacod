# m0-revue-suite-contenu — les points de la revue M0 sur `zombies` (session de William)

Prompt de départ à coller tel quel dans Claude Code, sur l'ordinateur de William (macOS), à la
racine d'un clone de `estran-studio/alacod`. Travail **en parallèle de M2** (sessions b0/b1 sur la
machine `orca`, orchestrateur `alacod-orch-39`) : ne touche pas au moteur de M2 (salles, objets).

---

Tu travailles avec William sur les suites de sa revue humaine de M0 : le clone `zombies` (Call of
Duty Zombies) doit devenir agréable à jouer. Ce sont surtout des réglages de **contenu et de
présentation** ; William décide du ressenti, toi tu proposes des valeurs, tu mesures, tu
prépares les parties à jouer, tu tiens le carnet. Réponds en français.

## 0. Mise en place

```bash
git fetch origin
git checkout -b m0-revue-suite-contenu origin/main
```
Rust nightly (`rust-toolchain.toml`). Première compilation à froid : 20 à 40 min. Pour jouer :
`cargo run -p zombies --profile headless` (avec rendu, profil optimisé). Pour enregistrer une
partie : `ALACOD_RECORD=$PWD/docs/captures/revue-m0/<nom>.ron APP_VERSION=x cargo run -p zombies
--profile headless` (écrit à la fermeture de la fenêtre depuis R7). Touches (R6) : WASD, **H**
interagir, **C** dash, Tab arme suivante, R recharger, F mêlée, Shift gauche sprint, clic tirer.

## 1. Lire, dans cet ordre

1. `CLAUDE.md` : règles de déterminisme (obligatoires dès qu'on touche la simulation) et frontière
   simulation/présentation (le son et l'UI vont derrière `PresentationPlugin`).
2. `docs/taches/README.md` : §1 « Variante cloud » (clone seul, pas de meta-repo ni de worktree),
   §4 vérification, §5 preuve des traces, §6 commits, §7 rapport.
3. `docs/digests/revue-m0.md` : le carnet de la revue, en particulier « Bilan de la revue solo ».
   C'est le fil conducteur ; chaque changement y renvoie à sa note (S4, S5…).
4. `docs/taches/dettes.md` (dettes connues, à ne pas re-signaler) et `docs/conventions.md` pour
   les sections citées plus bas.

## 2. Les points, dans l'ordre conseillé

Chaque point : proposer à William une ou deux options chiffrées, appliquer celle qu'il choisit,
lui faire jouer (enregistrement si utile), noter son retour dans le carnet.

1. **S5 — vagues trop dures au départ** : `games/zombies/assets/waves/wave_config.ron` (nombre
   de zombies par vague, cadence d'apparition, santé/vitesse, multiplicateurs par vague ; D29 :
   le multiplicateur de dégâts par vague n'est pas appliqué partout, à vérifier avant de s'y
   fier). Mesure : vague 1 vidée en ~22 s, vague 2 en ~30 s (enregistrement `revue_mouvement`).
   Mesurer avant/après avec `alacod-sim` (4 `acheteur`, graines 1..20, jusqu'à la vague 5) et
   donner le tableau (frames par vague, morts, dégâts encaissés).
2. **S4 — armes** : fusil à pompe « super fort », pistolet inutile, mitraillette forte mais trop
   imprécise (spread). Armes dans `games/zombies/assets/ZombieShooter/Sprites/Character/weapons.ron`
   (manifeste `game.ron`, kind `Weapon`) ; dispersion appliquée depuis D51 (conventions §16/§29).
   Après tout changement d'arme : `make gen GAME=zombies` (scénarios générés par arme).
3. **S7 — tirs trop forts** et **S6 — pas d'indication sonore de vague** : présentation seule
   (aucune trace ne doit bouger). Sons dans `games/zombies/assets/ui/feedback.ron` (`sounds`,
   `crates/content/src/feedback.rs`, joués par `crates/game/src/feedback.rs`) : ajouter un volume
   par son (champ RON, lint) ; son de début et de fin de vague **dérivé de l'état** (`WaveState`
   lu dans `Update`, jamais un événement émis par la simulation). D32 : l'échantillon de tir est
   un enregistrement de tir soutenu ; s'il faut de nouveaux fichiers son, les déclarer dans
   `games/zombies/assets/assets.yaml` (sources et licences) et demander à William.
4. **S2/S8 — une meilleure carte** : `games/zombies/assets/maps/avant_poste.ldtk` (gabarits de
   salles assemblés par la graine ; R8 : graine fixe 123456 → toujours les mêmes 4 salles sur 9).
   Constat : les 5 `ZombieSpawn` sont tous dans `Depart`. Les spawners sont déjà activés par salle
   (`crates/game/src/character/enemy/spawning.rs` : salle où se trouvent les joueurs) : ajouter
   des spawners et des fenêtres aux autres salles, revoir la disposition avec William (LDtk ou
   script, conventions §1). Vérifier ensuite `make test_scenarios` (`avant_poste_demo`,
   `revue_murs_avant_poste`…) et `alacod-sim` sur 20 graines `avant_poste`.
5. **R10 — « impossible d'acheter un soda »** : non reproduit (aucune machine dans les salles
   tirées par la graine 123456). Demander à William une capture ou la partie concernée ; ne rien
   corriger sans repro.
6. **Décisions à demander à William, à ne faire que s'il dit oui** : R2 (`make record_session`
   en profil `headless`), R3 (message clair si `ALACOD_HEADLESS=1` sur un binaire avec rendu),
   R6 (touches : choix et doc ; flèche droite liée deux fois), R8 (graine tirée au hasard par
   partie hors simulation, sauf `--seed`). **R9 (bits de poids fort du RNG) est hors de cette
   branche** : il déplace toutes les traces, c'est une tâche moteur pour l'orchestrateur.

## 3. Traces et critères M0

- Les réglages de vagues, d'armes et de carte **changent des traces** (`clone_*`,
  `equilibrage_*`, `bots_four_mixed`, scénarios générés…). C'est voulu : preuve README §5 (dump
  `main` contre branche, `scripts/trace-diff.py`, chaque différence expliquée par le réglage),
  puis `BLESS=1 make test_scenarios`, avec la justification dans le commit (« changement de
  gameplay voulu : S5 … »). Un commit de réglage par note, un commit de bless séparé.
- Les attentes des scénarios du clone peuvent devoir être recalées (méthode conventions §34) ;
  le critère M0 reste : bots qui atteignent la vague 5 sur 20 graines, 0 desync, 0 soft-lock.
- Le son et le HUD ne doivent déplacer **aucune** trace : s'ils le font, c'est un bug.

## 4. Règles

Fixed (jamais `f32` dans `GgrsSchedule`), `order_iter!`, `BTreeMap`, `despawn_rollback`,
`FrameEvents`, présentation dérivée de l'état. Pas de `git stash`. Pas de push sur `main`, pas de
modification de `docs/taches.md` ni du tableau de `dettes.md` (l'orchestrateur les tient). Rien
n'est « vérifié » sans avoir tourné devant toi, avec les chiffres.

## 5. Livrer

Quand William le décide (tout ou une partie des points) :
1. Vérification standard (README §4 : `make test_scenarios`, tests des crates, `make lint`,
   `cargo fmt --all -- --check`, `./scripts/check-forbidden.sh`,
   `./scripts/check-rollback-registration.sh`, `make gen` des trois jeux sans modification
   inattendue) ; p2p et bench : « non vérifiés ici » (l'orchestrateur les rejoue).
2. Rapport `docs/taches/rapports/m0-revue-suite-contenu.md` (README §7 : fait, vérifié avec
   chiffres, non fait, dettes) ; carnet `docs/digests/revue-m0.md` mis à jour (colonne État).
3. `git push -u origin m0-revue-suite-contenu`, puis donner à William la ligne
   `LIVRÉ m0-revue-suite-contenu <sha> : <une ligne>` ; il la relaie à l'orchestrateur
   (`alacod-orch-39`), qui vérifie, fusionne avec M2 en cours et merge.
