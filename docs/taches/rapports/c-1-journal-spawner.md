# C-1 — Journal du spawner trop bavard

Verdict de William (21 nov 2026, 1837 `476e9a2`) : « extrêmement de logs ». Mesure 1837 : 2 859 lignes `INFO game::character::enemy::spawning` en 55 s.

## Cause
`enemy_spawn_from_spawners_system` (`crates/game/src/character/enemy/spawning.rs`) tourne à chaque frame quand les vagues sont coupées (testbed, jeux à spawners) et journalisait en `info!`
ses trois sorties anticipées `spawner_skip` (`no_players_in_room`, `max_enemies`, `no_valid_spawner`). Les frames resimulées (synctest/rollback) les répètent : ~3 lignes par frame jouée.

## Correction
Les trois `info!` deviennent `trace!`, format `ggrs{f=… spawner_skip …}` inchangé (`CLAUDE.md` § 10). Aucun changement de logique : ni ordre, ni RNG, ni état.

## Preuve
- Test `crates/scenario/tests/journal_par_frame.rs` : sous un abonné `tracing` au niveau `info`, quatre parties de 600 frames (testbed `arena`, `corridor`, `salles`, zombies `test_map`) ne doivent produire aucune ligne INFO/WARN
  répétée (même texte aux `f=` près) à au moins une frame sur deux. Le runner force `WaveModeEnabled(true)` ; le test le remet à `false` comme une partie réelle de jeu sans vagues, sinon le spawner ne tourne pas.
  **Rouge avant** (`testbed/arena` : 1 794 lignes `spawner_skip` en 600 frames), **vert après**.
- Chiffre, 3 600 frames (60 s) de `testbed/arena`, headless : **10 821 lignes INFO avant, 27 après** (`ALACOD_C1_FRAMES=3600`). Autres cartes après : corridor 18, salles 32, zombies `test_map` 84.
- Scénarios `idle`, `two_players_idle`, `testbed_arena_idle`, `testbed_corridor_idle`, `testbed_dummy_shoot`, `testbed_ally_safe` : verts, aucune trace modifiée.
- `make diff_log` : relancer les clients avec `RUST_LOG=info,game::character::enemy::spawning=trace` pour garder les lignes `spawner_skip`.

## Reste
Mesure native release de 1837 non faite (feu vert non donné). Les autres `info!` du jeu n'ont pas été trouvés répétés à chaque frame par le test sur ces cartes ; le reste de la partie réelle (présentation, scénarios à spawners de 1837) n'est pas couvert : dette D230 si 1837 en voit encore.
