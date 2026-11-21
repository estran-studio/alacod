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
| (d) capture du vrai jeu | **en attente** (feu vert du coordinateur : disque au plancher) | `docs/taches/rapports/preuves/x-6h/` |
| (f) contrôle rapide découpé | **en attente** (même raison) | `$HOME/orca/runs/alacod/logs/x-6h-check-rapide-<sha>.log` |

## Ce qui reste
- Capture release (couloir → salle de combat → portes rouvertes → salle suivante, `room_fit` et `room_slide_seconds` posés temporairement dans `camera.ron` du testbed, `min_zoom` abaissé : les salles du testbed font 128 px), puis suppression de `target/release`.
- Contrôle rapide découpé, dernières lignes du journal ici.
- Dettes D228 (limites de la caméra) et D229 (pas de « la salle attend »).
