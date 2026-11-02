# Journal du coordinateur, polissage des clones — semaine 1 (28 oct – 1er nov 2026)

Copie intégrale de la continuation du coordinateur (`studio/runs/alacod-polissage.md`, PC0 à PC63), archivée le 2 nov 2026 quand la continuation a été compactée pour la semaine 2. Synthèse par tâche : `polissage-semaine-1.md`.

> Studio, jeu 29 ~17 h 15 UTC : le terminal du coordinateur (`term_7f5fc126-…`) a disparu après PC30 ; coordinateur relancé par le studio dans le checkout principal, nouveau terminal `term_39a7c2cc-…`, même run `run_b9bd92cceed4`, brief de reprise (décisions a–e du 29 : push de `main` sans PR, X-4b sur `origin/main` à réconcilier, son coupé, Sonniss au dépôt privé, disque). Reprise à PC31 ci-dessous par le coordinateur.

# alacod / plan-polissage § 0 — continuation du coordinateur

Coordinateur : terminal omp **`term_39a7c2cc-4355-4b49-9002-0750a882f4de`** (depuis PC31 ; avant : `term_7f5fc126-…`) dans le checkout principal du Mac
(`~/Project/bascanada/alacod`, branche `main` ; handle à relister après tout redémarrage d'Orca). Rôle : lancer, suivre,
relire, merger (`--no-ff`, moi seul) ; jamais coder. Run d'orchestration : **`run_b9bd92cceed4`** (créé le 2026-10-28 14 h 00 UTC).
Consignes envoyées aux workers : `studio/runs/alacod-polissage/` (Mac) = `/home/debian/orca/runs/alacod-polissage/` (VPS, copie) :
`commun.md`, `plan-polissage-s0.md` (copie du § 0, **non commité** dans le checkout de William), une spec par lot.

Reprise : `orca orchestration check --wait --types "worker_done,escalation,question,status" --timeout-ms 900000 --json`
(dans ce terminal) ; état : `orca orchestration worker-list --run run_b9bd92cceed4 --include-remote --json`.

## État des dépôts (2026-10-28)

- `main` local `59268426` (licence MIT/Apache, **non poussé**) ; `origin/main` `00dc83ce` (docs lots X) : divergents, docs
  seules des deux côtés. William a `docs/taches/plan-polissage.md` modifié (le § 0) et `tests/scenarios/revue_gungeon_bots.ron`
  non suivi : **ne jamais y toucher** ; vérifier avant chaque merge que la branche ne touche pas `plan-polissage.md`.
- Les quatre worktrees partent de `origin/main` `00dc83ce` (comme le disent les fiches). Merger dans `main` local fera
  entrer `00dc83ce` aussi (docs X seules, sans conflit avec le fichier modifié de William).
- Dépôt `estran-studio/alacod` **public** : aucune image payante commitée (surcharge locale gitignorée).
- Disque Mac : 56 Go libres (95 %) → b0 et b1 partagent `CARGO_TARGET_DIR=~/orca/workspaces/alacod/.target-mac`. VPS : 78 Go libres.

## Voies

| Voie | Machine, worktree | Lot en cours (tâche, dispatch) — état au PC36 | Modèle | Branche | Suite prévue |
|---|---|---|---|---|---|
| b0 | Mac, `~/orca/workspaces/alacod/b0` (garde le serveur E4 `:8777`, PID 22332) | S1-1, S1-5, S1-6, S2-9, E4 préparé **mergés** ; libre | sonnet | — | **E4 ven 30 dès que William arrive** (nouvelle tâche, `b0-e4-s2-9.md` § 1) ; S1-11 |
| b1 | Mac, `~/orca/workspaces/alacod/b1` (123 Mo, gardé pour les écoutes de William : protocoles et surcharges y sont) | S1-8, S1-9, S2-10, S2-11 **mergés** ; libre | sonnet | — | écoutes de William ; vidéo 60 s throne ; worktree supprimé après les écoutes |
| b2 | (aucun worktree : VPS `b2` et Mac `b2-s1-10-mac` supprimés, PC58) | lot 0, S1-7, S1-4, **S1-10 mergés** ; libre ; **S1-10** repris sur le **Mac** `~/orca/workspaces/alacod/b2-s1-10-mac` (`ctx_727cdaa8f379`, PC54) | omp bedrock sonnet | `polissage-zombies-map-equilibrage` | S2-12 |
| b2-allumette | **Mac**, `~/orca/workspaces/alacod/b2-allumette-mac` (VPS supprimé, PC44) | **S1-3** (`task_dc9718409401`, `ctx_85a185910b98`) | omp bedrock opus | `polissage-reseau-amis` | sam 31 deux réseaux, dim 1 zombies à 2 |
| b4 | (worktree supprimé) | D176 **mergé** (`79538914`) | omp bedrock opus | — | — |
| b3 | VPS `ovh-inner`, `/home/debian/orca/workspaces/alacod/b3-3` (neuf, PC59/61 ; `b3-2` supprimé) — **S2-12** (`ctx_7567caf20ea9`) ; avant : | S2-7, S2-8, S-0b **mergés** ; S1-10 points 1–2 **prêts** (`ctx_05d1308bf550`, terminal **repris par l'utilisateur** : release refusée `user_takeover`) | claude sonnet | `polissage-zombies-map-equilibrage` (reprise par b2) | S2-12 cavernes throne |

Placement : b2 sur le VPS (possible : remote `ovh-inner` connecté, repo `f30e88bd-…`). b3 : S2-7 sur le VPS (contenu, campagne,
disque du Mac), S2-8 sur le Mac (captures impossibles sur le VPS, D94) → deux worktrees successifs pour b3, un seul worker à la fois.
Adresser les workers VPS par dispatch, jamais par handle de terminal.

## Points de contrôle

- **PC0 — mer 28, 14 h 05 UTC** : run créé, quatre workers lancés et `live`, modèles effectifs = demandés (claude sonnet), base
  `00dc83ce` vérifiée dans les quatre worktrees. Cartes : b0–b3 `in-progress` ; b1 `--unread` (décision packs + disque).
  Aperçus attendus : b2 lot 0 prêt à merger (aujourd'hui) puis aperçu S1-7 fin J1 ; b1 audit classe × état fin J1 ;
  b3 campagne « avant » fin J1 ; b0 geste manette de William (J1/J2) puis aperçu fin J2.
  Constat : l'agent `claude` d'Orca lance **omp** (titre « π », transcript `provider omp`, modèle `anthropic/claude-sonnet-5-5`) :
  Sonnet effectif ; pour Opus (S1-6, S1-3), vérifier `launch.effective` et le modèle du transcript au lancement.
  14 h 10 : b1 construisait dans `b1/target` malgré `commun.md` → message aux dispatches b1 (bascule vers `.target-mac`,
  `rm -rf b1/target`) et b0 (rappel). Disque Mac 51 Go libres à 14 h 10.
- **PC1 — mer 28, 14 h 25 UTC** : b2 heartbeat (lot 0, `make check` en cours) ; b0 heartbeat (investigating).
  **b1 aperçu J1 reçu** : audit classe × état, branche `polissage-zombies-visuel` `199ce4ac`, rapport
  `docs/taches/rapports/polissage-zombies-visuel.md` § 1 : 21 classes, 14 empruntées, 4 absentes, 3 sans objet, **0 propre** ;
  le moteur ne joue que Idle/Run/Dying_* (pas d'état Attack, rollback, hors périmètre) ; le pack ne couvre ni distributeurs de
  perks, ni boss propre, ni l'atlas de murs à 36 tuiles (à recomposer). Décisions du coordinateur envoyées à b1 (réversibles) :
  option A (surcharge locale étendue `hud.local.ron` + moteur de présentation générique), boss = Zombie_Big agrandi/teinté par
  donnée, distributeurs en attente de William (b1 cherche 1–2 candidates CC0/CC BY sans importer), réestimation **≈ 4 j acceptée**
  (S1-9 → J5, captures + vidéo 60 s → J6 ; décale S2-10 d'un jour).
- **PC2 — mer 28, ~14 h 40 UTC (message du studio, de la part de William)** : packs téléchargés dans `~/Downloads` :
  TheLazyStone `PostApocalypse_AssetPack_v1.1.2(1).zip` (licence commerciale 2 $ **payée** ; confirmation de l'auteur pour un
  build web public **non obtenue** → option A maintenue, rien de payant commité), `kenney_impact-sounds.zip`, deux archives
  Snake's Gun Sounds, Sonniss GDC 2026 1–5 (≈ 6,9 Go, royalty-free sans attribution). Leohpaz Undead **non acheté** (Stripe) →
  sons de zombies dans Sonniss. Aymen B et Ittai absents (throne, avant S2-10 lundi ; le studio relance William).
  Règle : archives lues en place ou extraites **sélectivement** sous `~/orca/workspaces/alacod/packs/<pack>/` (créé), licences
  dans `assets.yaml`. Note aux workers : `studio/runs/alacod-polissage/packs.md` (copiée sur le VPS). b1 : intégration
  TheLazyStone lancée (message haute priorité). Le studio supprime `x-4b-capture-mac` et son `target/` (≈ 24 Go) ; le
  `cargo clean` du checkout de William reste à William. Questions 1, 3, 4, 6b transmises à William par le studio.
- **PC3 — mer 28, ~14 h 50 UTC** : **b0 aperçu J1** : `bevy_gilrs` 0.19.1 / `gilrs` 0.11.2 présents dans l'arbre wasm de zombies
  sans changer `Cargo.toml` ; build wasm OK (8 min 35, dev) ; dans Chrome 156 avec une manette **simulée** (`getGamepads`
  injecté), le D-pad déplace le joueur. **Manette physique non vérifiée** (pas de manette sur le Mac) : gâchettes, deux sticks,
  reconnexion → William. Constats : la visée au stick n'existait pas (`Pan` jamais lu) → ajoutée ; contradiction de la fiche sur
  le D-pad réglée (stick gauche = déplacement) ; actions de jeu masquées sous un écran ouvert. Estimation 3 j inchangée.
  Réponse du coordinateur : validé, garder le choix de mutation jouable sous écran, ajouter une page de test `getGamepads` de
  deux minutes pour William avant E4.
- **PC4 — mer 28, ~14 h 45 UTC (réponses de William par le studio)** : (1) **Décision de William (2026-10-28)** : images payantes
  hors du dépôt public ; **dépôt privé d'assets** estran-studio (à créer) cloné par la CI du site ; d'ici là option A. Proposition
  d'insertion du coordinateur : voir « Lot S-0b » ci-dessous. (2) Manette : William prend deux minutes ; b0 pose lui-même sa carte
  `--unread` (instant + commande exacte) quand c'est prêt — message envoyé. (3) S1-3 : tâche de préparation lancée sur le VPS :
  **b2-prep** (`task_55830486b9c1`, `ctx_ce73cb5fa82c`, worktree VPS `/home/debian/orca/workspaces/alacod/b2-allumette`,
  branche `polissage-reseau-amis`, **claude opus** — processus `claude --model opus` constaté sur le VPS ; sur le VPS l'agent est
  Claude Code, sur le Mac c'est omp), spec `b2-prep-s1-3-allumette.md`, échéance fin J2. Cinquième worker concurrent, demandé par
  William ; lecture seule, b2 reprend la branche à J3. **14 h 30 : carte b2-allumette `--unread`** : il manque (a) la clé
  d'ovh-inner (`[clé publique d'ovh-inner]`, SHA256:[retiré]) dans
  `~debian/.ssh/authorized_keys` d'alumette (`[hôte d'alumette]`), (b) Docker en lecture pour `debian` sur alumette
  (`sudo usermod -aG docker debian`). (4) Aymen B arrivé : `~/Downloads/UPDATE 2_ZombiePack.zip` (S2-10) ; Ittai manque.
  (5) Disque : checkout de William 144 → 34 Go, Mac à 157 Go libres ; cible partagée plus obligatoire (b0/b1 la gardent).
  (6b) distributeurs : pas tranché ; b1 propose 1–2 candidates, décision avec les preuves de fin de semaine.
  `packs.md` et `commun.md` mis à jour (Mac et VPS).
- **PC5 — mer 28, ~14 h 50 UTC** : packs complets relayés à b1 (tableau « avec le pack » demandé, pas encore reçu) ; b2-prep
  `worker_done` relu, worker libéré ; b2 prévenu du retard de `releases.json` du dépôt sur la prod (lot 0). Question de b1
  (`msg_d7930a8cf1cd`) : captures avec la surcharge payante dans le dépôt public ? Réponse du coordinateur : **A** — seules les
  captures du repli public sont commitées (`docs/captures/polissage-zombies-visuel/`) ; celles de la surcharge restent hors dépôt
  dans `~/orca/workspaces/alacod/captures/polissage-zombies-visuel/` (Mac), chemin dans le rapport ; elles iront au dépôt privé d'assets.
- **PC6 — mer 28, ~15 h 05 UTC** : **b1, tableau classe × état AVEC le pack** (status 14 h 59 UTC) : `polissage-zombies-visuel`
  `28660d93` (3 commits sur `00dc83ce`, 39 fichiers, aucune trace touchée), rapport § 2 (tableau après), arborescence de surcharge
  documentée (§ 91 → **à renuméroter en § 92**, § 91 pris par X-4b). Avec le pack, classes principales (joueur, zombies, boss,
  armes, projectiles, portes, fenêtres 4 états, murs, sol, icônes de HUD) : **0 empruntée** ; lisibilité coop faite (couleurs,
  légende F7, halos, noms F6, `hide_markers`, aussi en repli public). Restent empruntés/absents : distributeurs de perks (6b), slash
  (licence inconnue), ombre, 3 power-ups sur 5 (D162). Captures : repli public commité `docs/captures/polissage-zombies-visuel/repli-public_*.png` ;
  surcharge hors dépôt `~/orca/workspaces/alacod/captures/polissage-zombies-visuel/surcharge-locale_*.png`. Dettes D161 (un build
  web depuis le Mac embarquerait `sprites/postapo/` et `*.local.ron` : copie de `games/*/assets/` entière par
  `build-web-games.mjs`/`embed-game-content.rs`) → à traiter dans S-0b avant tout build web local ; D162, D163. `make test_tools` :
  3 échecs d'environnement sur le Mac (`/usr/bin/python3` 3.9, script ≥ 3.10) → vérifier sur le VPS au merge. Retours envoyés : § 92,
  sol de la surcharge qui couvre le hors-salle, transcript de test_tools.
- **PC7 — mer 28, ~15 h 25 UTC** : **b1 `worker_done` S1-8 succeeded** (15 h 23 UTC, `ctx_574a07183120`) : `polissage-zombies-visuel`
  `a19b172e` poussé, ligne `LIVRÉ` et rapport complets ; vérifié par le worker : format, forbidden, rollback, sim_boundary,
  test_crates, **810 scénarios sans BLESS**, lint de 5 jeux, lint zombies avec et sans surcharge ; `make test_tools` : 3 échecs
  d'environnement (`test_claude_say.py`, `/usr/bin/python3` 3.9.6 du Mac). Mes deux retours (§ 92, sol hors-salle) n'avaient pas
  été lus avant le `worker_done` → **terminal réutilisé** pour une tâche de retouche : `task_fa7c34caef35` / `ctx_11445012013b`
  (même worktree, même branche). Merge de S1-8 après la retouche, avec `make test_tools` vérifié sur le VPS.
  S1-8 fini en J1 au lieu de 4 j : la réestimation à 4 j est caduque ; b1 enchaînera S1-9 (préparation des sons à partir des
  packs dès maintenant, câblage après le merge de S1-7).
  Méthode de merge retenue : merge `--no-ff` fait dans un worktree d'intégration du coordinateur
  (`~/orca/workspaces/alacod/integration`, branche `polissage-integration` depuis `main`), `make check` sur le résultat, puis
  `git merge --ff-only polissage-integration` dans le checkout de William (aucun build dans son checkout).
- **PC8 — mer 28, ~15 h 30 UTC** : **b0 aperçu « fin J2 » reçu avec un jour d'avance** (15 h 27 UTC) : `polissage-manette`
  `1dddd3b0` poussé. Chrome 156, zombies et throne en wasm, manette standard **simulée** : deux sticks (8 directions, seuil
  0,4), gâchettes analogiques, 17 boutons, croix, axe Y : chaque bit vérifié dans le journal. Trouvailles corrigées : visée au
  stick absente (ajoutée) ; RT tirait dès 0,02 de course (leafwing) → seuil 0,5 / relâche 0,35 ; **zombies et throne n'avaient
  aucun écran de pause ni d'inventaire** (gabarits `pause_screen.html` absents) → gabarits ajoutés (fichiers nouveaux dans
  `games/{zombies,throne}/assets/ui/`, + `game_over_screen.html` retouché) ; actions masquées sous un écran ; A valide l'attente
  du lobby local (`ui/lobby.rs`, `release_local_hold`). Aide « Touches » dans la pause, filtrée par clone ; captures
  `docs/taches/rapports/polissage-manette/{zombies,throne}-{pause,aide,inventaire}.png` (branche). Non vérifié : manette physique,
  reconnexion, PlayStation, deux manettes. Build E4 servi sur `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-manette`
  (200 vérifié). Réponse : fichiers OK ; carte `--unread` « MANETTE 2 min » demandée à b0 maintenant. **À dire à b2 au départ de
  S1-4** : `ui/lobby.rs` et `game_over_screen.html` touchés par S1-1 (merge S1-1 avant S1-4, rebase).
- **PC9 — mer 28, 15 h 45 UTC** (horloge relevée ; les heures des PC5–PC8 sont corrigées d'après les messages) : attente vide ;
  b2 lot 0 : `make check` encore en cours (scénarios) avec `releases.json`, `docs/taches.md` et `crates/content/tests/lint_fixtures.rs`
  modifiés, non commités ; b3 : campagne « avant » en cours ; b0 : `make check` en cours, carte manette pas encore posée ;
  b1 : retouche en cours. VPS : 39 Go libres, charge ≈ 5,6.
- **PC10 — mer 28, 16 h 15 UTC** : trois attentes vides → inventaire : cinq dispatches `live`, aucun `requiresAction` hors
  l'ancien dispatch b1 réglé (`ctx_574a07183120`, terminal transmis). b0 a posé sa carte `--unread` ; j'y ai mis le geste exact
  (« WILLIAM, MANETTE 2 min : … `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-manette` … »). b2 : `make check` du lot 0
  lancé à 14 h 01, scénarios depuis 15 h 27 (VPS chargé, 10 de charge avec b3) — lent, pas bloqué ; l'aperçu S1-7 de fin J1 glisse.
- **PC11 — mer 28, ~16 h 40 UTC (studio, de la part de William)** : **test physique de la manette fait** sur
  `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-manette` : « ça marche assez bien ». Notes non bloquantes : (a) un peu
  trop rigide → **à régler dans S1-6** (feel, réglages par clone), pas dans S1-1 — à mettre dans la spec de S1-6 ; (b) icônes de
  boutons de manette dans les menus et l'aide → plus tard, entrée dans `propositions.md` (racine d'alacod, à créer par b0), pas de
  dette. Transmis à b0 (haute priorité), carte b0 levée (`--read`). S1-1 continue : menus, salon, game over, E4 ven 30.
  b0 a aussi pris « conventions § 91 » (`cce1eb1f`) → renumérotation demandée : **§ 92 = b1 (S1-8), § 93 = b0 (S1-1)** ; les voies
  suivantes prennent § 94 et plus, à annoncer au coordinateur avant d'écrire.
- **PC12 — mer 28, 23 h 20–23 h 35 UTC** : (1) **Panne des workers VPS** : b2 et b3 arrêtés depuis 15 h 46 (b3) et 18 h 09 (b2) sur
  « Claude Code can't use your Claude subscription … payment is past due » ; rien ne m'était parvenu (aucun message, `live` dans
  `worker-list`). À 23 h 25, `claude -p` et `omp -p` répondent de nouveau sur le VPS → les deux relancés par `terminal send`
  (un `orchestration send` ne réveille pas une TUI au repos) ; ils ont repris. Leçon : après trois attentes vides, **lire la queue
  des terminaux** (`worker-read --source terminal`), pas seulement la liveness. (2) b2 avait poussé le lot 0 :
  `origin/polissage-online-faux` `3aceb282` (`6060d2d6` + journal), `make check` code 0 ; b3 : `origin/polissage-throne-jouable`
  `06b4b58d` (D66/D67 vérifiées, campagne partielle). (3) b0 : S1-1 terminé et poussé (`polissage-manette` `a64a0598` : rapport,
  LIVRÉ, D158–D160, notes de William, `propositions.md`, § 93) ; réponse à son `ask` : `worker_done` maintenant, serveur
  `127.0.0.1:8777` laissé pour William (PID et arrêt dans le rapport), E4 complet = nouvelle tâche ven 30.
  (4) **Verdict de William sur b1** (captures `surcharge-locale_*`) : « les nouvelles sprites sont bien et l'affichage a l'air
  correct ». **Décision de William (2026-10-28)**, dans S1-8 : portes doubles (une paire collée = une seule porte double) ;
  vraies fenêtres (états dessinés, pas une barre d'UI) et zombie qui attaque la fenêtre (pose de présentation dérivée, aucun type
  rollback, modèle `present_shooting` de X-4b, branche non mergée). **Réestimation du coordinateur : ≈ 2 j** (portes 0,5 · fenêtres
  0,5 · pose d'attaque 1) → S1-8 finit J3 (ven 30) ; S1-9 J4 (sam 31) et vidéo 60 s J5 (dim 1) = **dates du § 0.2, rien ne glisse**,
  aucun échange ; tient dans les 4 j de la réestimation de b1 (le cœur de S1-8 a été livré en J1). Risque : si la porte double
  exige de changer les cartes ou la simulation (traces de zombies, `BLESS`), b1 s'arrête et pose la question. Spec
  `b1-s1-8-suite.md` écrite ; b1 prévenu, la tâche part sur son terminal après le `worker_done` de la retouche.
- **PC13 — jeu 29, ~0 h UTC** : messages traités : **b0 `worker_done` S1-1 succeeded** (`polissage-manette` `def324ce`, 810 scénarios
  sans BLESS, test_crates et lint verts, `test_tools` rouge d'environnement, serveur `127.0.0.1:8777` PID 22332 laissé pour l'E4) →
  worker libéré ; **b3 aperçu campagne « avant »** (100 graines × 1 et 2 bots, profil prudent, jusqu'à l'étage 3) : 1 bot : étage 1
  100 %, étage 2 89 %, run complète 41 %, 49 morts (48 à l'étage 3), 10 soft-locks ; 2 bots : 100 / 97 / 86 %, 1 mort, 13
  soft-locks ; 0 desync ; aucun étage < 20 % → **aucun réglage**, aucune trace ; D66/D67 déjà codés (teinte/échelle, orientation,
  rayons) → clôture côté code, confirmation visuelle sur le Mac (S2-8) ; soft-locks (23) à surveiller. **b2 lot 0 prêt** : `release.yaml`
  → `build-web-games.mjs` réécrit `releases.json` au tag (`WEB_ENABLE_ONLINE` non posé) → la prod ne change qu'au tag ; `online:
  false` coupe le lien « Avec des amis », « Créer un salon privé » et les invitations, la route `/online` reste ouverte → **la recette
  S1-3 demandera `online: true` dans le build testé** (`WEB_ENABLE_ONLINE=1` ou preview).
  **Intégration** : worktree `~/orca/workspaces/alacod/integration` (branche `polissage-integration` depuis `main` `59268426`,
  poussée) : `caa001ac` = merge `--no-ff` de `polissage-online-faux` ; `ebab067a` = merge `--no-ff` de `polissage-manette` (conflit
  du journal `docs/taches.md` résolu en gardant les deux lignes `LIVRÉ`) ; aucune `.trace` modifiée (seul `ui_pause_menu.ron`,
  script d'UI, étendu) ; `make test_tools` **vert sur le VPS** (`/tmp/chk-int`, Python 3.13, 38 tests) ; `make check` sur le Mac
  s'arrête à `test_tools` (Python 3.9) → reste de `make check` lancé sans `test_tools` (en cours). Puis `main` avancera en `--ff-only`.
  **b0 S1-6 lancé** : `task_22becec3a517` / `ctx_0330b3d9f0bd`, worktree b0, **modèle effectif `anthropic/claude-opus-5-5`** (transcript),
  spec `b0-s1-6-feel.md` (diagnostic de la rigidité d'abord, § 94, E4 de S1-1 vendredi au milieu).
- **PC14 — jeu 29, ~0 h 45 UTC** : `make format check_forbidden check_rollback_registration check_sim_boundary test lint` **vert**
  sur `polissage-integration` (Mac, EXIT 0, 38 min, `test scenarios ... ok`, aucun FAILED ; journal `/tmp/integration-check-1.log`
  et `-1b.log`) + `make test_tools` vert sur le VPS → **`main` avancé en `--ff-only` à `ebab067a`** dans le checkout de William
  (ses `plan-polissage.md` modifié et `revue_gungeon_bots.ron` non suivi intacts). **Mergés : lot 0 `polissage-online-faux`, S1-1
  `polissage-manette`.** `main` local **non poussé** (comme `59268426` avant lui : la poussée de `main` est à William, règle PR) ;
  la branche `polissage-integration` (= `main`) est poussée et sert de base aux voies du VPS. S-0b confirmé (dépôt privé créé par
  le studio), spec écrite.
- **PC15 — jeu 29, VPS `ovh-inner` injoignable depuis ~23 h 45 UTC (mer 28)** (avertissement du studio : runtime Orca distant
  injoignable **et** ssh en timeout au banner → l'hôte ou le réseau ; William prévenu, son infra). b2 (`ctx_1619a37a8faf`, S1-7) et
  b3 (`ctx_43af9bf6b227`, S2-7) y tournent : liveness `unverifiable` ≠ mort. **Consigne** : aucun `worker-stop`, aucun abandon,
  aucune relance en double sur le Mac ; attendre le retour de l'hôte, puis `worker-show`/`worker-list --include-remote` pour relire
  l'état réel. Dernier état connu poussé sur origin : b2 `polissage-online-faux` `3aceb282` (mergé), `polissage-sons-evenements`
  créé vers 23 h 25 (rien de poussé constaté) ; b3 `polissage-throne-jouable` `06b4b58d` (campagne « avant », D66/D67) ; b2-prep
  `polissage-reseau-amis` `98b923b0`. Les voies du Mac (b0 S1-6, b1 S1-8 suite) ne dépendent pas du VPS.
  Vérifié sur origin (Mac, `git ls-remote`) : `polissage-sons-evenements` **absente** → le travail S1-7 de b2 n'existe que sur le
  disque du VPS ; au retour, lui faire pousser en premier. Présentes : `polissage-feel` `d9467975` (b0), `-integration` `ebab067a`,
  `-manette` `def324ce`, `-online-faux` `3aceb282`, `-reseau-amis` `98b923b0`, `-throne-jouable` `06b4b58d`, `-zombies-visuel` `c0edbc30`.
- **PC16 — jeu 29, ~0 h 25 UTC** : derniers signaux avant la panne : b2 23 h 26 (« S1-7 : content, clés + lint + tests verts ;
  game : volume global, sons »), b3 23 h 32 (« make check » de S2-7). **b0 S1-6, diagnostic de la rigidité** (`ask` 23 h 56) :
  (1) affichage sans interpolation : sur l'écran à 60 Hz du Mac, 21 % des images figent le joueur en course, 59 % ont un à-coup →
  **corrigé en présentation** (1 % et 13 %), aucune trace ne bouge ; (2) stick numérisé : 8 directions (seuil 0,4 par axe, diagonale
  entre 24° et 66°), vitesse tout ou rien (0 ou 150 px/s en 3 frames) → ne se règle que par un **input analogique** (≈ 1 j :
  `BoxInput` + deux `i8`, format réseau changé, vieux clients incompatibles, `PlayerScript`, enregistrement, bots ; traces
  identiques seulement si le chemin numérique reste bit à bit, sinon `BLESS` de tous les jeux) ; baisser l'accélération
  (3000 → 1500 px/s²) adoucirait aussi le clavier validé par William et bougerait zombies **et** throne. **Réponse** : défaut de b0
  retenu (ni analogique ni accélération dans S1-6 ; William rejuge avec l'interpolation à l'E4 vendredi) ; question posée à William.
- **PC17 — jeu 29, 0 h 30 UTC : VPS de retour** (ssh OK, charge 55–66 sur 8 cœurs, 17/23 Go de mémoire, 35 Go de disque libre ;
  la charge vient surtout d'autres voies hors de ce run : bots de `x-4b-import-sprite`, hermes, syncthing, longues compilations
  `bevy_render`/`bevy_ui`). `worker-list` : b2 et b3 `live`. **b2** a poussé son WIP `polissage-sons-evenements` `1d080836` (« clés de
  sons, repli `death_<id>`, volume global, sons d'événements, non compilé ») ; son `cargo test -p game` a été tué faute de mémoire ;
  il attendait ma réponse (perdue pendant la coupure). **b3** : `make check` de S2-7 en cours, `fc69ff4c` (fmt de `lint_fixtures.rs`)
  local. Décision : un seul build lourd à la fois sur le VPS pour mes voies, **b3 d'abord**, b2 ensuite avec `CARGO_BUILD_JOBS=4` ;
  messages envoyés aux deux, b2 réveillé par `terminal send`.
- **PC18 — jeu 29, ~0 h 35 UTC** : **b0 aperçu S1-6** (`polissage-feel` `d9467975`, poussé) : sans interpolation, 16–20 % des images
  affichent 0 ou 2 frames simulées → 8–21 % d'images figées en course, 24–59 % d'à-coups ; **avec interpolation** (`PostUpdate`,
  horloge lissée) : 1 % figées, 12–13 % d'à-coups, 10 ms de retard moyen (`scripts/frame-pacing.py`, scénario neuf `feel_zombies`,
  deux passes). Valeurs zombies = throne inchangées (accélération/freinage 3000 px/s², 150 px/s, dash 64 px/8 f/cd 24/i-frames 6,
  flash 4 f, secousse 8/12, hit-stop 2, caméra lerp 12) ; nouveau en données : flash de bouche, recul visuel, éclats de mort (zombies
  sobre, throne plus lourd). **811 scénarios sans BLESS** (seule trace neuve : `feel_zombies`), `schedule_order_diff` identique
  (4550 lignes, 645 types). Vidéos 60 s : `~/orca/workspaces/alacod/b0/target/videos/polissage-feel/feel_zombies.{avant,apres,avant-apres}.mp4`
  (copie hors de `target/` demandée). Réestimation 1,5 j. Suite : build web `e4-feel` pour comparer avec `e4-manette` à l'E4.
- **PC19 — jeu 29, ~0 h 40 UTC** : escalade de b2 (0 h 30, arrivée après ma réponse de 0 h 33) traitée. Consigne ajoutée : **rebase de
  `polissage-sons-evenements` sur `origin/polissage-integration` avant sa compilation unique** (S1-1 a étendu `ui_pause_menu.ron`
  et touché `ui/screens/{model,mod,input}.rs`, `game_over.rs` : garder « Touches » et ajouter le curseur de volume à côté).
- **PC20 — jeu 29, ~1 h 05 UTC** : **b1 S1-8 suite livrée** (`worker_done` 0 h 57, `polissage-zombies-visuel` `f5fb100c`) : audit des
  portes : avant_poste 8 paires, la_fonderie 12, toutes jumelles par `paired_door` (créées à l'assemblage des salles), deux prix mais
  ouverture commune dans la même frame → **aucune carte ni simulation à changer** ; porte double en présentation (`DoorPair` hors
  rollback, `door_pairs` pure et testée, second battant en miroir, une étiquette F6, donnée `hud.ron double_doors: true`) ; fenêtres :
  image du pack ×2 (entier) + teinte d'état, barre gardée à 2 px ; pose d'attaque : `PresentationClip` (même type que X-4b) +
  `present_attacking` dérivé de `MonsterState::Attacking { last_attack_frame }` pendant 48 frames, seulement si la feuille a `Attack`.
  Preuves hors dépôt (`~/orca/workspaces/alacod/captures/polissage-zombies-visuel/`) : `surcharge-locale_porte_double_{fermee_f00140,ouverte_f00200}.png`,
  `surcharge-locale_1j_fonderie_portes_f00900.png`, `surcharge-locale_fenetre_4_etats_Idle-Damaged-Critical-Broken_f300-372-392-450.png`,
  vidéo 14 s `surcharge-locale_fenetre_zombie_frappe_brise_entre_demi-vitesse.mp4` (vue : le zombie frappe, la fenêtre cède, il entre).
  810 scénarios sans BLESS, `schedule_order_diff` identique, lint de 5 jeux. **Point de merge X-4b** : `PresentationClip` défini des deux
  côtés (`crates/animation/src/lib.rs`, `visuals.rs`) → une seule définition au merge de X-4b. b1 enchaîne **S1-9** sur le même terminal
  (`task_33baf10c8cea`, `ctx_47a2e0ded77c`, base `f5fb100c`). **Intégration** : `a7a7a49b` = merge `--no-ff` de `polissage-zombies-visuel`
  (conflits `conventions.md` — § 92 placé avant § 93 —, `taches.md`, `dettes.md` : ajouts des deux côtés gardés) ; `test_tools` vert
  sur le VPS ; `make check` (sans `test_tools`) en cours sur le Mac (`/tmp/integration-check-2.log`). `polissage-integration` poussée
  avant la fin de ce `make check` (b2 doit rebaser dessus) : si rouge, corriger avant que b2 rebase.
- **PC21 — jeu 29, ~1 h 40 UTC** : **b0 S1-6 fini hors E4** (`polissage-feel` `d6b06aa7`/`d70be914`, `make check` vert sauf
  `test_tools`, 811 scénarios sans BLESS, `schedule_order_diff` identique ; builds `e4-manette` et `e4-feel` servis sur `:8777`) ;
  réponse à son `ask` : **worker_done maintenant**, puis **S1-5** (perf wasm, Sonnet, spec `b0-s1-5-perf-web.md`) dans le même
  worktree, avec l'E4 de vendredi (S1-1 + comparaison du feel) comme étape prévue. **b1 aperçu S1-9** (`polissage-zombies-sons`
  `be072c0a`, poussé, base `f5fb100c`) : tableau événement × son complet ; armes Snake's (domaine public selon l'auteur, « used
  commercially, credit not needed », sans texte de licence dans l'archive), Kenney CC0, créatures et événements Sonniss (licence PDF :
  libre de droits sans attribution, **interdit de revendre ou redistribuer les sons tels quels**, pas d'entraînement d'IA) ; D32
  confirmé ; volumes de départ posés. **Décision du coordinateur, par prudence** : les 14 extraits Sonniss étaient déjà publics sur la
  branche poussée → b1 réécrit sa branche sans eux (`--force-with-lease`, sa seule branche), fichiers gardés en local gitignoré ;
  question posée à William (11). GitHub peut garder un temps les commits orphelins accessibles par SHA. Clés `window_break` /
  `window_repair` absentes de S1-7 → demandées à b2. Mesure de b1 sur les fenêtres : Broken lisible sans la teinte, Damaged à peine,
  **Critical seulement par la teinte** (à D163).
- **PC22 — jeu 29, 03 h 20–03 h 35 UTC** : (1) **VPS revenu à 03 h 20** (studio : coupure réseau/relais, pas de redémarrage, uptime
  41 j) ; **disque du VPS à 97 % (6,7 Go libres)** → `target-sweep` ordonné à b2 et b3 à 03 h 23 (message + `terminal send`). Résultat :
  b3 (lot fini) a **supprimé son target (37 Go)** et un binaire copié (1,2 Go) ; b2 : sweep de `b2/target` 34 → 27 Go (8,5 Go libérés,
  profil headless seul). VPS à 03 h 25 : **105 Go libres (45 %)**, b2 a ensuite relevé 68 Go libres (d'autres voies écrivent). Mac :
  sweep de `.target-mac` (aucun cargo en cours) 120 → 102 Go, **26,4 Go libérés**, 68 Go libres (93 %). (2) **Intégration : `make check`
  sans `test_tools` vert** (EXIT 0, `test scenarios ... ok`, 24 min) avec S1-8 → **`main` avancé en `--ff-only` à `a7a7a49b`** (fichiers
  de William intacts). **Mergés : lot 0, S1-1, S1-8.** (3) **b3 S2-7 livré** (`worker_done` 02 h 58, `polissage-throne-jouable`
  `70e283bf` : D66/D67 vérifiées en code, campagne avant, aucun réglage, `make check` vert ; reste : confirmation visuelle Mac et les
  23 soft-locks, décision de William) → worker libéré ; carte VPS `in-review`. **b3 S2-8 lancé sur le Mac** : worktree
  `~/orca/workspaces/alacod/b3` (base `origin/polissage-integration`), `task_4c5c6639aa6e` / `ctx_a382e2c73ff7`, sonnet, spec
  `b3-s2-8-lisibilite-coop.md` (vidéo D66/D67 d'abord). (4) **b0 S1-6 livré** (`worker_done` 03 h 21, `polissage-feel` `d70be914` :
  interpolation coupée au téléport, à la nouvelle partie, au rollback et en capture ; flash/recul/éclats par arme ; `feel_zombies`
  trace neuve ; vidéos et CSV dans `~/orca/workspaces/alacod/captures/polissage-feel/` ; § 94 ; 811 scénarios) → worker libéré ; merge
  **après S1-7** (ordre § 4, `feedback.rs` commun). **b0 S1-5 lancé** : `task_12217d8d2886` / `ctx_6a4cc401a3bf`, sonnet, spec
  `b0-s1-5-perf-web.md` (E4 vendredi inclus).
- **PC23 — jeu 29, ~03 h 45 UTC** : **b1 S1-9 étape 1 livrée** (`worker_done` 03 h 28, `polissage-zombies-sons` `0355a2f7`, branche
  réécrite sans Sonniss en `05b6e1f0` puis `0355a2f7`, `--force-with-lease` de sa seule branche ; les 14 extraits Sonniss sont sur le
  disque sous `games/zombies/assets/sounds/sonniss/`, gitignorés ; `be072c0a` reste lisible par SHA sur GitHub) : armes Snake's
  branchées, `window_break`/`window_repair` (Kenney) prêts, 810 scénarios sans BLESS, captures du repli public
  `docs/captures/polissage-zombies-sons/`, vidéos 60 s hors dépôt. Les clés Sonniss ne peuvent pas aller dans le `feedback.ron` public
  (fichier absent en CI = lint rouge) → **étape 2 de S1-9 = surcharge `feedback.local.ron`** (même principe que `hud.local.ron`) après le
  merge de S1-7. b1 avancé sur **S2-10 throne visuel** en attendant (`task_bfb7e994bfb1`, `ctx_06b6a816308f`, même terminal, spec
  `b1-s2-10-throne-visuel.md`, packs Aymen B + Ittai par la surcharge, interruption pour l'étape 2 de S1-9 dès le merge de S1-7). b3 :
  section § 95 attribuée (§ 94 = b0 S1-6 ; § 96 réservé à b1 S2-10). **S2-7 mergé** : `2ff27132` (journal résolu ; aucun code changé
  depuis `a7a7a49b`, `git diff --stat a7a7a49b HEAD -- crates games tests scripts website Makefile Cargo.*` vide ; `make format
  check_forbidden lint` vert) → **`main` avancé à `2ff27132`**, `polissage-integration` poussée ; b3 rebase dessus.
  **Mergés : lot 0, S1-1, S1-8, S2-7.**
- **PC24 — jeu 29, ~03 h 55 UTC** : réponses de William (par le studio) appliquées (voir « Décisions de William ») : b1 reçoit
  l'interlude des distributeurs CC0 (haute priorité, avant de reprendre S2-10) ; carte b2-allumette levée ; b0 a le § 97 (S1-5,
  compteur FPS `?perf=1` / F8 / `ALACOD_FPS`, build jetable intégration + `polissage-feel` non poussé pour mesurer l'interpolation).
  **S1-3 : spec `b2-s1-3-reseau-amis.md` écrite (copiée sur le VPS)**, avec les décisions 7, 8 et 1/9. **À lancer ven 30 au matin**
  en **Opus** dans le worktree VPS `b2-allumette` (branche `polissage-reseau-amis`), même si b2 n'a pas fini S1-4 : le rendez-vous de
  samedi est fixe. b2 (Sonnet) continue alors S1-7 puis S1-4 dans son worktree ; deux workers sur la voie b2, comme pour la préparation,
  et à le dire au studio.
- **PC25 — jeu 29, ~03 h 50 UTC** : **b2 aperçu S1-7** (03 h 46) : `origin/polissage-sons-evenements` rebasée sur l'intégration ;
  28 clés optionnelles (dont `death_<id>` avec repli `death`, `window_break`/`window_repair`, `wave_start/end`, `room_cleared`, `floor`,
  `boss_spawn/phase/death`, `purchase`, `pickup`, `explosion`, `bomb`, `menu_navigate/confirm/back`) ; lint : clé inconnue et
  `death_<id>` d'un personnage inconnu en erreur ; volume global dans « Réglages » de la pause (← → / croix), `MasterVolume` hors
  rollback, en dB en plus des `volumes`, `ALACOD_MUTE` prioritaire ; conflits avec S1-1 résolus dans `ui/screens`. `make check` et
  `test_scenarios` en cours (`CARGO_BUILD_JOBS=4`). **Reste à faire jouer sur le Mac** : protocole « audio navigateur » de son rapport
  (premier clic débloque l'`AudioContext`) → à confier à b0 pendant S1-5 (Chrome) après le merge de S1-7.
- **PC26 — jeu 29, ~04 h 00 UTC** : **interlude distributeurs CC0 livré par b1** (status 03 h 53) : `polissage-zombies-distributeurs`
  `fbdc5924` (depuis `a7a7a49b`), source karsiori « Pixel Art Vending Machines » (`https://karsiori.itch.io/pixel-art-vending-machines`,
  **CC0 lue sur la page itch**, l'archive n'a pas de fichier de licence) ; Kenney Roguelike Modern City écartée. 4 machines aux couleurs
  des perks, 48 px de haut, **commitées** (CC0 : repli public et surcharge) ; présentation : `world_sprites` pose `perk_machine/<perk>`,
  `hide_soda_tiles` masque la tuile LDtk `SodaLocation` ; scénarios `clone_solo`, `buy_perk`, `shop_tour` sans BLESS, lint zombies avec et
  sans surcharge ; capture `docs/captures/polissage-zombies-distributeurs/repli-public_1j_f00200.png` ; D162 : partie distributeurs
  fermée. Suite complète des 810 scénarios **à faire au merge** (prochaine fournée avec S1-7). b1 reprend S2-10.
- **PC27 — jeu 29, ~04 h 05 UTC** : **runtime Orca local bloqué en `starting`** (`orca status` : app `running`, runtime `starting` ;
  `check --ack` → `runtime_timeout` à deux reprises sur ~3 min). Je n'ai ni relancé Orca ni rien arrêté. La Delivery
  `delivery_3c77539940a7` (interlude distributeurs de b1, **déjà traitée**, PC26) n'est **pas acquittée** : elle sera rejouée au retour
  du runtime, l'acquitter sans rien refaire. Les workers continuent ; leurs messages s'accumulent dans la boîte du run.
  Runtime revenu avant 04 h 22 (Delivery `delivery_3c77539940a7` acquittée sans rien refaire).
- **PC28 — jeu 29, 04 h 23 UTC : URGENT, Mac sur batterie** (`pmset -g batt` : 12 %, en décharge, ~10 min ; signalé par b0 à 04 h 22).
  Effet mesuré par b0 : FPS d'affichage de Chrome 85 → 30 sur batterie (économie d'énergie) → mesures S1-5 sur batterie **non valables**.
  Actions : message haute priorité + `terminal send` aux trois workers du Mac (b0 `ctx_6a4cc401a3bf`, b1 `ctx_06b6a816308f`, b3
  `ctx_a382e2c73ff7`) : commit WIP et push maintenant, aucun build long avant mon feu vert ; carte b0 `--unread` « brancher le Mac ».
  Si le Mac s'éteint : au retour, relire `worker-list`, la queue des terminaux et les branches sur origin avant toute relance.
- **PC29 — jeu 29, 04 h 25–04 h 30 UTC** (batterie 10 %, toujours en décharge) : **WIP poussés** : b0 `polissage-perf-web` `353061c2`
  (compteur FPS `fps_counter.rs`, `?perf=1` / F8 / `ALACOD_FPS`, page `perf-duo.html`, séries Chrome solo zombies/throne en release :
  affichage 85, simulation 60, `advance` 0,5–1,1 ms en moyenne, 10–11 ms au pire ; reste sur secteur : 2 joueurs — WebRTC local
  impossible, seule interface `utun1` du VPN proposée à Chrome, repli 2 joueurs locaux — et avec/sans interpolation) ; b1
  `polissage-throne-visuel` `fa2081bb` (audit `a19fa800`, `scripts/repack-throne.py`, licences, captures du repli public).
  **b3 S2-8 livré** (`worker_done` 04 h 24, `polissage-lisibilite-coop` `cc25298f`, rebasé sur l'intégration avec S2-7) : vidéo de
  recette sur le Mac → **D66 et D67 fermées** (tourelle bleutée, immobile, tournée vers le joueur ; salve en huit rayons courts, aucun
  cercle plein) ; throne : `player_colors`, `legend` F7, `names`, halo et visée par les données seules, `legibility.rs` inchangé,
  § 95, captures 2 et 4 joueurs `docs/captures/polissage-lisibilite-coop/` ; 810 scénarios sans BLESS (avant rebase) ; `make check`
  après rebase non relancé (batterie) → **à faire au merge**. Worker libéré ; carte `in-review`. **Suite b3 : S-0b** (spec prête),
  à lancer quand le Mac est sur secteur.
- **PC30 — jeu 29, ~04 h 35 UTC** : **b1 S2-10 livré** (`worker_done` 04 h 26, `polissage-throne-visuel` `2cf02b75`) : surcharge locale
  Aymen B + Ittai (`scripts/repack-throne.py`, rien commité), repli public inchangé, § 96, captures 1/2/4. Après, avec les packs :
  pilote, ennemis (9 sur 5 peaux séparées par l'échelle), **tourelle propre** (zombie tourelle d'Ittai), roi rat dérivé (Zombie 4 ×1,4),
  projectiles (une image par équipe), 6 armes au sol sur 13, power-ups, coffres, murs (`wall_art`) et sol : propres. Restent **absents
  ou empruntés** : **portail** (aucun dans les packs, carré violet), 7 armes au sol sur l'image générique, armes portées sur 3 images
  pour 13, `slash`, ombre, objets au sol ; aucune attaque (pas dans les packs). Réestimation de b1 : 3,5–4 j, livré en ~1 h après
  l'audit (à vérifier au merge). **Non relancés** (batterie) : `test_crates` et les 810 scénarios → **au merge**. Worker libéré (S1-9
  étape 2 sera une nouvelle tâche après le merge de S1-7). Question à William (13) : portail en CC0 comme les distributeurs ?
- **PC31 — jeu 29, 16 h 33–17 h 05 UTC : REPRISE par un nouveau coordinateur** (`term_39a7c2cc-…`, `run-use` → `consumer_generation` 2).
  L'ancien terminal a vécu jusqu'à **13 h 24 UTC** au moins (pas 05 h) : il a reçu le `worker_done` de b2 (**lot 0 + S1-7 livrés**,
  `relay_75b29a1daf3e` 13 h 23, `polissage-sons-evenements` `28e3a710`, `make check` et `test_scenarios` sans BLESS verts côté b2), fait
  le merge `--no-ff` **`19ede770`** (S1-7) dans l'intégration (non poussé, sans `make check`), donné le feu vert secteur à b0 (13 h 24),
  puis commencé le merge de S1-6 (`d70be914`) et s'est arrêté **au milieu des conflits** (`feedback.rs`, `lib.rs`, `taches.md`).
  (1) **Disque** : 38 Go libres → `target-sweep` de `.target-mac` (aucun cargo d'alacod en cours ; seul un `cargo test` de harmonium,
  hors run) : 123 → 112 Go, 16,2 Go libérés ; suppression de `b0/target/headless` (18 Go, résidu d'avant la cible partagée ; le
  serveur E4 PID 22332 sert `b0/website/static`, pas `target/`) → **70 Go libres**. `commun.md` : cible partagée **obligatoire** sur le
  Mac, seuil 30 Go. (2) **Intégration** : merge S1-6 interrompu **annulé** (`git merge --abort`, à refaire) ; **`a892c9a5`** = merge de
  `origin/main` (X-4b, `2011d76b` ; **`origin/main` ne contient pas notre `main` local** `2ff27132` : base commune `00dc83ce`). Conflits :
  `PresentationClip` une seule définition (`crates/animation` : `clip_timing`, `advance_frame`, `Option<&PresentationClip>` de X-4b) ;
  poses `Shoot` (X-4b) et `Attack` (S1-8) gardées, **`present_shooting` exclut les ennemis** (`Without<MonsterState>` : les ennemis
  portent un `WeaponInventory`, contrairement à ce que disait le commentaire de S1-8, les deux systèmes non ordonnés se seraient disputé
  `PresentationClip`) — correction de fusion du coordinateur, à relire par b1 ; conventions § 91 avant § 92–93 ; D156–D157 avant D158+ ;
  journal des deux côtés. Trace neuve venue d'X-4b : `sprite_import_fantassin.trace` (testbed, déjà sur `origin/main`). Poussée
  (`origin/polissage-integration` = `a892c9a5`, base des lots nouveaux). `make test_tools` **vert sur le VPS** (`/tmp/chk-int`, 38 tests) ;
  `make format check_forbidden check_rollback_registration check_sim_boundary test lint` en cours sur le Mac (`/tmp/integration-check-5.log`).
  (3) **Workers** : b0 S1-5 `worker_done` accepté (15 h 51, `b2e326a0`, perf : 60 fps de simulation à 1 et 2 joueurs locaux, wasm 48/41 Mo
  au-dessus des 25 Mio de Pages ; non mesuré : 2 onglets en ligne, 120 Hz) ; sa question de 15 h 25 close (dispatch inactif) ; release
  refusée **`user_takeover`** (William a pris le terminal b0 : laissé retenu). Anciens dispatches b1 retenus : `ownership_transferred`.
  Delivery `delivery_3ed8be0dea17` acquittée. **Lancés** : b1 `task_0cd3c7f9799e` / `ctx_e4ef108942d8` (S1-9 étape 2 puis S2-11, spec
  `b1-s1-9-etape2-s2-11.md`, § 98, sonnet effectif) ; b3 `task_0225de1ce093` / `ctx_d77b637b132b` (S-0b **avancé à jeu 29**, spec mise à
  jour : Sonniss et throne inclus, § 99) ; b2 `task_fcad0d9ab6d0` / `ctx_475fa19f8960` (**S1-4 textes FR**, même terminal VPS
  `term_93e08106-…` réutilisé, spec `b2-s1-4-textes-fr.md`, § 100, pousser avant ven 30 midi pour S1-3). `commun.md` : son coupé sur le
  Mac, Sonniss au dépôt privé, base `origin/polissage-integration`, § 98–100. Copies VPS faites.
  **Suite** : `make check` vert → `main` `--ff-only` → **push `origin/main`** (décision a) ; puis fournée 2 (S1-6 refait, S1-9 étape 1
  `0355a2f7`, distributeurs `fbdc5924`), fournée 3 (S2-8, S2-10, S1-5) ; S1-3 ven 30 matin en Opus (b2-allumette) ; E4 de b0 ven 30.
- **PC32 — jeu 29, 17 h 05–18 h 00 UTC** : (1) **`make check` vert sur `a892c9a5`** (`/tmp/integration-check-5.log`, EXIT 0, `test
  scenarios ... ok`, 38 min) + `test_tools` VPS → **`main` = `a892c9a5`, poussé sur `origin/main`** (décision a ; fichiers de William
  intacts). (2) **Fournées 2 et 3 en un seul passage** : `6c650c10` S1-6 (conflits `content/feedback.rs` et `lib.rs` additifs : contrôles
  de sons S1-7 + `check_shot`/`check_death` S1-6, deux modules), `f16f1930` S1-9 étape 1 (notes de William : deux sections),
  `0edef129` distributeurs, `6616909c` S2-8 (D167–D168), `197ee8a6` S2-10 (§ 96), `418d6e9e` S1-5 (§ 97), `15c8fc51` ordre `cargo fmt`
  des modules → **`make check` vert** (`/tmp/integration-check-6.log`, EXIT 0, 30 min ; le premier passage avait échoué sur `format`
  seul) + `test_tools` VPS → **`main` = `15c8fc51`, poussé**. Seule trace neuve : `feel_zombies.trace` (S1-6, la sienne).
  **Mergés et poussés : lot 0, S1-1, S1-8 (+ distributeurs), S2-7, X-4b, S1-7, S1-6, S1-9 étape 1, S2-8, S2-10, S1-5.**
  (3) **b3 S-0b livré** (`7a72dfa2`) ; relu : le nightly `main.yaml` aurait publié les images TheLazyStone sur le site dès le retour du
  runner (licence « No Redistribution », confirmation de l'auteur pour un build web public **jamais obtenue**) → **retouche** sur le même
  terminal (`task_426288b45142` / `ctx_fd51b0a3e430`) : étape gardée par le secret **et** `vars.ALACOD_WEB_PRIVATE_ASSETS == '1'` (non
  posée ; run de preuve `actions/runs/40922436192`), dette renumérotée **D169** (D167–D168 prises par S2-8), `feedback.local.ron` de
  zombies ajouté à `alacod-assets` (`1beb8f5`, 169 fichiers) → livré `8087bd55`, worker **libéré**. Runner `vps-alacod-builder` **hors
  ligne** (constaté par b3, `gh api`). (4) **b1** : S1-9 étape 2 prête (`522ebdc5`, `feedback.local.ron` lu par le registre, lint
  avec/sans, vidéo 60 s muette hors dépôt, protocoles d'écoute 1 et 2) ; S2-11 en cours sur `polissage-throne-sons` ; accord donné pour
  les clés `shot_<arme>`/`reload_<arme>` et `mutation` (fichier de b2, S1-7 mergé), roi rat sans `BossState` laissé (simulation),
  dettes **D170–D172** à b1. (5) **b2** S1-4 : `polissage-textes-fr` `71eea973` poussé tôt (module `ui/texts.rs`), `make check` en cours.
  (6) Worktree VPS `b3` (S2-7 mergé) **supprimé** (`orca worktree rm`, branche gardée). `.target-mac/wasm32-unknown-unknown` (19 Go,
  aucun build wasm en cours ; les builds E4 sont déjà sous `b0/website/static/builds`) supprimé : 33 → 52 Go libres.
  (7) **Intégration** : `62ff6365` S-0b (statuts `assets.yaml` des packs non redistribuables posés au merge), `4cea5046` S1-9 étape 2 ;
  `make check` en cours (`/tmp/integration-check-7.log`).
- **PC33 — jeu 29, 18 h 00–18 h 40 UTC** : **`make check` vert sur `4cea5046`** (`/tmp/integration-check-7.log`, EXIT 0, `test scenarios
  ... ok`) + `test_tools` VPS OK → **`main` = `4cea5046`, poussé** (S-0b + S1-9 étape 2 en plus). Worktree Mac `b3` **supprimé** (S2-8 et
  S-0b mergés, branches gardées). Carte du **checkout principal** `--unread` : cinq questions groupées pour William (confirmation TheLazyStone
  avant de poser `ALACOD_WEB_PRIVATE_ASSETS`, runner hors ligne, portail/armes CC0 de throne, achat Aymen B, heure de l'E4). **b0 lancé** :
  `task_e358d7467e87` / `ctx_dee8ef21780d` (spec `b0-e4-s2-9.md` : prépare l'E4 et sa carte, S2-9 zombies/throne en attendant, l'E4
  interrompt dès que William arrive ; audio navigateur de S1-7 joué par William). Reste aujourd'hui : b1 S2-11, b2 S1-4 ; **S1-3 ven 30 au
  matin** (Opus, `b2-allumette`).
- **PC34 — jeu 29, 18 h 40–18 h 50 UTC** : **b1 `worker_done`** (S1-9 étape 2 `522ebdc5` + **S2-11** `2c571294` : 12 armes de throne en 4
  familles, clés `shot_<arme>`/`reload_<arme>` et `mutation`, `feedback.local.ron` lu en wasm sous le drapeau, D170 roi rat sans `BossState`,
  D171 tirs d'énergie = fusil changé de hauteur ; `make … test_crates test_scenarios_par lint` EXIT 0 sans BLESS) → branche basée sur `main`
  `4cea5046`, merge `--no-ff` **`facb7c92`** à arbre **identique** à la branche (`git diff 2c571294 HEAD` vide, donc sa preuve vaut) +
  `test_tools` VPS → **`main` = `facb7c92`, poussé**. Worker b1 **libéré** ; worktree b1 **gardé** (les protocoles d'écoute de William y
  lancent le jeu ; carte `ÉCOUTE William` posée par b1). **Dépôt privé** : 28 extraits Sonniss de throne + `throne/ui/feedback.local.ron`
  copiés par le coordinateur depuis `b1/games/throne/assets/` (procédure du README) → `alacod-assets` `a14f7d3` (208 fichiers, dépôt
  `private` vérifié par `gh api`). Runner `vps-alacod-builder` toujours `offline`. **b3 relancé sur le VPS** (worktree neuf `b3`) : S1-10
  points 1–2 (audit + campagne avant zombies, spec `b3-s1-10-avant.md`, dettes D173–D175), `task_3f1633289eed` / `ctx_05d1308bf550`.
  **Mergés et poussés : tout sauf S1-4 (b2, en cours) et S1-3 (ven 30).**
- **PC35 — jeu 29, ~20 h 30 UTC** : réponse à b0 (S2-9) : **un seul `budgets.ron`, calibré sur le VPS** (pas de `budgets.macos.ron`) ;
  **b0 `worker_done`** : S2-9 `e2138dfc` (bench strict vert sur le Mac sous charge, pire rollback 12,7 ms zombies / 6,1 ms throne, D70
  250 µs/frame, quatre plafonds provisoires d'après le relevé VPS ; **96 scénarios sans budget** attendent trois passes au calme sur le VPS,
  liste `docs/taches/rapports/polissage-perf-budgets/sans-budget-vps.txt`) ; E4 **préparé, non tenu** (`polissage-e4` `076aaa2b` : build
  `e4-sons` repli public servi sur `:8777`, protocole en dix lignes, tableau des notes vide, carte `--unread`). Merges : `perf-budgets`
  (arbre identique à la branche vérifiée) puis `e4` (docs) → **`main` = `b170c026`, poussé** ; `test_tools` VPS OK. Worker b0 **libéré** ;
  worktree b0 gardé (serveur E4). **E4 ven 30 = nouvelle tâche b0 quand William arrive** (spec `b0-e4-s2-9.md` § 1, notes à remplir).
- **PC36 — ven 30, 02 h 48–03 h 15 UTC** : **ménage du studio** (demandé par William) : `.target-mac/headless` (75 Go) supprimé, debug et
  release gardés → Mac **118 Go libres** ; le prochain `make check` recompile `headless` (~25 min). Règle : un worktree part dès que son dernier lot
  est fusionné ; `scripts/target-sweep.sh` entre deux `make check` ; **30 Go libres minimum des deux côtés**. Exception assumée : `b1` (123 Mo)
  reste jusqu'aux écoutes de William (les protocoles de `polissage-zombies-sons.md` et `polissage-throne-sons.md` § 4 y lancent le jeu avec les
  surcharges locales). Messages en attente (livrés d'un coup, mes `check --wait` sortaient vides) : **b2 `worker_done` S1-4** (22 h 22,
  `polissage-textes-fr` `4a019f24`, `ui/texts.rs`, § 100, `make check` et `test_scenarios` sans BLESS verts, captures à faire sur le Mac) et
  **b3 `worker_done` S1-10 points 1–2** (02 h 28 : `avant_poste` 5/5 spawners dans `Depart` confirmé, 0 spawner/fenêtre dans 7 salles sur 9 ;
  `la_fonderie` 37 spawners, démenti ; campagne avant 4 × 100 graines, vague médiane 9/9/6/6, 0 desync, 19 blocages sur `avant_poste`, fusil
  jamais acheté, mitraillette de départ dominante ; D173–D175). **Intégration** : `a9c41166` = merge de S1-4 (conventions § 100 après § 98,
  journal ; branche basée sur `a892c9a5`, code sans conflit) ; poussée ; `make check` en cours (`/tmp/integration-check-8.log`). Workers : b2
  **libéré** ; b3 : release refusée `user_takeover` (terminal repris) → laissé. VPS : `b2/target` balayé 64 → 27 Go, **98 Go libres**.
  **Lancés** : **S1-3 en Opus** (`task_dc9718409401` / `ctx_0fc661962926`, `b2-allumette`, spec mise à jour : § 101, rebase sur l'intégration
  avec S1-4, priorité des builds) — processus `claude --model opus` constaté sur le VPS ; **b2 S1-10** points 3–5 (`task_7c41437ddedb` /
  `ctx_a63ac0405f91`, spec `b2-s1-10-zombies-map.md`, BLESS des traces **zombies seulement** avec preuve § 5, `nice`, § 102).
  **Leçon** : un `check --wait --types "worker_done,…"` ressort vide alors qu'une Delivery attend quand des **heartbeats** (type hors filtre)
  sont en tête de la file FIFO ; après une attente vide, faire un `check` **sans** `--wait` (même filtre), traiter, acquitter, puis reprendre.
- **PC37 — ven 30, ~03 h 50 UTC** : **`make check` vert sur `a9c41166`** (`/tmp/integration-check-8.log`, EXIT 0, `test scenarios ... ok`,
  33 min avec la recompilation de `headless`) + `test_tools` VPS OK → **`main` = `a9c41166`, poussé** (S1-4 en plus). Mac 88 Go libres.
  **Mergés et poussés : lot 0, S1-1, S1-4, S1-5, S1-6, S1-7, S1-8 (+ distributeurs), S1-9, S2-7, S2-8, S2-9, S2-10, S2-11, S-0b, X-4b, E4 préparé.**
  En cours : S1-3 (Opus), S1-10 (b2). Restent : E4 (b0, avec William), écoutes (William), S2-12, S1-11, captures FR de S1-4 sur le Mac.
  **b1 lancé** : captures des textes FR de S1-4 en repli public (`task_5feb7a654dbc` / `ctx_090e4cd8a414`, spec `b1-s1-4-captures.md`,
  branche `polissage-textes-fr-captures`). Vidéo 60 s de throne : **faite** dans S2-11 (§ 5 de `polissage-throne-sons.md`).
- **PC38 — ven 30, 03 h 31 UTC : aperçu J3 de S1-3** (`polissage-reseau-amis` rebasée sur `a9c41166`, rapport § 2) : sondes de l'annexe A
  identiques au 28 (allumette `23524d8`, TURN UDP+TCP, prod `v0.0.2-4630291a` `online: true`) ; **D116 fermée** (fenêtre d'admission ré-armée
  à chaque pair fantôme → borne cumulée `ADMISSION_MAX_SECS` = 10 s, test) ; fin de session à 30 min : retour au salon, **pas d'écran figé** ;
  trouvé et corrigé un écran figé **avant** la partie (socket refusé, ticket périmé) → retour au salon (pas dans la prod de samedi) ; ticket 60 s :
  le socket s'ouvre après le wasm (10,7 Mo br) et les assets → correctif prêt sur `polissage-reseau-amis-ticket` `2a5573b1`, **non appliqué** ;
  protocole de samedi dans `docs/jouer.md` ; D165 (natif sans TURN). **Heure proposée samedi : 14 h heure de l'Est = 18 h UTC, 45 min** (carte
  `--unread` de b2-allumette ce soir).
- **PC39 — ven 30, ~03 h 50 UTC** : **b1 captures FR livrées** (`polissage-textes-fr-captures` `c66ef2d7`, 13 PNG en repli public ; signaling
  public refusé et Docker arrêté → points 4–5 sur un `matchbox_server` local) → merge docs **`412d438c`, `main` poussé**. Constats : **glyphes
  accentués manquants** (cases vides) au salon et sur « PARTIE INTERROMPUE », « Retour au salon » sur deux lignes, « kills »/« frames »/« machine_gun »
  en anglais → **S1-4 suite** confiée à b1 sur le même terminal (`task_797dad861e9c` / `ctx_e407f6a80e67`, branche `polissage-textes-fr-suite`,
  captures « -apres », présentation seule). À noter pour samedi : le signaling public a refusé une connexion **native** du jeu depuis le Mac
  (attendu ? le service n'accepte que `alacod.estran.studio`/`alacod.pages.dev` en CORS et un ticket) — S1-3 le sait (§ 1 de son rapport).
- **PC40 — ven 30, 11 h 00–11 h 30 UTC** : **b2 aperçu S1-10** (10 h 59) : `avant_poste` : 2 à 4 spawners + 1 à 2 fenêtres dans chacune des 7
  salles (script LDtk) ; armes : pistolet 22 × 3/s, mitraillette 7 dispersion 0,07, pompe 20 par plomb portée 320, fusil 55 × 3/s prix 1500 ;
  campagne après lancée, puis BLESS zombies + preuve § 5. (Branche locale `s1-10-local` : `b3-2` tient encore `polissage-zombies-map-equilibrage`.)
  **b1 S1-4 suite livrée** (`08d0ef81`) : cause des cases vides = `TextFont` sans police sur le salon en ligne et « PARTIE INTERROMPUE » →
  `texts::UI_FONT` pour tous les jeux ; bouton sur une ligne ; résumé « vague 1 · 0 éliminés · 500 points / 0:23 » ; « mitrailleuse » via
  `hud.ron names` ; 13 captures `-apres` ; preuve du worker (format, forbidden, rollback, sim_boundary, test_crates, lint, test_scenarios 4/4
  tranches sans BLESS) sur un arbre **identique** au merge + `test_tools` VPS → **`main` = `a2ade254`, poussé**. Worker b1 **libéré**.
- **PC41 — ven 30, 12 h 40 UTC** : attentes vides depuis 11 h 30 → inventaire : **b2 S1-10 au repos depuis 07 h 36 UTC** (campagne « après »
  finie : `b2/target/metrics/polissage-zombies-map-equilibrage/apres-*.json`, plus aucun processus ; l'agent croyait encore « 3 shells »)
  → réveillé par `terminal send` (suite : tableau comparé, BLESS zombies + preuve § 5, `make check`, `worker_done`). **S1-3** : J3 fini
  (`make check` et `test_scenarios` verts, aucune trace), rappel interne à 21 h 47 UTC pour poser la carte `--unread` de samedi (18 h UTC,
  45 min, deux machines avec Chrome dont une sur le partage de connexion d'un téléphone). VPS au calme, 53 Go libres.
- **PC42 — ven 30, 13 h 15 UTC** : la campagne relancée par b2 à 12 h 53 était **morte au départ** (`alacod-sim: error while loading shared
  libraries: libbevy_dylib.so` : `make test_scenarios` l'avait reconstruit avec `game/dev-dylib` ; les `.json` d'avant écrasés) → signalé par
  `terminal send` ; b2 a reconstruit sans dev-dylib, **4 `alacod-sim` vivants depuis 13 h 20** (logs qui grossissent). b2 a aussi trouvé que la
  première campagne « après » était fausse : les vagues choisissaient les spawners par distance sans regarder la salle (vague 1 seulement) →
  zones d'activation `active_x/y/width/height` sur les spawners des 7 salles (graine 1 : vague 9, 105 kills). Fin attendue vers **16 h 20 UTC** :
  vérifier alors que l'agent repart (il s'est déjà endormi une fois sur des shells finis). Leçon pour les campagnes : **vérifier le processus et le
  `.log` après tout lancement** (`pgrep -fa alacod-sim`).
- **PC43 — ven 30, 16 h 50 UTC : bilan « vendredi 17 h, LIVRÉ ou pas par voie »** (`main` = `a2ade254`, poussé) :
  | Voie | LIVRÉ (mergé dans `main`) | Pas livré |
  |---|---|---|
  | b0 | S1-1, S1-5, S1-6, S2-9 | E4 (attend William ; build `e4-sons` prêt sur `:8777`) |
  | b1 | S1-8 (+ distributeurs CC0), S1-9, S2-10, S2-11, captures et correctifs de S1-4 | écoutes de William (carte b1) |
  | b2 | lot 0, S1-7, S1-4 | **S1-10** : campagne « après » finit vers 17 h UTC (3 sur 4 faites), puis BLESS zombies + preuve § 5 + `make check` |
  | b2-allumette | — | **S1-3** : J3 fait (D116 fermée, pas d'écran figé, protocole) ; recette **sam 31 18 h UTC** ; session zombies dim 1 |
  | b3 | S2-7, S2-8, S-0b ; S1-10 points 1–2 (prêts, sur la branche de S1-10) | — |
  Carte du checkout principal `--unread` mise à jour (bilan + 5 questions groupées). Leçon : `pgrep -f motif` via `ssh … '…'` se compte
  lui-même (la ligne de commande contient le motif) → `pgrep -fa "[t]arget/…"`.
- **PC44 — ven 30, 18 h 10–19 h 10 UTC : limite hebdomadaire de l'abonnement Claude atteinte sur le VPS** (b2 S1-10 bloqué sur « You've hit
  your weekly limit · resets Oct 31, 10pm (UTC) » ; `claude -p` sur le VPS renvoie la même chose) → **la recette S1-3 de samedi 18 h UTC
  tombait avant le reset**. Constat : sur le VPS, `omp --model anthropic/claude-opus-5-5` **retombe silencieusement sur `openai-codex/gpt-6-sol`**
  (transcript `provider openai-codex`, barre d'état « GPT-6-Sol ») alors que `launch.effective` dit Opus → ne jamais croire `launch.effective`
  seul, lire la barre d'état ou le transcript. Actions : `worker-stop` des deux workers Claude bloqués (`ctx_0fc661962926` S1-3,
  `ctx_a63ac0405f91` S1-10 ; branches propres et poussées : `da5377be`, `c84ef0f1`) ; S1-3 relancé une fois sur le VPS en omp (`ctx_43517dfa87fb`,
  GPT-6-Sol en fait) puis arrêté ; **S1-3 relancé sur le Mac** : worktree neuf `~/orca/workspaces/alacod/b2-allumette-mac`, omp
  `amazon-bedrock/global.anthropic.claude-opus-5-5` (**« Opus 5.5 (Global) »** lu dans la barre d'état), `--retry-of`, `ctx_85a185910b98` ;
  message de placement envoyé (la tâche d'origine nomme le VPS). **S1-10 relancé sur le VPS en omp** (`ctx_eb0edd806537`) : tourne sous
  **GPT-6-Sol**, gardé (lot de contenu, preuves mécaniques) et signalé à William sur la carte. Worktree VPS `b2-allumette` **supprimé** (S1-3
  continue sur le Mac ; 40 Go de target) → VPS 89 Go libres. Spec S1-3 : section « Reprise … sur le Mac ». **Règle pour les prochains
  lancements** : sur le Mac, `--agent omp --model amazon-bedrock/global.anthropic.claude-<opus|sonnet>-5-5` (l'agent `claude` d'Orca y lance omp
  sur l'abonnement Anthropic, peut-être le même compte limité) ; vérifier le modèle dans la barre d'état.
- **PC45 — ven 30, 19 h 15 UTC** : **S1-3 (Mac, Opus) a repris** : carte `--unread` posée sur `b2-allumette-mac` (recette **sam 31 18 h UTC**, 45 min,
  deux Chrome dont un sur partage de téléphone, accès alumette pendant le test) ; il vérifiera dans Chrome **muet** le correctif du ticket et le
  retour au salon ; pas de rebase avant la fin du lot (merge de l'intégration dimanche). **S1-10 (VPS, GPT-6-Sol)** : comparatif provisoire —
  `avant_poste` 1 bot **67 → 94 parties plafonnées (régression)**, 2 bots 62 → 54 ; `la_fonderie` victoires 67 → 74 / 88 → 87 ; 0 desync ;
  un dernier zombie collé à son `steering_point` hors de vue (graine 2, zombie 385) ; fusil toujours jamais acheté. **Consigne** : pas de
  livraison avec la régression (le zombie bloqué bloque aussi un humain) ; chercher si les nouveaux spawners/fenêtres posent des zombies dans des
  cases sans issue (`nav_stats`, `nav_probe`), corriger la carte, relancer `avant_poste` 1 et 2 bots ; objectif plafonds ≤ avant ; aucun BLESS
  avant ; moteur de navigation intouchable sans accord. Protocole des 15 min de William : `ALACOD_MUTE=1 make zombies GARGS="--seed 42"`, puis
  pistolet / mitraillette / pompe / fusil 5 min chacun (à grouper dimanche, sans écoute : William décide du son).
- **PC46 — ven 30, 18 h 54 UTC (reçu 19 h 40)** : **S1-3 vérifié dans Chrome muet sur le Mac** (`polissage-reseau-amis` `8dd4d560`, rapport § 2.7,
  banc local allumette `23524d8` + site `npm run dev` + wasm avant/après) : avant correctif, à 10 Mbit/s, ticket périmé → socket refusé → retour au
  salon à +100 s (pas d'écran figé) ; **après** (`2a5573b1`) : ticket reposé à +30/+60/+90 s, partie synchronisée → correctif vérifié ; retour au
  salon vérifié (expiration en partie, avant connexion, arrêt serveur), 0 DESYNC. **D166** : la connexion WebRTC prend 40–41 s (le fork matchbox
  attend la fin de la collecte ICE ; le Mac sort par un VPN `utun5` qui bloque l'UDP STUN/TURN) → **protocole de samedi : VPN coupé** (`docs/jouer.md`,
  carte mise à jour). Le correctif du ticket reste hors de la prod de samedi (prochain tag, décision de William).
- **PC47 — ven 30, 20 h 15 UTC** : **S1-10, cause du zombie bloqué** (`nav_probe`, graine 2 f5000) : zombie 385 immobile à (144, 32), `steering_point`
  = sa position, fenêtre 54 cassée juste dessous, goulet de 2 cases autour de la fenêtre **d'origine** de `Depart` (pas un spawner neuf) : le guidage
  pose la cible sur le bord et le zombie s'arrête une fois la fenêtre cassée. Accord : élargir les ouvertures de fenêtres d'`avant_poste` dans la
  carte (collider gardé, aucune mécanique), rejouer 3 graines puis la campagne `avant_poste` 1/2 bots ; **dette moteur D176** (b2) ouverte pour le
  défaut de guidage (correctif = simulation de tous les jeux, hors lot).
- **PC48 — ven 30, 20 h 40 UTC : escalade de S1-10** : ouvertures de fenêtres élargies à 4 cases → graines 2, 3, 45, 76 bloquent encore (zombies qui
  visent le bord du collider d'une fenêtre intacte sans la frapper) ; « pas résoluble par la carte seule ». **Décision du coordinateur** : correctif du
  guidage/de la sélection de cible autour des fenêtres = **lot moteur à part** (D176), branche `polissage-nav-fenetres`, **nouvelle voie b4 sur le
  Mac en Opus Bedrock** (`task_432a48f21fa0` / `ctx_a1fe470a7fbb`, worktree `~/orca/workspaces/alacod/b4-nav-fenetres`, spec `b4-d176-nav-fenetres.md`,
  § 103) : test avant/après, BLESS des traces de zombies seulement avec preuve § 5, toute autre trace = arrêt. b2 : annule le contrefactuel, garde
  spawners/fenêtres/zones/armes, pousse la reproduction dans D176, puis attend ; après le merge de b4 il merge l'intégration et relance la campagne.
  Ce n'est pas une mécanique nouvelle (bug qui bloque aussi un humain) ; à dire à William avec le résultat.
- **PC49 — ven 30, 21 h 15 UTC : b4 D176** : cause = `flow_direction` rend une direction **nulle** quand le point visé est à moins de 1/16 px sans être
  atteint → zombie figé (100 à 566 frames dans 9 scénarios zombies). Correctif : 9 traces zombies bougent (toutes à la 1re frame du cas dégénéré,
  preuve § 5 prête) ; `avant_poste` (carte de b2) graines 2/3/45/76 : bloquées vague 2–3 avant, **vague 9 et kills continus après** ; `la_fonderie`
  10 graines 1 bot : 9 victoires / 1 plafond / 0 mort → 7 / 0 / 3 (les zombies ne gèlent plus : jeu un peu plus dur, S1-10 rééquilibrera).
  Décisions : `la_fonderie_demo` → **nouvelle graine** où le bot gagne (attentes gardées, frames recalées) ; `two_players_idle` attentes recalées
  (mort f1346 au lieu de f1570) ; BLESS des 9 traces zombies avec preuve § 5 ; aucune autre trace.
- **PC50 — ven 30, 22 h 15 UTC** : **disque du Mac à 13 Go libres** (99 %) : b4 compile en `headless` ; `.target-mac/debug` (71 Go, aucun cargo
  dessus) et `wasm32-unknown-unknown` (13 Go) supprimés, `tmp` vidé → **115 Go libres**. S1-3 (Mac) attend samedi 18 h UTC (boucle de `check`
  jusqu'à 17 h 50) ; une fenêtre modale d'omp (« rubber duck ») bloquait son écran → fermée par Échap. b4 : `make check` en cours. b2 S1-10 attend b4.
  Shell du coordinateur : une attente d'une heure a perdu son backend (sortie vide), sans effet.
- **PC51 — ven 30, 22 h 40 UTC : D176 livré et mergé** (b4, `polissage-nav-fenetres` `8fc87958`) : `flow_direction` vise la case d'après quand le
  point visé est trop proche (`normalize_or_zero` rendait une direction nulle à 0,004 px du coin) ; `enemy_target_selection` frappe une fenêtre
  intacte que le corps chevauche ; scénario `avant_poste_fenetre_coin` + test unitaire (échouent avant) ; graines 2/3/45/76 d'`avant_poste` (carte
  de b2) : vague 2–3 → vague 9. **BLESS de 9 traces**, toutes de jeu `zombies` (vérifié : `game: "zombies"` ou défaut `zombies` de `replay.rs`) :
  `bench_horde`, `clone_solo`, `downed_all_lose`, `downed_bleedout`, `idle`, `la_fonderie_demo` (graine 1), `movement_melee`, `run_lose_summary`,
  `two_players_idle` (recalé), preuve § 5 au rapport ; `schedule_order_diff` OK, fuzz sans écart, 813 scénarios en 4 tranches, `make check` vert
  sauf `test_tools` Mac. Merge `--no-ff` à arbre **identique** à la branche + `test_tools` VPS → **`main` = `79538914`, poussé**. Worker b4 libéré,
  worktree `b4-nav-fenetres` supprimé. b2 relancé sur S1-10 (merge de l'intégration, campagne après avec D176, BLESS zombies, `make check`).
- **PC52 — sam 31, 01 h 05 UTC** : S1-10 : merge `905b38ea` de l'intégration (D176 dedans), campagne `apres-d176-*` relancée 22 h 37 (4 processus
  vérifiés) ; `make test_scenarios` sans BLESS : **40+ traces zombies** divergent dès f0 (armes/cartes) et 10 scénarios ont des attentes invalidées
  (`ammo_burst`, clones, `dash_iframes`, `downed`, `feel_zombies`, `la_fonderie_demo`, `weapons_workout`) ; 6 traces `generated/zombies` régénérées,
  aucune trace d'un autre jeu. Consigne : attentes recalées seulement avec une ligne de cause chacune ; une attente de mécanique sans lien
  (`dash_iframes`, `downed`) se rejoue et ne se recale que si la cause est un nombre ; preuve § 5 « f0 » expliquée (champ haché au spawn).
- **PC53 — sam 31, 01 h 15–03 h 40 UTC** : S1-10 : attentes mécaniques rejouées (ammo, clones, `dash_iframes`, `downed_*`, `la_fonderie_demo`
  passent ; `feel_zombies` à réparer) ; cause des traces à f0 = `WeaponInventory` hache `WeaponConfig.firing_modes` des armes de départ
  (`crates/combat/src/weapons/mod.rs:254-260,518-527`) ; campagne après-D176 `avant_poste` solo : morts 31 → 5, plafonds 67 → 93 mais **blocages
  (InProgress, dernier kill > 4000 frames) 12 → 0** — le plafond est la borne de 20 000 frames (Spawning 70), pas un soft-lock → **critère accepté :
  blocages**, plafond et répartition affichés, rythme jugé par William. Puis **le worker du VPS (omp → GPT-6-Sol) s'est arrêté sur la limite
  d'usage de Codex** (`usage_limit_reached`, ~2,5 h d'attente) avec **65 fichiers non commités** → **commit et push du coordinateur**
  (`polissage-zombies-map-equilibrage` `88041423`, WIP non vérifié : traces de zombies seules, vérifié par `game:` des `.ron`), `worker-stop`,
  métriques (avant de b3, après et après-D176 de b2) copiées sous `~/orca/workspaces/alacod/captures/s1-10-metrics/` (30 fichiers, 18 Mo).
  **S1-10 repris sur le Mac** : worktree `~/orca/workspaces/alacod/b2-s1-10-mac`, omp **`amazon-bedrock/global.anthropic.claude-sonnet-5-5`**
  (« Sonnet 5.5 (Global) » lu), `--retry-of`, `ctx_671dd445443e`, spec : section « Reprise du sam 31 ». **Plus aucun agent utilisable sur le VPS**
  (Claude : limite hebdomadaire jusqu'à 22 h UTC ; Codex : limite d'usage ; Bedrock non configuré) : tout lot nouveau va sur le Mac.
  Worktrees VPS `b2` et `b3-2` gardés jusqu'au merge de S1-10 (branche poussée, métriques copiées), puis supprimés.
- **PC54 — sam 31, 04 h 30 UTC** : le worker S1-10 du Mac (`ctx_671dd445443e`) était lancé **sans `--profile womp`** (barre « π », « No API key
  found for amazon-bedrock ») alors que S1-3 et b4 l'avaient eu → arrêté, relancé à l'identique (`ctx_727cdaa8f379`) : `omp --profile womp --model
  …sonnet-5-5` cette fois (vérifié par `ps`). **Vérifier `ps … --profile womp` après chaque lancement omp.** S1-3 attend 18 h UTC (Opus, womp).
- **PC55 — dim 1 nov, 04 h 05–04 h 20 UTC : REPRISE** (le coordinateur était resté à l'invite après `getaddrinfo ENOTFOUND bedrock-runtime…` ;
  Bedrock répond de nouveau, vérifié par `omp -p` ; usage de William réinitialisé ; abonnement Claude du VPS réinitialisé sam 22 h UTC).
  **État réel** : (1) **S1-10** (`ctx_727cdaa8f379`, Mac) : worktree sur la branche locale `wquintal/vodyanoy` = **`88041423`** =
  `origin/polissage-zombies-map-equilibrage`, rien de modifié, à l'invite ; sur le VPS, `make test_scenarios` « final » **terminé (Terminated)**
  le 31 à 04 h 30 sans résultat ; `make check` « final » **rouge à `test_crates`** : `expectations.rs` `ammo_reserve_matches_starting_weapons_at_creation`
  (réserve Plomb 72, attendu 48 : `weapons.ron`) et `player_downed_true_once_fallen` (f100, joueur 1 pas à terre) ; brouillon du rapport § 3–4
  (`sec34.md`) et journaux copiés sous `~/orca/workspaces/alacod/captures/s1-10-metrics/` → réveillé par `terminal send` avec cet état (corriger les
  deux tests avec cause, `test_scenarios`, rapport § 3–5, `make check`, LIVRÉ). (2) **« Processus orphelin » du VPS** : plus aucun `make
  test_scenarios` ; restait un **watcher** `bash -c 'while pgrep -f "make check"…'` du worker S1-10 arrêté (worktree `b2`), en boucle depuis 23 h
  parce que `pgrep -f` se trouvait lui-même → **tué** ; le VPS ne fait plus rien pour ce run. (3) **S1-3** (`ctx_85a185910b98`, Mac, Opus) : recette de
  samedi **non tenue** (William bloqué) ; il bouclait jusqu'à sam 17 h 50 puis erreurs réseau → réveillé : carte à jour, merge de l'intégration, attente
  de l'heure que William choisira. (4) Worktree `b1` passé en `--detach origin/main` (`79538914`, contient S1-9/S2-11) avec ses surcharges locales,
  pour l'écoute. Serveur E4 `:8777` (PID 22332) répond 200. **Carte `alacod/main`** : liste du jour pour William (E4, recette deux réseaux = heure à
  donner, zombies à 2, écoute des sons, 15 min d'armes après S1-10) avec les commandes exactes, + questions ouvertes.
- **PC56 — dim 1, 04 h 30 UTC** : S1-10 : `make test_scenarios` sans BLESS **vert sur le Mac** (5 passed, 1133 s) ; les deux tests recalés par des
  nombres d'arme : réserve 48 → 72 (chargeur du pistolet 6 → 9 × 8), chute f100 → f101 (mitraillette 7 dégâts au lieu de 8). Reste preuve § 5,
  rapport § 3–5, `make check`, LIVRÉ.
- **PC57 — dim 1, 04 h 34 UTC** : S1-3 : carte `b2-allumette-mac` à jour (recette à replanifier, sans heure) ; intégration `79538914` fusionnée
  (`456db196`, D165–D166 + D176 gardés), poussée ; `make check` Mac vert sauf `test_tools` (cause trouvée : `scripts/claude-say.sh`, heredoc Python
  avec accents graves dans `$(…)`, illisible par le bash 3.2 de macOS ; vert sous bash 5.2 du VPS ; hors lot, à son propriétaire K5-0 `f630903f`).
  Attend l'heure de William.
- **PC58 — dim 1, 05 h 05–05 h 20 UTC : S1-10 livré et mergé** (`f2dd05a7`) : campagne avant/après (avec D176) 4 × 100 graines : parties
  **bloquées 14 → 1** (solo) et **5 → 0** (duo), morts 31 → 5, victoires `la_fonderie` solo 67 → 79, vague atteinte et durée des vagues inchangées,
  fusil toujours jamais acheté ; 0 desync (80 parties au synctest distance 2) ; **54 traces BLESS, toutes `zombies`** (48 `game: "zombies"`, 6 par
  défaut `zombies` : `downed_*`, `friendly_fire_*`, `immune_tag`, `stat_move_speed`), preuve § 5 par contrôles A/B (hash `WeaponInventory` à f0) ;
  code hors contenu : `crates/scenario/tests/expectations.rs` (deux nombres d'arme). Merge à arbre **identique** à la branche (`make -k check` du
  worker vert sauf `test_tools` Mac) + `test_tools` VPS → **`main` = `0c39eb1d`, poussé**. Worker libéré ; worktrees `b2-s1-10-mac` (Mac), `b2` et
  `b3-2` (VPS) **supprimés** (branches gardées) → VPS 131 Go libres, **aucun worktree alacod sur le VPS**. `target-sweep` de `.target-mac` (40 → 32 Go)
  → Mac 59 Go libres. Carte `alacod/main` mise à jour (S1-10 livré, liste du jour).
  **Restent** : S1-3 (attend l'heure de William), E4, écoutes, 15 min d'armes, S2-12 (cavernes throne), S1-11 (session d'amis ven 6), `test_tools` du Mac
  (`scripts/claude-say.sh` sous bash 3.2, K5-0).
- **PC59 — dim 1, 05 h 30 UTC** : `claude -p` répond de nouveau sur le VPS (quota réinitialisé) → **S2-12 lancé sur le VPS** (`task_0371393036f9` /
  `ctx_7567caf20ea9`, worktree neuf `b3`, branche `polissage-throne-map`, `claude --model sonnet` vérifié par `pgrep`, spec `b3-s2-12-throne-map.md` :
  audit des cavernes, causes de 4–5 soft-locks de S2-7, corrections de contenu, campagne après, BLESS throne seulement, **commit/push toutes les deux
  heures**, § 104, D177–D179). Le Mac reste libre pour William (E4, écoutes, recette S1-3).
- **PC60 — dim 1, 06 h 15 UTC : aperçu S2-12** : cavernes = automate cellulaire (sol 0,70/0,70/0,78, une chambre dominante, au niveau 3 les 9 spawns
  dans la même chambre) ; **les 5 soft-locks rejoués ont la même cause** : deux choix de mutation empilés entre étages (`choice_frames` 600) que les bots
  ne pressent jamais → transition retenue 1200 frames = exactement `FLOORS_SOFTLOCK_FRAMES` du runner → **faux positif du détecteur**. Proposition du
  worker `choice_frames` 600 → 300 **refusée** (temps de choix d'un humain changé pour masquer un défaut de mesure) ; à la place : catégorie « attente de
  mutation » ou exclusion dans le détecteur (outillage, sans trace qui bouge), sinon D177. `fill_ratio` 0,42/0,40/0,40 accepté (contenu, jugé par la
  campagne et une capture Mac) ; spawns du niveau 3 à répartir si c'est du contenu.
- **PC61 — dim 1, 15 h 05 UTC** : file : heartbeats seulement (S1-3 jusqu'à 07 h 05, S2-12 à 15 h 03 « reworking : revert choice_frames, fix detector,
  level-3 spawns »), acquittés. Le worktree VPS de S2-12 s'appelle **`b3-3`** (pas `b3`) : branche `polissage-throne-map` poussée à `856d3595`
  (« rapport, make check vert »), 2 fichiers en cours, campagne `alacod-sim` throne en route (processus vérifiés). S1-3 au repos, attend l'heure de
  William (carte `--unread`). Disques : Mac 53 Go, VPS 87 Go.
- **PC62 — dim 1, 15 h 10–16 h 05 UTC : CLÔTURE (consigne de William : on termine, rien de nouveau avant sa revue)**.
  (1) S1-10 : déjà livré, mergé et poussé (PC58, `0c39eb1d`), worker libéré, worktrees supprimés. (2) **S2-12 arrêté** (lancé en PC59, avant la
  consigne) : campagne et `cargo test` tués sur le VPS, `worker-stop`, travail en cours commité et poussé tel quel (`polissage-throne-map` `d8c53061`,
  WIP **non vérifié, non mergé**), worktree VPS `b3-3` supprimé. (3) **S1-3 clos « préparé, non recetté »** (`worker_done` 15 h 54, `polissage-reseau-amis`
  `8297b2ea`, poussée, propre ; `releases.json` inchangé, `online: false` pour zombies et throne ; journal `PRÉPARÉ` ; rapport § 4) → worker libéré,
  **worktree Mac `b2-allumette-mac` gardé pour la recette**. (4) **Rapport de fin de voie** `docs/taches/rapports/polissage-semaine-1.md` (issue et preuve
  par tâche) commité → **`main` = `d7978ee3`, poussé**. Vérifié : `worker-list` → aucun worker actif, aucun terminal à décider ; VPS : aucun processus
  du run, aucun worktree alacod, 131 Go libres ; Mac : 55 Go libres, le seul `cargo` actif est celui de harmonium. Worktrees Mac restants : `b0` (serveur
  E4 `:8777`, PID 22332), `b1` (écoutes, `main` détaché, surcharges locales), `b2-allumette-mac` (recette), `integration`. Carte `alacod/main` à jour.
  **État de la continuation : CLOSE — « semaine 1 livrée sauf S1-3 recette ».** Semaine 2 (S1-3 recette, S2-12, équilibrage throne, vidéos 60 s,
  S1-11, budgets VPS, dettes) : rien sans décision de William.
- **PC63 — dim 1, ~17 h UTC : build web « semaine1 »** (William « n'a vu aucune progression » : le site sert `v0.0.2-4630291a` du 10 oct, son checkout
  n'a ni la surcharge privée ni les sons). Build fait dans `b1` (`main` `79538914` détaché, surcharge à jour depuis `alacod-assets` `a14f7d3`, 198 fichiers) :
  `ALACOD_WEB_PRIVATE_ASSETS=1 WEB_BUILD_ID=semaine1 WEB_PROFILE=release WEB_ENABLE_SOLO=1 node scripts/build-web-games.mjs zombies throne` (13 min,
  « surcharge privée incluse » : 80 fichiers zombies, 118 throne ; `releases.json` remis), copié sous `b0/website/static/builds/semaine1/` → servi par
  `:8777` (PID 22332), HTTP 200 sur `loader.html`, `build.json` et `wasm_bg.wasm` des deux jeux. Essai headless : zombies a rendu une fois (pack visible,
  écran de défaite en français) ; les relances headless suivantes n'ont plus démarré, ni zombies ni throne (même le build `e4-sons`) → **arrêté sur
  consigne du studio** (pas la preuve demandée ; c'est William qui joue). Navigateurs gérés fermés. Son : pas de coupure par défaut dans le jeu (volume
  global 100 %, démarre au premier clic) → couper par Échap → Réglages → Volume ou en coupant le son de l'onglet ; dit sur la carte. Carte `alacod/main`
  : URL + ce qu'il doit voir. **Gungeon (lecture, non corrigé)** : départ dans le hub (`games/gungeon/assets/game.ron:56-58`, `hub.ldtk` fermé de murs) ;
  seule sortie = borne « porte » en (13, 6) (`hub/porte.ron` `EnterRun`) à activer avec **H** (clavier, `control.rs:86`) ou **Y / triangle**
  (`control.rs:133`) ; **aucune invite** (`ui/hud.rs:1683-1684` renvoie une chaîne vide pour `InteractionType::Hub`) → le joueur se croit enfermé.
  gungeon n'est pas dans `build-web-games.mjs` ni `releases.json` (zombies et throne seulement) : la version que William a jouée n'est pas celle du site
  officiel [INFÉRENCE : build local ou ancien]. Correctif évident (invite « Porte — H ») = semaine 2. **En attente des notes de William.**

### Lot S-0b — dépôt privé d'assets (**confirmé le 28** : b3 sur le Mac, J4 sam 31 ; spec `b3-s-0b-assets-prives.md`)

- **Quand / qui** : b3 sur le Mac, **J4 sam 31** (après S2-8 de J3 ; le créneau « campagne zombies avant » de b3 glisse à J5,
  S1-10 de b2 n'en a besoin qu'à J6). Après que b1 a fixé l'arborescence de surcharge (J2–J3). Sonnet, ≈ 0,5–1 j. b1 n'est pas retardé.
- **Contenu** : (1) dépôt privé `estran-studio/alacod-assets` (création par William, ou par le coordinateur sur son accord) ;
  (2) structure `<jeu>/…` = chemins de la surcharge sous `games/<jeu>/assets/` (`sprites/postapo/`, `sprites.local.ron`,
  `ui/hud.local.ron`, sons payants s'il y en a), `licences/<pack>/` (texte de licence, preuve d'achat, date), `README` ;
  (3) dans alacod, `scripts/assets-prives.sh` : clone/pull vers `~/orca/workspaces/alacod/packs/alacod-assets` et copie dans les
  chemins gitignorés ; (4) CI du site : étape qui clone le dépôt privé (clé de déploiement en lecture, secret
  `ALACOD_ASSETS_DEPLOY_KEY`) avant le build wasm ; sans le secret (fork, PR), le repli public se construit ; (5) `assets.yaml` :
  `statut: dépôt privé alacod-assets` pour chaque pack payant. **Preuve** : run de CI (cinq lignes) avec et sans le secret,
  build local avec et sans le script, lint vert dans les deux cas.
- **D161 (b1, 28 oct)** à régler dans S-0b : `build-web-games.mjs`, `embed-game-content.rs` et le `Makefile` copient `games/*/assets/`
  en entier ; un build web lancé depuis un checkout qui a la surcharge locale l'embarque. Avant le dépôt privé : aucun build web
  public depuis le Mac avec la surcharge ; S-0b rend l'inclusion explicite (avec le dépôt privé seulement) et le dit dans la preuve.
- **Fait par le studio le 28 (autorisation de William)** : dépôt `https://github.com/estran-studio/alacod-assets` (privé, `README`
  seul, `main`) ; clé de déploiement en lecture seule « ci-alacod-site (lecture seule) » sur `alacod-assets` (clés de déploiement
  activées au niveau de l'organisation) ; clé privée dans le secret `ALACOD_ASSETS_DEPLOY_KEY` d'`estran-studio/alacod`. Rien
  d'autre à attendre de William. D161 réglée dans S-0b (inclusion explicite de la surcharge au build web).

## Ordre de merge (§ 4, réduit au § 0)

`polissage-online-faux` → S1-7 → S1-9 → S2-7 (traces throne seules) → S1-8 → S1-6 → S1-3 → S1-1 → S1-4 → S2-8 → S1-5 / S2-9 → S1-11.
Chaque merge : `make check` vert sur la branche rebasée, preuve de la fiche lue, aucune trace d'un autre jeu modifiée
(`git diff --stat main...<branche> -- tests/scenarios`), `merge --no-ff`, ligne de journal et rapport présents.

## Preuves livrées

- **b1 aperçu J1** (audit « avant ») : `polissage-zombies-visuel` `199ce4ac`, `docs/taches/rapports/polissage-zombies-visuel.md` § 1.
- **b1 tableau avec le pack** (28 oct, 14 h 59 UTC) : `polissage-zombies-visuel` `28660d93`, rapport § 2 ; captures repli public
  `docs/captures/polissage-zombies-visuel/repli-public_{1j,2j,4j}_*.png` (+ `4j_legende`, `4j_tir`) ; captures surcharge (hors dépôt)
  `~/orca/workspaces/alacod/captures/polissage-zombies-visuel/surcharge-locale_*.png`. Aperçu de mi-parcours, pas encore livré.
- **b2-prep, préparation S1-3** (28 oct, 14 h 37 UTC, status « prête ») : `origin/polissage-reseau-amis` `98b923b0`,
  `docs/taches/rapports/polissage-reseau-amis.md` § 1 (docs seules). **Service sain** : TLS Let's Encrypt jusqu'au 2027-01-07,
  `/browser/v1` en 401 sans jeton, CORS `alacod.estran.studio` et `alacod.pages.dev` (pas `localhost`), Matchbox serveur 0.13.0 =
  fork client 0.15 au fil près (deux commentaires), image `23524d8` déployée par la CI le 2026-10-10, TURN coturn 4.6.3 alloué en
  UDP et TCP (TURNS 5349 fermé). Rien à redéployer. Écueils : `SESSION_TTL` 30 min (partie coupée), ticket de signaling 60 s
  délivré avant le téléchargement du wasm (risque sur partage de connexion, correctif `website/` prêt si J4 le montre), natif
  sans TURN (D164 candidate), prod `releases.json` = `v0.0.2-4630291a` `online: true` alors que le dépôt est resté à
  `rtc-fix-20261009` (b2 prévenu pour le lot 0). Plan J3–J5 et protocole J4 en dix lignes (§ 1.7). Relu par le coordinateur : bon.

## Décisions de William

Tranchées le 28 : packs (TheLazyStone payé, option A, puis dépôt privé d'assets créé par le studio), disque, deux minutes de manette
(faites : « ça marche assez bien »), préparation S1-3, option A et boss par donnée, S-0b (b3, sam 31), décisions 2, 3, 4 du § 0.3 par défaut.
**Tranchées le 29 (par le studio)** :
- (7) `SESSION_TTL` d'Allumette : **gardé à 30 min** (retour au salon entre deux parties ; à revoir plus tard).
- (8) Recette de samedi : **la prod actuelle `v0.0.2-4630291a`**, pas de nouveau tag.
- (1/9) Accès alumette (clé d'ovh-inner, Docker pour `debian`) : **William le configure samedi pendant le test** ; b2 avance sans d'ici là.
  Carte b2-allumette levée (`--read`).
- (6b) Distributeurs de perks : **source CC0** (pas CC BY) ; b1 propose 1–2 candidates et intègre la meilleure sans attendre
  (interlude demandé à b1 le 29, branche `polissage-zombies-distributeurs`).
**Tranchées le 29 après PC30 (brief de reprise, PC31)** :
- (a) **Plus de PR obligatoire sur `main`** : le coordinateur pousse `main` après chaque merge (`make check` vert + preuve de la fiche +
  aucune trace d'un autre jeu modifiée).
- (b) `origin/main` a reçu X-4b (`30526995`, `2011d76b`, poussé par le coordinateur 1837-cli) : le fusionner dans
  `polissage-integration` d'abord, `make check`, `main` `--ff-only`, push (fait : `a892c9a5`, PC31).
- (c) **Aucun son sur le Mac sans demande de William** (`ALACOD_MUTE=1`, Chrome muet) ; dans `commun.md`.
- (d) = (7), (8), (1/9) ci-dessus ; distributeurs CC0 livrés (`fbdc5924`).
- (e) = (11) **tranchée** : extraits Sonniss jamais dans le dépôt public → dépôt privé `estran-studio/alacod-assets` avec les images
  payantes (S-0b) ; repli public = Kenney/Snake's.

**Encore ouvertes** (le studio dit « rien d'autre en attente côté William pour les clones » : à confirmer qu'il les a reçues ;
défauts appliqués en attendant) :
- (10) **Déplacement analogique au stick** : William rejuge la rigidité à l'E4 de vendredi avec l'interpolation. Si c'est encore rigide :
  tâche à part (≈ 1 j, format de `BoxInput` changé, vieux clients incompatibles, `BLESS` de tous les jeux si le chemin numérique bouge).
  Défaut : rien.
- ~~(11) Sons Sonniss dans le dépôt public ?~~ **Tranchée le 29 (e)** : dépôt privé + `feedback.local.ron` (S1-9 étape 2), repli public
  Kenney/Snake's. Carte b1 à lever.
- (12) **Fenêtres de zombies** : Critical lisible seulement par la teinte. Défaut : acceptable (D163).
- (13) **Throne, classes sans image dans Aymen B ni Ittai** : portail (carré violet), 7 armes au sol sur 13 (mortier, lance-grenades,
  laser, plasma, traqueur, disque, arsenal), armes portées (3 images pour 13). Même règle que les distributeurs (source CC0, b1 propose
  et intègre) ? Défaut proposé : oui pour le portail et les armes au sol, armes portées laissées à 3 images.

Rendez-vous : E4 ven 30 (b0, dans S1-5), second réseau sam 31 (b2, S1-3, prod actuelle, accès alumette par William), zombies à 2 dim 1,
throne 30 min mer 4, session d'amis ven 6.
