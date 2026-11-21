# Revue de X-6f (changement de niveau par une porte) — consigne du relecteur, 21 nov 2026

Tu es **relecteur, lecture seule** : tu ne modifies aucun fichier, tu ne commites rien, tu ne lances **aucun cargo ni make** (disque du Mac à 33 Go, plancher 30).
Français partout.

Branche : `origin/wquintal/x-6f-changement-niveau` @ `00f96d58` (code `7ffc5454`), contre `origin/main` (`94038bb9` ou plus récent).
Tu travailles dans le worktree `~/orca/workspaces/alacod/x-6f-changement-niveau` (déjà sur cette branche ; `git fetch` permis, rien d'autre qui écrit).
Diff : `git diff origin/main...origin/wquintal/x-6f-changement-niveau`.

À lire : fiche du lot `docs/taches/plan-1837-cli.md` § 6 ligne **X-6f** (contenu et preuve attendue), rapport `docs/taches/rapports/x-6f-changement-niveau.md`,
`docs/conventions.md` § 128 (nouveau), § 17 et § 117, `CLAUDE.md` (checklist GGRS). Journal du contrôle rapide : `~/orca/runs/alacod/logs/x-6f-check-rapide-7ffc5454.log`.

## Ce que tu vérifies (dans cet ordre)
1. **Déterminisme et rollback** (P1 si violé) : itérations ordonnées par `GgrsNetId` dans tout système de `GgrsSchedule` touché (`floor_transition_system`, `crates/map_ldtk/src/game/floors.rs`,
   `crates/run/src/floors.rs`) ; aucun `f32`, `HashMap`, `.iter().next()` non trié ; sélection du « joueur qui franchit » déterministe à égalité ; `despawn_rollback` ;
   aucun état caché lu par la simulation hors rollback ; champs neufs sur des types sous checksum (`DoorComponent`, `FloorState`, séquence, porte générée…) : hachés
   **seulement s'ils sont actifs** (sinon toutes les traces bougent) ou hors checksum avec justification ; aucun type rollback ni système nouveau sans `schedule_order` à jour.
   Le worker affirme « aucun système, type rollback ni enregistrement ajouté » et n'a pas joué `make schedule_order_diff` : **vérifie cette affirmation dans le diff**
   (enregistrements `rollback_and_trace_*`, `add_systems`, ordre des sets) ; le test `scenario/schedule_order` est vert dans le journal.
2. **Défaut inchangé** : sans `exit_by_door` / `exit_next_level`, le comportement de `Floors`, des étages `floor:` et des portes à clé de X-6a1 est identique au bit près
   (portail, consommation du RNG de génération dans `imp/basic.rs`, `grid.rs`, `floor.rs`). Les 79 scénarios rejoués sans BLESS le soutiennent ; cherche ce qu'ils ne couvrent pas.
3. **Conformité à la fiche** : toute l'équipe passe, sans attendre le niveau vide ; porte refusée sans clé ; clé consommée ou non selon la donnée ; coéquipier à terre / feu follet
   arrive dans son état ; relance = niveau 0 même graine ; lint (sortie sur le dernier niveau, niveau sans sortie) ; invite « Entrer » en présentation seulement
   (`crates/game/src/ui/hud.rs` ne doit rien écrire dans la simulation). Les 7 scénarios `sortie_porte_*` testent-ils vraiment ce que leur nom dit (attentes, frames) ?
4. **Frontière simulation/présentation** et **rien de 1837 dans l'engine** (noms neutres, contenu du testbed).
5. **Preuve** : capture `docs/taches/rapports/preuves/x-6f/` (vidéo 21 s, trois images : regarde les images) ; le « tableau avant » est argumenté par les traces commitées
   rejouées à l'identique : dis si c'est suffisant. `games/testbed/assets/game.ron` modifié de 2 lignes : vérifie que cela ne change aucun scénario existant
   (ordre des contenus, séquence `Floors` par défaut — piège du § 40 « la séquence par défaut »).

## Rendu
Un fichier `~/orca/runs/alacod/revue-x-6f-verdict.md` (hors dépôt, c'est la seule écriture permise) : verdict en une ligne (**merge** / **merge après corrections** / **ne pas merger**),
puis les défauts classés P1 (bloquant) / P2 (à corriger avant merge si simple) / P3 (suivi), chacun avec `fichier:ligne`, le problème et la correction proposée.
Puis `worker_done` avec ce fichier en rapport. Pas de question à William.
