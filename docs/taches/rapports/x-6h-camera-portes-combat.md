# X-6h — Caméra fixe par pièce et portes fermées pendant le combat

- **Ce qui existait déjà** : salles `locks: true` (`room_lock_system`, `crates/map_ldtk/src/game/rooms.rs:754`, `RoomStates` `Dormant → Locked → Cleared`), portes et jumelles refermées puis rouvertes, ennemis endormis, **rappel des joueurs absents à l'entrée** (à terre et feu follet compris), portes à clé jamais touchées par une salle (§ 117), caméra `room_frame` / `frame_in_room` (`crates/game/src/camera/mod.rs:122`).
- **Ce qui manquait** : (1) une porte à serrure déjà ouverte ne se refermait pas derrière le joueur ; (2) la règle coop n'était pas une donnée ; (3) pas de zoom ajusté à la salle ni de glissement d'une salle à l'autre.
- **Ce qui est ajouté** : `CameraSettings::room_fit` et `room_slide_seconds` (`fit_room_zoom`, `room_slide_factor`, purs, présentation seule) ; `room_lock_system` referme une porte à serrure ouverte et la rouvre à la fin du combat (`still_keyed`, `kept_closed`) ; `RoomKindDef::no_rally` (`RoomKindTable::rallies`). Aucun type ni système rollback neuf. `docs/conventions.md` § 130, dettes D228-D229.

## Relevé (avant code)
L'enchaînement `porte_sortie_interieur` n'a aucune salle de combat qui verrouille : les 28 gabarits de `interieur.ldtk` sont `locks: false` ou vides (« combat » de remplissage sans ennemi) ; le verrou ne se déclenche que s'il y a un ennemi vivant. La preuve se fait donc sur `salles.ldtk` et `portes_cle_salles.ldtk` (existants) et deux cartes neuves (`scripts/construire-cartes-x6h.py`). Trou trouvé : `room_lock_system` traitait toute porte à serrure comme « scellée » (ni refermée, ni rouverte), donc une porte à clé ouverte restait ouverte derrière le joueur.

## Choix coop
Ramener l'équipe à l'entrée (règle d'Isaac, déjà là), désactivable par type de salle (`no_rally`). La salle qui attend se bloquerait avec un coéquipier à terre, feu follet ou déconnecté. À terre / feu follet : ramenés dans leur état, ne déclenchent pas le verrou.

## Preuve
| Point | Issue | Chemin |
|---|---|---|
| (b) un joueur : fermées puis ouvertes | vert, traces neuves | `tests/scenarios/combat_portes_solo`, `combat_salle_videe_stable`, `combat_cle_manquante`, `combat_porte_cle_referme` |
| (b) deux joueurs | vert, traces neuves | `combat_coop_a_terre`, `combat_coop_feu_follet`, `combat_coop_sans_rappel` (+ `rooms_lock_coop` existant : rappel debout) |
| (c) test pur caméra | vert (6 tests `camera::tests`) | `crates/game/src/camera/mod.rs` |
| (e) traces avant/après | 118 scénarios existants (room*, key_door*, door_key*, sortie_porte_*, interieur_*, wisp_*, isaac*, gungeon*, arena*, boss*, secret*, bot_isaac*, grid*) : **tous verts, traces inchangées, aucun BLESS** (`make test_scenarios SCENARIO=…`, 2205 s) ; `schedule_order` : aucun système ni type rollback ajouté | journal local `/tmp/x6h-family.log` |
| (d) capture du vrai jeu | fait : testbed natif release, fenêtre sur ce Mac, piloté par `alacod remote` (`input`, `step`, `screenshot`, 124 images assemblées à 6 i/s, 20,7 s, 59 Ko) : départ cadré par `room_fit`, entrée dans la salle de combat (battants fermés), combat, battants rouverts, glissement vers la salle suivante. Carte `salles.ldtk` graine 1, `camera.ron` du testbed modifié le temps de la capture (`room_frame`, `room_fit`, `room_slide_seconds: 0.35`, `min_zoom: 0.15`), `game.ron` idem, rétablis sans commit ; `target/release` supprimé | `docs/taches/rapports/preuves/x-6h/` (`x-6h-combat-portes-camera.mp4`, 4 images) |
| (f) contrôle rapide découpé | **vert** sur `0e3cc75e` (code, fusion de `origin/main` dc734091 comprise) : `make format test_tools check_forbidden check_rollback_registration check_sim_boundary lint check_integration` code 0 (1231 s) ; `test_crates` découpé : lib, bins, doc puis les 59 binaires d'intégration, 0 échec, `schedule_order` vert (1 test). Coupé une fois par la garde disque (29 Go) à `placement`, la suite rejouée depuis `placement` (10 binaires, 0 échec) | `~/orca/runs/alacod/logs/x-6h-check-rapide-<sha>.log` |

## Ce qui reste
- La capture n'a pas de salle « couloir » séparée (les salles du testbed font 128 px, le départ en tient lieu) ; le défaut de `camera.ron` du testbed n'est pas changé.
- Dettes D228 (limites de la caméra) et D229 (pas de « la salle attend »).

## Corrections de revue (22 nov 2026, verdict « merge après corrections »)
Tests nouveaux : rouge avant / vert après ; `cargo test --profile headless`, préfixe machine du 21 nov.

| Point | Correction | Rouge avant | Vert après |
|---|---|---|---|
| P2.1 | `still_keyed` (`rooms.rs`) : `lock.is_some() && (Interactable présent \|\| !DoorConfig::interactable)` : une porte à serrure non interactive reste fermée à la fin d'un combat ; test `une_porte_a_serrure_non_interactive_reste_fermee_apres_le_combat` (+ `…ouverte_se_referme_puis_se_rouvre`, garde du comportement voulu) | échec sur `fe20c153` : `assertion failed: …contains::<Collider>()` (rooms.rs:1751) | 2 tests verts |
| P2.2 | `PlayerPosition(handle: 1)` dans `combat_coop_a_terre` et `combat_coop_feu_follet` : (−904, −600) à f125 (départ, hors de la salle de combat), (−776, −576) à f230 (devant la porte d'entrée), tolérance 8 | — (attentes neuves, valeurs lues dans l'échec d'une première passe) | verts, **traces inchangées** |
| P2.3 | test pur `une_salle_plus_grande_que_la_vue_tient_avec_les_valeurs_du_testbed` : lit le `camera.ron` livré du testbed (`min_zoom` 0,6, `max_zoom_out` 3,0), fenêtre 960 × 540 : salle 1 600 × 900 → zoom 1,667, vue fixe centrée ; salle 4 000 × 600 → zoom maxi, la vue suit. **Pas de build release.** La preuve visuelle d'une grande salle se fera dans 1837 (ses salles dépassent la vue), au réépinglage | (API neuve : ne compile pas sur `fe20c153`) | vert |
| P3.1 | `RoomSlide::step` : glissement au changement de salle (durée) ou si `room_fits` ; sinon `lerp_speed`; test `le_glissement_ne_vaut_qu_au_changement_de_salle_ou_si_la_salle_tient` | (API neuve) | vert |
| P3.2 | avertissements `LintWarningKind::InertRoomSetting` (`no_rally` sans `locks`/`arena`, `room_fit`/`room_slide_seconds` sans `room_frame`, `room_slide_seconds` négatif ou non fini) ; `Registry::camera_rooms` ; `crates/content/tests/x_6h_room_setting_fixtures.rs` (fixtures `room_setting_inert`, `room_setting_ok`) | (kind neuf) | 2 tests verts |
| P3.3 | `combat_porte_cle_referme` : `MapDoor(nth 0 et 1, Open, f162)` (porte à clé ouverte avant la fermeture ; à f200 la salle est déjà verrouillée) | — | vert, trace inchangée |
| P3.4 | `tags: ["rapide"]` sur `combat_cle_manquante` (360 images), `combat_portes_solo` (650), `combat_coop_a_terre` (850) | — | joués par `make check_integration` |

Les sept scénarios `combat_*` rejoués : verts, aucune trace modifiée (`git status` : seuls les `.ron` changent). `origin/main` fusionnée (`git merge`, deux commits de docs).

## Dernières lignes du contrôle rapide (suite depuis `placement`)
```
scenario/schedule_order code=0 passed=1 failed=0
scenario/zigzag_return code=0 passed=2 failed=0
binaires joués : 10 / 11 tests d'intégration + lib/bins/doc ; failed : 0, codes non nuls : 0
```
