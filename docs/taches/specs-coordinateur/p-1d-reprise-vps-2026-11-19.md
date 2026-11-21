# P-1d / T-3 — reprise sur le VPS (jeu 19 nov 2026, coordinateur alacod, run `run_9effecfcae39`)

Objectif studio O2 au service d'O3 (alpha coop du prologue de 1837, dim 20 déc). Jalon **S1 (dim 15, dépassé, passe avant tout)**, puis S2 dim 22.
Tu es un worker **engine** (simulation + présentation) ; le coordinateur alacod merge, toi jamais.

## À lire, dans cet ordre
1. `CLAUDE.md` du dépôt (GGRS, frontière simulation/présentation).
2. `~/orca/runs/alpha/commun.md` (consignes communes ; écrites pour le Mac : traduis avec la table ci-dessous, **ce fichier-ci prime**).
3. Ta passation la plus récente : `docs/taches/rapports/p-1d-passation-vps.md` (dans ton worktree) — état exact, prochaine commande, rouge attendu, observation sous Xvfb.
4. Ta spec : `~/orca/runs/alpha/p-1d-parite-observee.md` + `~/orca/runs/alpha/p-1d-passation-2026-11-16.md` (surtout « CHANGEMENT D'ORDRE (William, lun 16) »).
5. La preuve qui fait foi : `docs/taches/plan-1837-cli.md` § 6, lignes « P-1d » et « Changement d'ordre du 16 nov ».

## Où, sur le VPS
| Quoi | Chemin |
|---|---|
| Ton worktree (branche `wquintal/p-1d-parite-observee` @ `6fbeeb31`, suit `origin`) | `/home/debian/orca/workspaces/alacod/p-1d-parite-observee` |
| Ta cible cargo (une seule, la tienne ; aussi pour le build release du jeu 1837 : `CARGO_TARGET_DIR`) | `/home/debian/orca/workspaces/alacod/p-1d-parite-observee/target` |
| 1837 (checkout du coordinateur 1837 : **jamais d'écriture**) | `/home/debian/orca/1837` |
| Copie jetable de 1837 `9e7c6df` (à créer, détachée) | `git -C /home/debian/orca/1837 worktree add --detach /home/debian/orca/workspaces/1837/p-1d-jetable 9e7c6df` (à retirer à la fin par `git worktree remove`) |
| Banc du labo (`1867_lore`, lecture seule, ni pull ni écriture) | `/home/debian/workspace/1867_lore/site/labo/armes/` (plus récent : `git fetch -q` puis `git show origin/main:<chemin>`) |
| Journaux et captures | `/home/debian/orca/runs/alpha/logs/` (captures sous `p-1d-captures/`) |

## Règles de la machine (8 cœurs, 22 Go, disque partagé)
- Préfixe chaque cargo/make : `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`. **Un seul cargo à la fois**, jamais en arrière-plan sous un autre.
  **Jamais de profil debug** (`--profile headless` partout ; `--release` seulement pour le build du jeu 1837).
- Disque : 44 Go libres au départ, **plancher 30 Go** ; `df -h /` avant tout gros build ; sous 35 Go, `scripts/target-sweep.sh target` d'abord ; sous 30 Go, `ask`.
  `scripts/target-sweep.sh target` après **chaque** `make check_rapide`. Jamais `pkill` par motif : tes processus seulement, par PID.
- Écran : tout ce qui demande une fenêtre ou un GPU passe par `xvfb-run -a` (lavapipe). Banc web : chromium de Playwright (`~/.cache/ms-playwright/chromium-1248`)
  sous `xvfb-run -a`, page servie en statique (`python3 -m http.server` dans `site/`), **une** capture par état, aucune boucle d'automatisation.
- Rien de lourd (cargo, jeu) entre **04:00 et 05:00 UTC** (suite de nuit).
- Rien de 1837 dans l'engine (noms neutres, testbed) ; la donnée de 1837 manquante (ex. `poses.hit` du fantassin) se **décrit** dans le rapport et un `status`, ne se commite pas.

## Ce que tu fais, dans l'ordre (le test d'abord, la correction ensuite)
1. Compiler ce qui est écrit et non compilé (`7f7ad107` → `5d7a1aec`) : la « Prochaine commande » de la passation ; corrige jusqu'au vert.
2. **Rouge constaté** : l'invariant `animation_vivante` **échoue sur 1837 `9e7c6df`** sans la lecture du tir (méthode de la passation), sortie complète dans
   `logs/p-1d-animation-vivante-rouge-9e7c6df.log` ; `status` « contrôle : animation_vivante rouge sur 9e7c6df » avec le nom et le module stables du composant
   (`game::model3d::playback::ModelPlayback`) : T-3b le réutilisera. Puis **vert** avec la correction (`logs/p-1d-animation-vivante-vert-<sha>.log`).
3. Lint `MissingPose` : rouge sur la fixture à pose manquante et sur un état sans pose (le fantassin de 1837 `9e7c6df` sans `poses.hit`), vert sur la fixture complète.
4. Les deux contrôles **dans `make check_rapide` de l'engine** et dans **`alacod test`** (actif par défaut, pour que le coordinateur 1837 les joue avant chaque merge de 1837).
5. `make check_rapide` sur ta tête poussée, journal `logs/p-1d-parite-observee-check-rapide-<sha>.log` ; aucune trace existante modifiée (sinon : laquelle, pourquoi, BLESS
   justifié par écrit) ; `make schedule_order_diff BASE=origin/main` si l'ordre de `GgrsSchedule` peut avoir bougé.
6. **Observé dans le vrai jeu** : copie jetable 1837 `9e7c6df` avec `[patch]` vers ton worktree, build release, `xvfb-run -a` + `ALACOD_MUTE=1 ALACOD_REMOTE=1`, état des
   `AnimationPlayer` par bevy_remote (méthode de la passation), captures ; **capture côte à côte banc / jeu du cycle complet du fantassin** (épauler, cône, tir, recul,
   rechargement, retour), avant et après correction, sous `logs/p-1d-captures/`.
7. `status` « E prêt <sha> » **dès que 1–6 sont verts** (avec les chemins des journaux et des captures), sans attendre la suite. Le coordinateur fait relire et merge.
8. Ensuite, dans le même lot, si le coordinateur ne dit pas autre chose : revue observée des autres états (spec, « Ordre » point 2), texte des lignes de
   `1837/docs/parite-labo.md`, rapport `docs/taches/rapports/p-1d-parite-observee.md` complet (§ 122, D218), ligne `LIVRÉ` dans `docs/taches.md`, note de passation supprimée.

## Communication
- Commite et pousse (avance rapide, jamais `--force`) à chaque étape. Carte : `orca worktree set --worktree active --comment "<état>" --json` (ne touche pas `--unread`).
- `status` à chaque étape franchie et au moins une fois par jour ; relève la boîte (`check` de ton préambule) après chaque build ou test.
- 429 / quota du fournisseur : `status` tout de suite (le coordinateur te bascule sur `anthropic/claude-sonnet-5-5`), n'attends pas.
- Question bloquante : `ask` court avec ton choix par défaut, et avance sur le reste.
- Fin : branche poussée, rapport commité, aucun processus laissé (jeu, Xvfb, serveur http, cargo), copie jetable 1837 retirée ; un seul `worker_done`
  (`--report-path`, SHA de tête).
