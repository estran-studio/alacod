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

## 5. Revue indépendante : corrections
1. **BLOQUANT fermé : tuile du rocher.** `hide_broken_rock_tiles` (indice `RockBroken`, jamais réaffichée) est remplacée par `map_ldtk::plugins::sync_rock_tiles` : chaque tick, la tuile non nulle de la couche `Rocks` (`LayerMetadata.identifier`, parent de la tuile) est visible ssi un `world::Rock` vivant (les `RollbackDespawned` sont exclus de la requête) est à < 4 unités. Juste après un tick à plusieurs frames et après un rollback qui annule la casse. Tests headless : `la_tuile_suit_l_etat_du_rocher` (rocher retiré → tuile cachée, autre couche intacte, rocher ressuscité → tuile revue), `rayon_de_la_tuile`. Capture `bomb_breaks_rock` f138 revérifiée : tuile partie, éclats visibles.
2. **Attentes `Event`** : 38 ajoutées au départ ; **24 retirées** (celles qui redisent un état : `Explosion`, `SecretWall`, `Familiar`, `BombFuse`, `RockBroken` — 12 scénarios n'ont plus de changement) ; **14 gardées** dans 11 scénarios (`Tear` via `tear_look` : `tears_*` ; `Champion` via la teinte `variant_tint` : `champion_*`). Pas de proof headless du rendu conditionné à `explosion.enabled`/brouillard (il est dans le système de rendu `apply_feedback_cues`, absent en headless, et aucune attente de compteur de cues n'existe) : preuve = captures et vidéos. Fixtures ajoutées : `feedback_explosion_shake_frames_too_long`, `feedback_explosion_color_out_of_range` (4 fixtures `explosion` au total).
3. **Vidéos** : `ffmpeg` de `/opt/homebrew/bin` a `libx264` mais **pas `drawtext`** : `make videos` / `scripts/scenario-video` échoue à l'encodage. Vidéos produites à la main (captures `play_scenario --every 2`, 30 images/s, sans bandeau de frame) dans `docs/taches/rapports/m3-t17-feedback-v3/videos/` (copie de `target/videos/m3-t17/`) : `bomb_place_explode.mp4`, `tears_luck_chance.mp4` (larmes empoisonnées), `familiar_follow.mp4`, `champion_explodes.mp4`, `bomb_breaks_rock.mp4`, chacune avec son `.events.json`. Image n = frame 2n.

Résultats après correctifs : `make test_scenarios_par SHARDS=3` sans BLESS : 482 scénarios, 3 tranches ok, 236 s, aucune trace modifiée ; `cargo test -p map_ldtk -p game -p content` ok ; `make test_crates` : 987 réussis, 0 échec ; `make lint` ×5 aucune erreur ; `cargo fmt --all -- --check` OK ; `make check_sim_boundary` OK.
