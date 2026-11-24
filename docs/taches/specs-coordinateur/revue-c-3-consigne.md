# Revue de C-3 (un décor solide bloque les corps) — consigne du relecteur, 23 nov 2026

Tu es **relecteur, lecture seule** : tu ne modifies aucun fichier, tu ne commites rien, tu ne lances **aucun cargo ni make** (William joue sur ce Mac ; disque surveillé). Français partout.

Branche : `origin/wquintal/c-3-collision-props` @ `f07dfcd7` (contrôle rapide découpé vert @ `b9d9d957`, journal `~/orca/runs/alacod/logs/c-3-check-rapide-b9d9d957.log`), contre `origin/main`.
Worktree : `~/orca/workspaces/alacod/c-3-collision-props` (`git fetch` permis, rien d'autre qui écrit). Diff : `git diff origin/main...origin/wquintal/c-3-collision-props`.

À lire : brief `docs/taches/specs-coordinateur/c-3-collision-props.md` ; fiche `docs/taches/plan-1837-cli.md` § 6 ligne **C-3** ; rapport `docs/taches/rapports/c-3-collision-props.md` ; `docs/conventions.md` § 132 (nouveau), § 112 (M-1b), § 118 (X-6a2) ; `CLAUDE.md` (règles GGRS, Hash neutre d'un champ ajouté, parité des types vides).

## Ce que tu vérifies (dans cet ordre)
1. **Opt-in au bit près** (P1 si violé) : un prop sans `solid` se comporte exactement comme avant (joueur ralenti, ennemi séparé, champ de flux inchangé) ; aucune trace existante modifiée dans le diff (seules les six `decor_solide_*.trace` sont neuves) ; `SolidBody` : est-il un type rollback enregistré ? si oui, variante **neutre** (un type nouveau sous checksum sans porteur déplace toutes les traces) ; sinon, est-il bien statique et reconstruit identiquement après un rollback (entité respawnée par `despawn_rollback`) ?
2. **Déterminisme** : `Fixed` seulement, ordre `GgrsNetId`, `BTreeSet`/`BTreeMap`, aucun `f32`, aucun `Entity` dans les logs, glissement du joueur calculé comme contre un `Wall`.
3. **Ennemis et champ de flux** : cases bloquées posées à la naissance et **libérées à la mort** d'un prop `Destructible` (scénario `decor_solide_destructible`) ; déclencheur de reconstruction existant réutilisé ; un ennemi ne reste pas coincé contre le bord du prop.
4. **Trous connus** : D240 (le **dash** et `recovery_direction` des vagues ignorent `SolidBody`). Juge : un joueur peut-il traverser une caisse solide d'un dash (1837 : `dash_distance 48`, caisse 14 u) ? Si oui, c'est le défaut même que corrige le lot : classe-le **P2 à corriger avant merge**, avec le chemin du code et la correction minimale ; idem pour toute autre voie de déplacement (recul, poussée, téléportation) qui contourne le blocage.
5. **Preuves** : tableau rouge/vert du rapport cohérent avec `~/orca/runs/alacod/logs/c-3-rouge.log` et `c-3-familles.log` ; réserve du worker : rouge de `decor_solide_ennemi` non rejoué avec les attentes finales — ses attentes échoueraient-elles sans le blocage (lecture des positions attendues) ? Capture `~/orca/runs/alacod/captures/c-3-decor-solide.mp4` (30 s, `play_scenario`, caisse invisible faute de sprite au testbed) : extrais deux ou trois images par ffmpeg, dis si l'arrêt du joueur et le contournement de l'ennemi se voient. Lint (`solid` sans `prop`), `describe Character`, § 132, D240, ligne `CLAUDE.md` présents et exacts. Rien de 1837 dans l'engine.

## Rendu
`~/orca/runs/alacod/revue-c-3-verdict.md` (hors dépôt, seule écriture permise) : verdict en une ligne (**merge** / **merge après corrections** / **ne pas merger**), puis P1 / P2 / P3 avec `fichier:ligne`, problème, correction proposée. Puis `worker_done` avec ce fichier en rapport. Pas de question à William.
