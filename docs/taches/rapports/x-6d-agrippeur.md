# X-6d — S'agripper au joueur : `Grab` (voie m, jalon S4 dim 6 déc, parti en avance le 20 nov)

Branche `wquintal/x-6d-agrippeur` (partie d'`origin/main` d63deb33). Plan : `docs/taches/plan-1837-cli.md` § 6, ligne X-6d (sa preuve fait foi). Section de conventions : **§ 126**. Dette : **D222**.
Travail sur le VPS (cible partagée `target-partage`). Rien côté 1837, rien de 1837 dans l'engine (testbed : `agrippeur`, `agrippeur_devie`, pantin réduit ; fixtures `grabber`).
Jalon servi : **S4 dim 6 déc** (« lutins qui s'agrippent »).

## Points du lot

| # | Point (preuve du § 6) | Issue | Preuve |
|---|---|---|---|
| 1 | Relevé avant code | **fait** | `status` du 20 nov (avant la première ligne), repris dans le § 126 ; aucun mécanisme d'attache existant (les familiers suivent par un mouvement) : composant `sim_core::grab::Riding` neuf |
| 2 | Behavior générique à nom neutre, valeurs en données (bond, accroche, gêne ralentie et/ou déviée, presque sans dégâts, décrochage au secouage ou à la mêlée, étourdi, durée maximale, nombre borné) | **fait** | `Behavior::Grab { reach, telegraph, leap_speed, leap_frames, hold, slow, sway, shake, shake_window, stun, cooldown, max_riders, bite_every }` (fin d'enum, `behaviors/src/lib.rs`), `GrabPhase` (`ai/state.rs`), système `ai/grab.rs` ; aucune valeur de jeu dans l'engine (défauts : bond ×3, fenêtre 20 frames) |
| 3 | Règles GGRS | **fait** | itération `order_iter!`/`order_mut_iter!`, `Fixed` et entiers, aucun RNG, aucune `HashMap` ; `BehaviorRuntime` (déjà neutre) gagne `grab`, `grab_ready_at`, `grab_bite` hachés et vidés seulement actifs ; `Riding` type rollback neuf **neutre** ; aucun `despawn` (un agrippeur tué meurt par le chemin existant, `despawn_rollback`) ; traces `ggrs{f=… grab net_id=… phase=…}` ; le runner joue chaque scénario en synctest |
| 4 | Présentation : poses `saut`, `accroche` (sur l'ancre des épaules), `etourdi` ; lint `MissingPose`, `animation_vivante` | **fait** | champs `poses.leap`, `poses.ride`, `poses.stunned` de `models.ron` (1837 y met ses clips `saut`, `accroche`, `etourdi`) ; `pose::GrabPose`, `playback::grab_pose`, `sim_activity` (« bond », « accroché », « étourdi ») ; `placement::ride_on_shoulders` + `ModelDef::shoulders_m` ; clips `leap`, `ride`, `stunned` du pantin (`scripts/gen-pantin.py`, générateur reproductible : les clips existants sont identiques, seules trois entrées s'ajoutent) ; fixtures `model_poses_grab_missing` (3 poses réclamées) / `_complete` |
| 5 | Scénarios neutres, chaque attente rouge sur régression provoquée | **fait** | douze scénarios `grab_*` et leurs `.trace` neuves (§ 126) ; attente neuve `GrabState` ; onze régressions provoquées par des interrupteurs temporaires (jamais commités), toutes rouges puis vertes : journal `~/orca/runs/alpha/logs/x-6d-agrippeur-regressions.log` (tableau ci-dessous) |
| 6 | Décrochage au stick **par l'entrée réelle** (T-3b) et au clavier | **fait** | `gestes_entree_reelle.rs` : `agrippeur_secoue_au_stick` (axe `LeftStickX` ±1 toutes les 4 frames), `agrippeur_secoue_au_clavier` (A / D), `agrippeur_frappe_depuis_la_manette` (bouton B) ; chemin leafwing → `collect_device_inputs` → `read_local_inputs` → enregistreur → runner ; verts, rouges sous `shake`, `melee` et `pose` |
| 7 | Traces existantes intactes | **fait, sur un échantillon** | 107 scénarios (`zigzag_*`, `retour_*`, `rodeur_*`, `repli_*`, `couvert_*`, `enemy_*`, `model_*`, `frappe_*`, `coup_*`, `tireur_*`, `four_players_*`, `two_players_*`, `tears_*`, `idle*`, `wisp_*`, `charge*`, `dash_*`, `arme_lourde*`, `status_*`, `familiar_*`, `bomb_*`, `melee*`) rejoués sans `BLESS` : `ok`, aucune `.trace` modifiée. La suite complète : la nuit du VPS |
| 8 | Lint, `describe` | **fait** | `OutOfRange` (bornes, ordre : seul `Flee` précède) fixture `behavior_grab` ; `cli.rs::describe_lists_grab` |
| 9 | Jeu court étiqueté | **fait** | `grab_secoue_clavier` (`tags: ["rapide"]`, 240 frames) |
| 10 | Vidéo | **fait** | `make videos` (Xvfb privé, `xauth` absent du VPS pour `xvfb-run`) : `grab_bond_accroche`, `grab_secoue_clavier`, `grab_deux`, `grab_coop`, `grab_devie`, `grab_duree_max` + `montage.mp4`, copiés dans `~/orca/runs/alpha/logs/x-6d-videos/` ; regardé : l'agrippeur réduit se ramasse, bondit, se pose sur les épaules du pantin (ancre), retombe assis et sonné à ses pieds |
| 11 | `make check_sim_boundary`, `make schedule_order_diff` | **fait** | `check_sim_boundary` : « OK : frontière simulation/présentation respectée par 13 crates ; 2 en dette D69 » ; `schedule_order_diff BASE=origin/main` : voir § Ordre des systèmes |
| 12 | Contrôle rapide | voir § Contrôle rapide | |

## Régressions provoquées (journal `x-6d-agrippeur-regressions.log`)

| Interrupteur (brique cassée) | Attentes rouges |
|---|---|
| aucun | tout vert (12 scénarios, 3 gestes) |
| `shake` (le secouage ne décroche plus) | `grab_secoue_clavier` (Dazed, MoveSpeed rendu, pose `stunned`, Idle, Chase), `grab_etourdi_frappe`, gestes stick et clavier |
| `melee` (ni le coup du porteur ni celui reçu) | `grab_melee_porteur`, `grab_coop`, geste du coup à la manette |
| `slow` (aucune gêne) | `grab_gene_vitesse` (Stat 62, x 486), `grab_devie`, `grab_deux` (Stat 62 au lieu de 15,5) |
| `riders` (`max_riders` ignoré) | `grab_deux` (le troisième s'accroche) |
| `timeout` (`hold` ignoré) | `grab_duree_max` |
| `sway` (aucune déviation) | `grab_devie` (y 696) |
| `window` (le compte ne repart jamais) | `grab_secoue_lent` (décroché à tort) |
| `snap` (l'agrippeur ne suit pas son porteur) | `grab_bond_accroche`, `grab_etourdi_frappe` |
| `pose` (aucune pose de `Grab`) | `Clip` de `grab_bond_accroche`, `grab_duree_max`, `grab_secoue_clavier`, invariant `animation_vivante` (« état « bond » … le clip joué `idle` »), les trois gestes |
| `bullets` (les balles du porteur le touchent) | `grab_tir_porteur` (30 → 15 PV) |
| `soft` (ses agrippeurs le « poussent ») | `grab_gene_vitesse` (x 448,8) |

## Ordre des systèmes (`make schedule_order_diff BASE=origin/main`)

Le script a d'abord répondu « identiques » : faux positif, la base et l'arbre de travail partageant la cible (`target-partage`), cargo a jugé l'arbre de travail
à jour avec les artefacts de la base, compilée en dernier (piège déjà noté dans `scripts/scenario-video`). Sortie de l'arbre refaite après `touch` des fichiers
du lot (`ALACOD_SCHEDULE_DUMP`, mêmes options), comparée à la sortie de base : `~/orca/runs/alpha/logs/x-6d-schedule/` (`base/`, `head2/`, `ensembles-testbed.diff`).
- **Types rollback** (cinq jeux) : seul `Riding` s'ajoute (après `BehaviorRuntime`), les autres gardent leur ordre relatif.
- **Systèmes, comme ensembles** (cinq jeux, identique) : s'ajoutent `grab_system`, un `apply_deferred` (les commandes `insert`/`remove` de `Riding` avant
  `update_enemy_targets` et `move_enemies`), les quatre systèmes de snapshot de `Riding` et `neutral_component_checksum<Riding>` ; rien ne disparaît. Dans `GgrsSchedule`,
  `behavior_select_system → grab_system → update_enemy_targets → move_enemies`.
- **Ordre** : 163 lignes changent de rang dans chaque jeu, entre systèmes sans contrainte d'ordre entre eux (d'abord les `clear_frame_events<…>` de `FrameStart`) : le tri
  topologique de bevy ne suit pas l'ordre d'insertion (`CLAUDE.md`). Aucune trace existante ne bouge (107 scénarios rejoués), ce qui est la preuve attendue.

## Choix et pourquoi

- **Un composant `Riding` dans `sim_core`** plutôt que l'état seul dans `BehaviorRuntime` : `combat` (balles, projectiles) doit savoir qu'un agrippeur est porté par le tireur, et `combat` ne voit pas `game`.
- **La gêne est un modificateur `MoveSpeed`** (comme le `Slow` des statuts) : visible par `Stat`, multiplié par agrippeur, renouvelé à `until` court (un agrippeur tué ne laisse rien), sans allocation par frame.
- **Étourdi = phase de `Grab`**, pas le statut `Stun` : sa durée est la donnée de la règle (pas un fichier de statut à référencer) et la pose `stunned` se lit sur la phase.
- **Cible du bond prise à la fin du télégraphe** : le ramassé se lit, le joueur qui bouge pendant le bond l'esquive encore.
- **Secouage sur les boutons de déplacement de l'input rollback** : stick et clavier passent par les mêmes bits ; rien hors simulation.

## Pour 1837 (à l'arrivée du `rev`)

Le lutin s'écrit dans son RON : `behaviors: Some([Grab(reach: …, telegraph: …, leap_frames: …, hold: …, slow: …, shake: …, stun: …, cooldown: …, max_riders: …, bite_every: Some(…)), Chase(profile: "Ground")])`
(ajouter `sway: Some((speed: …, period: …))` pour « dévié »), `ai.attack_range` = portée du contact du bond, `ai.attack_damage` = morsure. Dans `models.ron`, son modèle porte
`poses: (idle, walk, leap: Some((still: Some("saut"))), ride: Some((still: Some("accroche"))), stunned: Some((still: Some("etourdi"))))` ; le modèle du Frère peut déclarer `shoulders_m`
(hauteur de ses épaules, sinon 0,8 × `height_m`). Les valeurs (durée accroché, force de la gêne, nombre à la fois) restent ouvertes dans `design/production/lutin.md` : ce sont des données.

## Non fait / limites

- D222 (bond en ligne droite, chute à la position du porteur, secouage par seuils d'axe, guidage des projectiles, ancre par hauteur, valeurs d'essai).
- Pas de preuve p2p réelle (le synctest du runner rejoue chaque scénario avec rollbacks) ; pas de suite complète (la nuit du VPS).
