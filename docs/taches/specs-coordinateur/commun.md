# Run alpha 1837 — consignes communes aux workers alacod (Mac de William), depuis le 7 nov 2026

Remplace la version du VPS (`vps/commun-vps.md`, gardée pour l'historique). Décision de William (6 nov, soir) : tout le travail de 1837 passe sur son Mac
(14 cœurs, vrai GPU, Chrome) ; le VPS ne garde que la suite de nuit.


## Modèles (décision de William, sam 7 nov)
Le compte Anthropic est réservé aux coordinateurs : aucun worker dessus. P-1 (simulation) : `google-antigravity/claude-opus-5-5`, seul worker Opus. R-1 (rendu,
présentation) : `openai-codex/gpt-6.1-sol` — **Codex jamais sur la simulation** (REGLES § 7) : il ne touche aucun crate `SIM`, aucun système de `GgrsSchedule`,
aucun type rollback. M-1b, M-2 : `google-antigravity/claude-sonnet-5-5`. Sur une erreur 429, préviens le coordinateur (`status`) : il te relance chez un autre
fournisseur au lieu d'attendre.
**Disque (studio, 7 nov)** : seuil 30 Go ; sous 35 Go, sweep avant de lancer un nouveau build. Toujours `CARGO_INCREMENTAL=0`. Après **chaque** contrôle rapide :
`scripts/target-sweep.sh target` puis `scripts/target-sweep.sh target_wasm` (s'il existe), jamais pendant un build. Aucun build release ou wasm hors preuve finale.
Un worktree part avec son dernier lot (branche gardée).
**Jamais de profil debug (studio, 8 nov)** : toute commande cargo de tests ou de vérification passe `--profile headless` (comme le Makefile : `make test_scenarios`,
`make check_rapide`…) ; `--release` seulement pour le build du jeu ; le wasm par `make` (ou `alacod build --web`). Une commande cargo **sans `--profile`** construit
`target/debug` : un second build complet d'environ 30 Go. Un worker de l'engine pèse environ 25 Go en headless.
Objectif studio : **O3, alpha coop du prologue de 1837, dim 20 déc** (dans Chrome, à la manette, à deux en ligne). Plan : `alacod/docs/taches/plan-1837-cli.md`
(§ 3 décisions de William, § 6 voie m : ton lot, sa preuve et son jalon) ; plan par jour : `1837/docs/tranche.md`. Règles de fond : `alacod/CLAUDE.md` (GGRS,
frontière simulation/présentation). Briefs du studio (copies) : `~/orca/runs/alpha/vps/{brief-studio,brief-parite-rendu,plan-jour}.md`.

## Reprise
Tu reprends un lot commencé sur le VPS. Lis d'abord **ta note de passation** `docs/taches/passation-<lot>.md` (dans ton worktree) : état, prochaine étape exacte,
pièges. Ta spec (`~/orca/runs/alpha/<lot>.md`) a été écrite pour le VPS : traduis ses chemins avec la table ci-dessous ; ce fichier prime en cas de conflit.

| VPS (spec, passation) | Mac |
|---|---|
| `/home/debian/orca/workspaces/alacod/<lot>` | `~/orca/workspaces/alacod/<lot>` (ton worktree, déjà créé sur ta branche poussée) |
| `/home/debian/orca/workspaces/1837/<lot>` | `~/orca/workspaces/1837/<lot>` (à créer, voir plus bas) |
| `/home/debian/orca/1837` | `~/Project/bascanada/1837` (checkout du **coordinateur 1837** : jamais d'écriture) |
| `/home/debian/workspace/1867_lore` | `~/Project/bascanada/1867_lore` (checkout de William : lecture seule, **ni pull ni écriture** ; une version plus récente se lit par `git -C … fetch -q` puis `git show origin/main:<chemin>`) |
| `/home/debian/orca/runs/alpha/` | `~/orca/runs/alpha/` (journaux dans `logs/`) |
| Xvfb + lavapipe | le vrai GPU du Mac (fenêtre ou capture hors écran) |

## Où tu travailles
- **Engine** : ton worktree. Un **target à toi** (celui du worktree, jamais partagé : verrou de cargo). Préfixe tes commandes cargo/make par
  `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`. **Un seul `cargo` à la fois** dans ton worktree, jamais en arrière-plan pendant un autre.
- **Disque** (partagé par trois workers et le travail de William) : `df -h /System/Volumes/Data` avant un gros build ; sous **40 Go libres**, `scripts/target-sweep.sh <ton target>` ;
  sous 30 Go, `ask`. Jamais de `pkill`/`killall` par motif (`pkill -f rustc` a tué les compilations des autres sur le VPS) : arrête seulement **tes** processus, par PID.
- **Jeu 1837** (si ton lot y touche) : ton worktree du dépôt 1837, sur la branche du lot déjà poussée :
  `git -C ~/Project/bascanada/1837 fetch -q && git -C ~/Project/bascanada/1837 worktree add ~/orca/workspaces/1837/<lot> <branche-1837>`
  (`m-1-web`, `p-1-parite` ou `r-1-rendu` ; elle suit `origin/<branche-1837>`). Ta branche 1837 est **mergée par le coordinateur 1837** (Mac, `~/Project/bascanada/1837`),
  pas par toi ; il change aussi le `rev` de l'engine sur `main` de 1837 après chaque merge d'engine.
- **Design** (`1867_lore`) : lecture seule. La définition du niveau est écrite par William.
- Tu ne merges jamais `main` de l'engine ; pousse ta branche (`git push -u origin <branche>`, avance rapide seulement, jamais `--force`) à chaque point de contrôle.

## Règles (décisions de William)
- **Rien de 1837 dans l'engine** : blocs génériques (verbes, behaviors, composants, outillage) testés avec le testbed et des noms neutres ; contenu, valeurs et noms de
  1837 dans ses RON.
- **Avant le merge, seulement le contrôle rapide** : `make check_rapide` (`WASM=1` si ton lot touche le web), **joué par toi** sur ta tête poussée, journal complet dans
  `~/orca/runs/alpha/logs/<lot>-check-rapide-<sha>.log`. La suite de scénarios passe la nuit sur le VPS ; joue seulement les scénarios de ton lot
  (`make test_scenarios SCENARIO=a,b`). Une trace existante qui bouge : dis-le dans le rapport (voulu ou non, pourquoi) ; ne `BLESS` que les traces neuves de ton lot.
- Simulation : checklist GGRS de `CLAUDE.md` ; un type rollback neuf = variante neutre ; `make schedule_order_diff` si la simulation change.
- **Pas de son** : `ALACOD_MUTE=1` ; Chrome en muet.
- **Chrome** : pas de boucle de Chrome automatisé ni de campagne de bots ; une capture ponctuelle est permise. La partie du soir de William fait foi : prépare la commande
  et le protocole (dix lignes au plus).
- Chaque rapport cite **le jalon servi** (plan § 6, « Jalon servi par chaque lot »). Une tâche qui ne sert pas ton jalon ne part pas : écris-la dans `propositions.md`
  (racine de l'engine) et continue.

## Preuve, rapport, fin
- Rapport `alacod/docs/taches/rapports/<lot>.md` : par point du lot, issue (fait / pas fait / non vérifié) et preuve (chemin, commande, chiffre, SHA) ; dernières lignes
  de `make check_rapide` ; ce qui reste. Une ligne `LIVRÉ <branche> : …` à la fin de `docs/taches.md` (§ 10). § de `docs/conventions.md` et dettes : numéros du plan § 6.
  Supprime ta note `docs/taches/passation-<lot>.md` dans le commit du rapport (son contenu utile passe dans le rapport).
- Aperçu à mi-parcours (`--type status`, sans `--to`) au moins une fois par jour et à chaque étape franchie. Carte de ton worktree :
  `orca worktree set --worktree active --comment "<état>" --json` à chaque point de contrôle (ne touche pas `--unread`).
- **Boîte** : relève-la (`check` de ton préambule) à chaque point naturel (avant un nouveau fichier, après chaque build ou test) ; une consigne urgente du coordinateur
  peut aussi arriver dans ton terminal.
- Question bloquante : l'`ask` de ton préambule, courte, avec ton choix par défaut ; avance sur le reste en attendant.
- Branche prête au merge : `status` « E prêt <sha> » avec le chemin du journal du contrôle rapide ; le coordinateur alacod relit, merge et te renvoie le SHA de `main`.
- Fin : branche(s) poussée(s), rapport commité, aucun processus laissé ; **un seul** `worker_done` (`--report-path`, SHA de tête engine et 1837).

## Suite complète : jamais sur le Mac (studio, 8 nov, REGLES § 6)
Avant merge : **seulement** `make check_rapide` (`WASM=1` si web). Pas de `make test_scenarios`/`test_scenarios_par` complet sur le Mac : la suite tourne la nuit sur le VPS et se trie le matin. Un ou quelques scénarios ciblés (`SCENARIO=a,b`) restent permis. Suite complète avant merge seulement sur demande écrite du coordinateur (rollback ou déterminisme touché), et alors sur le VPS (`ssh ovh-inner`), jamais ici.
