# Revue de C-2 (poses des phases de comportement) — consigne du relecteur, 22 nov 2026

Tu es **relecteur, lecture seule** : tu ne modifies aucun fichier, tu ne commites rien, tu ne lances **aucun cargo ni make** (William joue sur ce Mac ; disque surveillé). Français partout.

Branche : `origin/wquintal/c-2-poses-comportement` (dernière tête poussée ; le contrôle rapide découpé du worker est en cours ou à venir, tu n'en dépends pas), contre `origin/main` (`98b01d90` ou plus récent).
Worktree : `~/orca/workspaces/alacod/c-2-poses-comportement` (`git fetch` permis, rien d'autre qui écrit ; le worker y travaille encore : lis par `git show origin/wquintal/c-2-poses-comportement:<chemin>` ou `git diff`).
Diff : `git diff origin/main...origin/wquintal/c-2-poses-comportement`.

À lire : brief `docs/taches/specs-coordinateur/c-2-poses-comportement.md` ; fiche `docs/taches/plan-1837-cli.md` § 6 ligne **C-2** ; rapport `docs/taches/rapports/c-2-poses-comportement.md` ;
`docs/conventions.md` § 131 (nouveau), § 119 (X-6b), § 120 (X-6c), § 122-123 (T-3, T-3b) ; `CLAUDE.md`.

## Ce que tu vérifies (dans cet ordre)
1. **Défaut inchangé** (P1 si violé) : un `Model` sans `charge_windup` / `charge` / `return_home` / `take_cover` choisit exactement les mêmes poses qu'avant dans `decide_poses`
   (`crates/game/src/model3d/playback.rs`, `pose.rs`) : cherche le chemin où la lecture de `BehaviorRuntime` change le choix même champ absent. Les 18 scénarios témoins (`rodeur_*`, `zigzag_*`,
   `repli_couvert_*`, `tireur_pantin_*`, `model_pantin`) sont dits inchangés : vérifie qu'aucune trace existante n'est modifiée dans le diff (`tests/scenarios/*.trace`).
2. **Correctif de harnais `ClipSpeed`** (le worker l'a changé : un clip joué à l'envers était lu comme un tour de boucle, 46 au lieu de −2) : est-il juste, et peut-il faire passer ou échouer à tort une attente
   existante (`tireur_pantin_*`, scénarios T-3b) ? Montre les attentes `ClipSpeed` existantes et dis si leur verdict change.
3. **Frontière simulation/présentation** : la lecture de l'état de comportement est en lecture seule, hors `GgrsSchedule` ; aucun champ ajouté à un type rollback ; aucune écriture d'état de simulation
   depuis la présentation ; `content` (crate SIM ?) n'importe rien de présentation.
4. **Conformité au brief** : les phases (annonce de `Charge`/`Telegraph`, charge/zigzag, `ReturnHome`, `TakeCover`) sont-elles lues au bon endroit, avec la bonne priorité face aux poses existantes
   (attaque, coup reçu, mort) ? `animation_vivante` couvre ces phases quand le champ est posé ; lint `reach.rs` = **avertissement** seulement, jamais erreur ; pose inconnue = erreur comme avant ;
   `describe Model` montre les champs. Les quatre scénarios `pose_*` (un et deux joueurs) exigent-ils bien le clip de **chaque** phase (pas seulement un clip non idle) ? Le rouge rapporté (attentes
   en échec, modèles sans les champs) est-il crédible au vu des attentes ?
5. **Rien de 1837 dans l'engine** (noms neutres, testbed seul) ; preuve : vidéos `docs/taches/rapports/preuves/c-2/` ou `~/orca/runs/alacod/videos/c-2-pose_*.mp4` (ouvre-en au moins une image par ffmpeg
   si possible, sans cargo) ; conventions § 131 et dettes D231–D232 présentes et exactes.

## Rendu
`~/orca/runs/alacod/revue-c-2-verdict.md` (hors dépôt, seule écriture permise) : verdict en une ligne (**merge** / **merge après corrections** / **ne pas merger**), puis P1 / P2 / P3 avec `fichier:ligne`,
problème, correction proposée. Puis `worker_done` avec ce fichier en rapport. Pas de question à William.
