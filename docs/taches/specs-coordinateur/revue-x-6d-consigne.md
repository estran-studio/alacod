# Revue avant merge (agent relecteur, LECTURE SEULE) — X-6d agrippeur générique `Grab`

Tu es relecteur : tu n'édites aucun fichier, ne commites rien, ne lances AUCUN cargo/make (disque au plancher). Travaille par `git -C /home/debian/orca/alacod fetch -q`
puis `git -C /home/debian/orca/alacod diff origin/main...origin/wquintal/x-6d-agrippeur` et `git show origin/wquintal/x-6d-agrippeur:<chemin>` ; ne fais aucun checkout.
Lis d'abord `CLAUDE.md` (via `git show origin/main:CLAUDE.md`) pour la checklist GGRS : elle fait foi.

Contexte (plan `docs/taches/plan-1837-cli.md` § 6, ligne X-6d) : behavior générique `Grab` (noms neutres, valeurs en données) : court vers le joueur, bondit, s'accroche (suit
l'ancre du porteur, nombre borné), gêne son déplacement, presque sans dégâts ; décrochage en secouant (input du porteur) ou d'un coup de mêlée ; décroché, étourdi ; durée maximale.
Journaux : `/home/debian/orca/runs/alpha/logs/x-6d-agrippeur-regressions.log`, `x-6d-*`. Rapport `docs/taches/rapports/x-6d-agrippeur.md`.

À vérifier :
1. Déterminisme GGRS ligne à ligne : `order_iter!` par `GgrsNetId`, tie-breaks, aucun `f32`/`HashMap`/`to_bits`, `RollbackRng` si aléa, types rollback neufs en variante neutre,
   champ ajouté à un type sous checksum avec `impl Hash` manuel (champs actifs seulement), discriminant d'enum à une variante, `despawn_rollback`, donnée dérivée réécrite à chaque
   frame, logs `ggrs{f=…}` sans `Entity`, enregistrement dans `CoreSimPlugin` avant les cinq de clôture.
2. Secouage lu depuis l'input rollback (`PlayerInputs`) dans `GgrsSchedule`, jamais le clavier/la manette directement ; seuil en donnée ; même chemin clavier/stick.
3. Interactions : cumul de plusieurs agrippeurs sur la vitesse, expiration des modificateurs, dash du porteur, mort/feu follet du porteur pendant l'accroche, changement d'étage,
   agrippeur tué accroché, collisions et invariant `joueur_hors_mur`.
4. Présentation : poses `saut`/`accroche`/`etourdi`, ancre des épaules (présentation seule), couverture par `MissingPose` et `animation_vivante`.
5. Tests : chaque attente nouvelle échoue vraiment (journal) ; entrée réelle via le harnais T-3b ; aucune trace existante modifiée ; rapport sans revendication non prouvée ; rien de 1837.

Sortie : écris ton verdict dans `/home/debian/orca/runs/alpha/logs/revue-x-6d.md` (merge / après corrections / non ; défauts classés bloquant / à corriger / mineur avec
fichier:ligne et raison factuelle ; ce que tu n'as pas pu vérifier sans compiler), puis un seul `worker_done` avec `--report-path` vers ce fichier.
