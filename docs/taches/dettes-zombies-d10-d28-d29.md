# dettes-zombies-d10-d28-d29 — difficulté et cartes de `zombies` (décisions de William du 2026-10-14)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (déterminisme, génération de cartes), `docs/conventions.md`
§1 (générateur), §10 (preuve d'un bless), §13 (vagues), et les dettes **D10, D28, D29** de
`dettes.md`. Branche `dettes-zombies-d10-d28-d29` depuis `main`. Estimation : 1,5 j. **Trois commits
séparés, un par dette, chacun avec sa preuve §5** : ils changent des traces différentes et doivent
se relire séparément. Prévenir l'orchestrateur avant de démarrer : William rééquilibre `zombies` sur
sa branche `m0-revue-suite-contenu` (à fusionner ou à éviter, voir avec lui via l'orchestrateur).

## À faire

1. **D29, multiplicateurs par vague appliqués** : `WaveState::current_health_multiplier` multiplie
   la vie des ennemis **au spawn** de la vague, `current_damage_multiplier` leurs dégâts (arme de
   mêlée et attaque directe) ; valeurs de `wave_config.ron` inchangées (0,05 et 0,03 par vague).
   Toutes les traces de vagues bougent : preuve §5 sur `idle`, `points_on_kill`, `clone_solo` (seuls
   PV et dégâts des ennemis diffèrent, puis ce qui en découle), bless avec la cause.
2. **D10, gabarit de départ exclu** des salles suivantes (`crates/map/src/generation/imp/basic.rs`,
   `get_next_room_recursize`) : preuve sur `idle` et `points_on_kill` (seules les entités de la salle
   disparue et les ids décalés), toutes les traces `test_map` rebénies, décision écrite en §1.
3. **D28, dégât direct des attaques ennemies réparé** (`enemy_attack_damage_translate_system` émet le
   dégât sans exiger un `DamageAccumulator` préexistant) : les ennemis font plus mal ; mesurer
   l'effet sur 20 graines `alacod-sim` zombies (4 bots) avant/après et proposer un rééquilibrage
   (`attack_damage` en donnée) qui garde le taux de victoire ; preuve §5.
4. Après les trois : campagne 200 graines `make sim` zombies (Relais, 4 acheteurs, `--until-wave 5`)
   pour vérifier que la difficulté reste jouable ; chiffres au rapport.

## Règles

Les jeux autres que `zombies` ne bougent pas. Pas de `f32`. `cargo fmt --all -- --check`.
Vérification README §4 (`make test_scenarios_par` ou `make test_scenarios`), rapport
`docs/taches/rapports/dettes-zombies-d10-d28-d29.md`, `LIVRÉ dettes-zombies-d10-d28-d29 <sha>`.
