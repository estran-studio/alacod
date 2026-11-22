# X-6h — corrections de revue (consigne du coordinateur, 22 nov 2026)

Tu reprends le lot **X-6h** sur sa branche `wquintal/x-6h-camera-portes-combat` @ `fe20c153`, dans le worktree `~/orca/workspaces/alacod/x-6h-camera-portes-combat`.
Brief d'origine (machine, communication) : `docs/taches/specs-coordinateur/x-6h-camera-portes-combat.md` ; il s'applique, **avec la règle machine du 21 nov** : tout cargo/make en
`nice -n 19 env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 …`, un seul cargo lourd sur toute la machine. Français partout.
Verdict : **merge après corrections**, `docs/taches/specs-coordinateur/revue-x-6h-verdict.md` sur `origin/main` (copie `~/orca/runs/alacod/revue-x-6h-verdict.md`). Lis-le.

## À corriger
1. **P2.1** (`crates/map_ldtk/src/game/rooms.rs:833`, `still_keyed`) : une porte à serrure **non interactive** fermée ne doit jamais être rouverte à la fin d'un combat (défaut d'avant X-6h). Corrige par un fait
   d'état, pas par l'absence d'`Interactable` (la version sans état proposée par le relecteur, `door.config.interactable`, convient si elle est juste) ; **test d'abord** (unitaire dans `rooms.rs` avec `room_app`,
   ou scénario neutre) rouge sur `fe20c153`, vert après.
2. **P2.2** : `combat_coop_a_terre` et `combat_coop_feu_follet` prouvent le rappel : `PlayerPosition` du joueur 1 hors salle avant le verrouillage et près de l'entrée après (valeurs lues dans la trace ; trace inchangée).
3. **P2.3** : le cas utile de `room_fit` est une salle **plus grande que la vue** avec le `min_zoom` livré. Ne refais pas de build release : ajoute un test pur avec les valeurs livrées du testbed et une salle plus grande que la
   vue ; dis dans le rapport que la preuve visuelle d'une grande salle se fera dans 1837 (ses salles dépassent la vue), au réépinglage.
4. **P3.1** : le glissement ne s'applique qu'au changement de salle (ou si la salle tient) ; sinon `lerp_speed` (test pur).
5. **P3.2** : lint (avertissements) : `no_rally` sans `locks`/`arena`, `room_fit`/`room_slide_seconds` sans `room_frame`, `room_slide_seconds` négatif ou non fini ; tests dans `crates/content/tests`.
6. **P3.3** : `MapDoor(nth: 1, state: Open, at_frame: 200)` dans `combat_porte_cle_referme.ron` (trace inchangée).
7. **P3.4** : `tags: ["rapide"]` sur deux ou trois `combat_*` courts (budget du jeu court, § 123).
8. Fusionne `origin/main` si elle a bougé (`git merge`, pas de rebase).

## Preuve et feu vert
Tests nouveaux rouges avant / verts après (dans le rapport, section « Corrections de revue ») ; aucune trace modifiée (les `PlayerPosition`/`MapDoor` ajoutées ne changent pas la trace) ; scénarios `combat_*` rejoués.
**Avant le contrôle rapide découpé, `status` au coordinateur et attends son feu vert** (le coordinateur 1837 utilise la machine). Disque : `df` avant ; sous 30 Go, arrête par PID et dis-le.
Journal en `~/orca/runs/alacod/logs/x-6h-check-rapide-<sha>.log`. Fin : `status` « X-6h prêt <sha> » puis **un seul** `worker_done`.
