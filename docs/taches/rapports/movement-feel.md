SHA de tête vérifiée : `26f1496` (`main`) plus le diff non commité de la branche ; aucun commit
sans le feu vert de William.
Fiche : aucune. Travail interactif demandé par William, session Claude Code sur son Mac
(aarch64-apple-darwin), **hors protocole Orca** : worktree `alacod-movement`, branche
`movement-feel`.
Base : `1a39283` ; le diff a été reporté sur `5cad4b3` (statuts T1.3 : conflits dans
`crates/game/src/character/mod.rs` et `crates/game/src/character/player/input.rs`, résolus à la
main), puis sur `26f1496` (D40, sans conflit).
Date : 2026-10-04 ; agent : Claude Code (Opus 5.5).

La demande : « on dirait qu'on patine sur de la glace » ; un contrôle précis, nerveux, fluide et
rapide, pour la course comme pour le dash, dans l'esprit de Gungeon, Nuclear Throne et Isaac.
Choix de William : dash nerveux avec i-frames, profil `dev` optimisé (D30), délai d'input GGRS
gardé à 5.

## Diagnostic (avant)

- **Course** : `acceleration` 150 px/s² pour `max_speed` 150 px/s, soit 1 s pour atteindre la
  pleine vitesse et 2 s pour un demi-tour. `friction` 10 ne freinait que si aucune touche de
  déplacement n'était tenue : vitesse × 5/6 par frame, ~28 frames et 12,5 px de glissade à
  l'arrêt ; un axe relâché glissait tant que l'autre était tenu.
- **Dash** : réglé à 60 px en 15 frames, il en couvrait 56 (frame de l'appui immobile, puis
  14 frames à 4 px) et s'arrêtait net ; il repartait tant que le bouton était tenu ; cooldown de
  180 frames (3 s, le commentaire disait « Half-second ») ; tenir Dash bloquait le tir ; aucune
  invulnérabilité (`invulnerable_until_frame` n'était jamais posé) ; sans touche ni visée,
  direction horizontale seulement.
- **Caméra** : `lerp_factor = lerp_speed × dt`, `lerp_speed` 5 : 63 % de l'écart rattrapé en
  ~190 ms, et un facteur supérieur à 1 sous 5 fps.
- **Profil `dev`** : ~15 fps rendus (D30).

## Fait

1. **Course** (`crates/game/src/character/movement.rs` : `approach_axis`, `run_velocity`,
   `grip`) : chaque axe va vers la vitesse visée ; il accélère de `acceleration` et freine de
   `deceleration` (nouveau champ optionnel, absent = `acceleration`), sans dépasser la cible.
   3000/3000 px/s² : pleine vitesse (2,5 px par frame) en 3 frames, arrêt en 3, demi-tour en 6.
   `friction` supprimé. `grip` (stat `Acceleration` résolue / base) multiplie aussi le freinage :
   la glace (×0,2) met 15 frames à lancer ou arrêter le joueur. 4 tests unitaires.
2. **Dash** (`crates/combat/src/actors.rs` : `DashState`, `dash_step`) : part sur l'appui (front
   montant) et avance dès cette frame ; 64 px exacts en 8 frames, par un pas qui décroît de 13,5
   à 2,5 px (on sort du dash en courant) ; cooldown de 24 frames compté depuis le départ ; un
   appui refusé est gardé 8 frames. Direction : touches, sinon visée, sinon regard (8
   directions), inversée par `Modifier`. Le dash s'arrête aux murs et n'est pas ralenti par les
   ennemis. Le tir est bloqué pendant le dash, plus par le bouton tenu. 6 tests unitaires.
3. **I-frames** (les 6 premières frames du dash) : `Health.invulnerable_until_frame` est posé,
   `Health::is_invulnerable_at` est le seul test. Balles et projectiles composables traversent
   le joueur (`Pierce` non consommé), les hitbox de mêlée l'ignorent, les autres dégâts sont
   annulés sauf `DamageKind::True` ; `Homing` et `Aimed` le visent toujours
   (`crates/combat/src/weapons/mod.rs`, `projectile.rs`, `weapons/melee.rs`,
   `crates/game/src/character/health/mod.rs`).
4. **Garde des statuts**, bug trouvé en reportant le diff sur `5cad4b3` (T1.3) : un appui gardé
   partait pendant un étourdissement et donnait des i-frames au joueur immobilisé. Garde
   `!incapacitated` dans `apply_inputs`. Preuve : `dash_stun_buffer`, 89 PV avec la garde, 99
   sans.
5. **Caméra** (présentation seulement) : lissage exponentiel `1 − e^(−lerp_speed·dt)`,
   indépendant du framerate ; `lerp_speed` 12, soit 63 % de l'écart en 83 ms (défaut du code et
   les trois `camera.ron`).
6. **D30** (`Cargo.toml`) : `[profile.dev] opt-level = 1`, `[profile.dev.package."*"]
   opt-level = 3` ; `docs/jouer-a-deux.md` mis à jour.
7. **Contenu** : les joueurs (`player_config.ron` de zombies et du testbed,
   `characters/pilote.ron` de throne) passent à 3000/3000, `max_speed` 150, dash 64/8/24,
   i-frames 6, buffer 8, sprint inchangé. `friction` retiré partout (16 personnages du testbed,
   13 de throne, 4 configs de zombies) : les ennemis ne lisent que `max_speed`.
8. **Générateur** (`crates/scenario/src/generate.rs`) : marche des scénarios de mêlée générés
   recalée (`WALK_X_FRAMES` 51, `WALK_PAUSE_FRAMES` 5, `WALK_Y_FRAMES` 19) ; les 16 `.ron` de
   mêlée générés ont été réécrits par `make gen`.
9. **Scénarios** :
   - 24 scénarios écrits à la main recalés (positions et frames), méthode dans
     `docs/conventions.md` §30 ;
   - la visée de `clone_solo`, `clone_duo` et `clone_quad` a été ré-enregistrée à partir de
     f1000 ;
   - `throne_mutation_choice` et `bot_prudent_nododge` ont été réenregistrés par
     `alacod-sim --save-scenario` ;
   - 2 nouveaux : `dash_iframes`, `dash_stun_buffer` ;
   - le test `caverne_dans_une_sequence_floors` (`crates/scenario/tests/cave.rs`) reprend les
     nouveaux inputs de `portal_next_floor`.
10. **BLESS, changement de gameplay voulu** : les 182 traces existantes rebénies, plus les 2
    neuves. Toutes changent dès la frame 0, pour deux raisons : `DashState`, composant rollback
    au checksum, a de nouveaux champs ; la stat de base `Acceleration` des joueurs passe de 150 à
    3000.
11. **Docs** :
    - `docs/conventions.md` : §30, nouveau (course, adhérence, dash, i-frames, scénarios,
      recalage) ; §26 ;
    - `docs/plan-engine.md` : ligne « Esquive » ;
    - `docs/taches/dettes.md` : D30 marquée faite, D41 ajoutée.

## Vérifié

- **Suite complète des scénarios** : 184 verts (745,9 s). Rejouée dans les tests des crates :
  verte, 384 s.
- **Preuve du BLESS** sur les scénarios sans input de déplacement (`idle`, `four_players_idle`,
  `four_players_shooting`) :
  - méthode : dumps `ALACOD_DUMP_TRACE` de `main` (`26f1496`) et de la branche, `Acceleration`
    de base ramenée à 150 par `sed`, puis `scripts/trace-diff.py --ignore DashState` ;
  - résultat : **identiques** sur toutes les frames (1499, 599, 299) ; zombies, tirs, dégâts et
    vagues y sont ceux de `main`. Les autres traces changent en plus par les trajectoires des
    joueurs qui se déplacent, et ce qui en dépend.
- **Tests des crates** (`scenario run combat game content map_ldtk sim_core stats bots
  effects`) : verts, dont 10 tests unitaires neufs (4 de course, 6 de dash).
- **Fuzz** (`ALACOD_FUZZ=0:8`, `fuzz_inputs`) : 8 graines de 3600 frames, aucune divergence.
- **Outils** :
  - `make lint` : zombies, testbed et throne sans erreur ;
  - `cargo fmt --check` : propre ;
  - `check-forbidden.sh` : 4 occurrences, toutes préexistantes ;
  - `check-rollback-registration.sh` : OK.
- **`make gen`** : testbed 36/36, zombies 16/16, throne 37/37 `ok`, aucun fichier généré modifié.
- **`alacod-sim`**, branche (`26f1496` + diff) contre `main` (`26f1496`) :
  - throne, `--bots 2 --profiles prudent,prudent --floors run --seeds 1..20 --until-floor 3
    --max-frames 12000` :
    - branche : **20/20** runs finies, moyenne 2323 frames, 0 mort ;
    - `main` : 18/20, 0 mort. Graines 16 et 17 en softlock au niveau 2 : plus aucun ennemi, aucun
      progrès pendant 600 frames. Moyenne 2702 frames sur ses 18 runs finies, contre 2355 pour
      la branche sur les mêmes graines.
  - zombies, `--bots 4 --profiles fonceur,fonceur,prudent,immobile --seeds 1..10 --until-wave 3
    --max-frames 20000` :
    - graines 1 à 9, finies des deux côtés : branche 2911 frames en moyenne, 0 mort ; `main`
      3462 frames, 2 morts (graine 9) ;
    - graine 10 : bloquée des deux côtés à la vague 2. C'est préexistant : il reste un zombie et
      il n'y a plus de munitions. Les deux `fonceur` de la branche y meurent entre f4000 et
      f4400 ; aucun mort sur `main`.
- **`make bench`** (`ALACOD_BENCH_STRICT=1`, rien d'autre ne tournait) : vert, aucun scénario
  sous son plancher de `tests/budgets.ron` (suite en 377 s). Pas de colonne Δ : aucune mesure de
  `main` dans ce worktree.

## Non fait / non vérifié

- **Jeu à la main** : aucune partie jouée par un humain ; le « feeling » reste à valider par
  William (`make zombies`).
- **P2P** (`make test_multiplayer`) : non lancé, il ouvre des fenêtres. Le déterminisme est
  couvert par le synctest des scénarios et le fuzz.
- **fps rendus du profil `dev`** (D30) : non mesurés. La première compilation sera plus longue.
- **wasm** : non compilé, la cible `wasm32-unknown-unknown` n'est pas installée sur le Mac.
- **Charges de dash, variantes par personnage** (B6) : non faites. Les réglages sont dans le RON
  de chaque personnage.
- **Ligne de journal** : laissée à l'orchestrateur, au merge.

## Dettes laissées, décisions ouvertes

- **I-frames façon Gungeon** : décision de William, à confirmer en jeu. Avec `dash_iframes: 0`,
  l'esquive n'a plus d'invulnérabilité, comme la roulade de Nuclear Throne.
- **D27** : la dispersion des armes autres que le fusil à pompe est un tirage ±0,5 rad (±29°)
  qui ignore `spread`. Elle nuit plus au « contrôle précis » que la course ; la corriger change
  toutes les traces de tir.
- **D41**, nouvelle : en tir ami `Always`, une balle touche son propre tireur ;
  `dash_stun_buffer` s'en sert.
- **Sprint** : rampe inchangée, 10 frames pour atteindre la pleine vitesse de sprint. Elle est
  plus molle que la course ; à régler si William la trouve lente.
- **Bots** : le freinage avant portail de `crates/bots/src/decide.rs` reste en garde ; seuls ses
  commentaires changent.
- **Graine zombies 10** : blocage préexistant (dernier zombie, plus de munitions). Avec la
  nouvelle course, les `fonceur` y meurent.
