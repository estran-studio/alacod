# Revue de X-6h (caméra fixe par pièce et portes de combat) — consigne du relecteur, 22 nov 2026

Tu es **relecteur, lecture seule** : tu ne modifies aucun fichier, tu ne commites rien, tu ne lances **aucun cargo ni make** (William joue sur ce Mac ; disque surveillé). Français partout.

Branche : `origin/wquintal/x-6h-camera-portes-combat` @ `fe20c153` (fusion de `main` `dc734091` incluse), contre `origin/main`. Worktree : `~/orca/workspaces/alacod/x-6h-camera-portes-combat`
(`git fetch` permis, rien d'autre qui écrit). Diff : `git diff origin/main...origin/wquintal/x-6h-camera-portes-combat`.

À lire : fiche `docs/taches/plan-1837-cli.md` § 6 ligne **X-6h** et § 3 (décisions du 21 nov) ; rapport `docs/taches/rapports/x-6h-camera-portes-combat.md` ; `docs/conventions.md` § 130 (nouveau),
§ 35, § 40 (caméra `room_frame`), § 117, § 113 ; `CLAUDE.md`. Journal du contrôle rapide : `~/orca/runs/alacod/logs/x-6h-check-rapide-fe20c153.log` (coupé une fois par la garde disque, rejoué depuis `placement`).

## Ce que tu vérifies (dans cet ordre)
1. **Défaut inchangé** (P1 si violé) : sans `room_fit` / `room_slide_seconds` / `no_rally`, la caméra d'isaac (`room_frame`) et les salles verrouillées (§ 35) se comportent comme avant ; « porte à serrure ouverte
   refermée en combat » : ce changement de `crates/map_ldtk/src/game/rooms.rs` touche-t-il des jeux existants (isaac `key_door*`, X-6a1 `map_door*`) ? Les 118 traces inchangées le soutiennent : cherche le cas non couvert.
2. **Déterminisme et rollback** (`rooms.rs`, `crates/world/src/rooms.rs`) : `order_iter!`, aucun `f32`/`HashMap`, champ neuf d'un type sous checksum haché seulement s'il est actif, aucun état caché ;
   `schedule_order` vert dans le journal.
3. **Frontière simulation/présentation** : `crates/game/src/camera/mod.rs` (`room_fit`, glissement) n'est lu par aucun système de `GgrsSchedule` ; le glissement dépend du temps de présentation, pas de la simulation.
4. **Conformité à la fiche** : portes fermées à l'entrée d'une salle `locks: true`, rouvertes à la fin ; porte à clé fermée sans clé ; salle vidée stable ; règle coop choisie (regroupement d'Isaac par défaut,
   `no_rally` pour la couper) justifiée ; à terre / feu follet ; caméra en local et en ligne (`online_follow`). Les 7 scénarios `combat_*` testent-ils ce que leur nom dit ? Tests purs de la caméra (plus petite,
   égale, plus grande que la vue ; glissement borné). Lint des nouveaux champs.
5. **Rien de 1837 dans l'engine** ; preuve : images de `docs/taches/rapports/preuves/x-6h/` (regarde-les), écart « pas de couloir séparé (salles de 128 px) » acceptable ou non.

## Rendu
`~/orca/runs/alacod/revue-x-6h-verdict.md` (hors dépôt, seule écriture permise) : verdict en une ligne (**merge** / **merge après corrections** / **ne pas merger**), puis P1 / P2 / P3 avec `fichier:ligne`,
problème, correction proposée. Puis `worker_done` avec ce fichier en rapport. Pas de question à William.
