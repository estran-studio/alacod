# Rapport m3-t17-feedback-v3 (feedback v3 : larmes lisibles, familiers, champions, bombes et explosions)

## 1. Fait (présentation seule, conventions §82)
- **Larmes** : `feedback_v3::tear_look` (couleur du statut posé, sinon turquoise si guidée ; translucide si spectrale ; taille ×1,25 `Split`, ×1,15 `Pierce`, ×0,8 par génération), appliqué par `world_sprites::style_tears` après `skin_bullets`.
- **Familiers/orbitaux** : sprites provisoires (`compagnon`, `orbital`, `rempart`), orientation (`orient_familiars` : suiveur retourné vers où va le porteur, orbital pointe vers l'extérieur).
- **Champions** : `ChampionTint` d'après `VariantDef::tint`, couleur de base des calques ; teintes ajoutées aux variantes du testbed (sauf `nu`).
- **Bombe** : mèche qui clignote de plus en plus vite et gonfle ; explosion = deux anneaux + secousse proportionnelle au rayon (`explosion:` dans `ui/feedback.ron`, lint + 2 fixtures) ; rocher cassé = éclats et tuile LDtk masquée (`map_ldtk::hide_broken_rock_tiles`, dépendance `bevy_ecs_tilemap` ajoutée : `Cargo.lock` +1 ligne) ; mur secret = poussière + anneau ; sprites de bombe/rocher cachés dès que l'entité est `RollbackDespawned`.
- **Preuve** : sept genres de `FeedbackKind` + `note` dans `FeedbackCue`, moments clés `feedback` ; 23 scénarios gagnent des `Event` (traces inchangées) ; 8 tests unitaires `feedback_v3`.
- Assets : 6 PNG originaux CC0 générés par `scripts/generer-sprites-feedback.py` (`assets.yaml`, placeholders).

## 2. Vérifié (résultats réels, après fusion de `origin/main` 0a719425)
- `make test_scenarios_par SHARDS=3` sans BLESS : 482 scénarios, 3 tranches ok, 448 s, **aucune trace modifiée**.
- `make test_crates` : 985 réussis, 0 échec.
- `make lint` : 5 jeux, aucune erreur. `make gen` ×5 (zombies, testbed, throne, gungeon, isaac) : ok, **aucun diff**.
- `cargo fmt --all -- --check` OK ; `make check_sim_boundary` OK ; `check-forbidden.sh`, `check-rollback-registration.sh`, `make test_tools` OK.
- Captures `play_scenario --capture` (Metal, 960×540, pas de lavapipe) : `docs/taches/rapports/m3-t17-feedback-v3/*.png`.

## 3. Frames à regarder (numéros de frame de simulation)
- `bombe-meche-explosion.png` (`bomb_place_explode`) : f14 à 36 clignotement rouge, f36 anneaux.
- `rocher-casse.png` (`bomb_breaks_rock`) : f100 trois rochers, f134 bombe rouge, f138 éclats, f150 passage libre.
- `mur-secret.png` (`secret_wall_bomb`) : f96 anneau + poussière.
- `larmes-poison.png` (`tears_luck_chance`, f26+), `larme-spectrale.png` (`tears_spectral_wall`, f24-52), `larmes-guidee-division.png` (`tears_stack_three`, f20, 44-56 : enfants plus petits).
- `familiers.png` (`familiar_two_copies`, f4-120), `champion-explosif.png` (`champion_explodes`, f10 rouge, f30 anneau rayon 70).

## 4. Points à relire / limites
- Aucun son (pas d'asset licencié).
- Le mur secret n'a pas de visuel propre : la tuile noire de la carte de test reste, seule la poussière annonce l'ouverture.
- Bombe de champion (mèche d'une frame) : un sprite une frame et « mèche (0 frames) ».
- Dépendance `bevy_ecs_tilemap` dans `map_ldtk` (crate PENDING, hors frontière SIM).
- Limites §31 (tick multi-frames, indice prédit annulé).
- Non fait : p2p à deux clients, bench (hors portée ; rien dans `GgrsSchedule`).
