# Rapport assets-lot-a (Mac de William, 2026-10-14)

Lots A (sons de tir, ferme D32) et B (sons d'événements) de `docs/plan-assets.md`. Branche
`assets-lot-a` depuis `origin/main` (2936b727).

## 1. Fait

### Lot A — sons de tir (commit d7618bca)

| Fichier (4 jeux) | Source | Auteur | Licence (lue sur la page) | Modifications |
|---|---|---|---|---|
| `sounds/shot_pistol.ogg` (0,45 s) | [Chaingun, pistol, rifle, shotgun shots](https://opengameart.org/content/chaingun-pistol-rifle-shotgun-shots), `pistol.wav` | Michel Baradari (soumis par qubodup) | [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/) (page + `info.txt` de l'archive) | rogné, fondu 0,15 s, mono, -3 dBFS crête |
| `sounds/shot_shotgun.ogg` (0,50 s) | idem, `shotgun.wav` | idem | idem | rogné avant le bruit de pompe, fondu 0,15 s, mono, -3 dBFS |
| `sounds/shot_smg.ogg` (0,30 s) | idem, `cg1.wav` (chaingun) | idem | idem | rogné, fondu 0,12 s, mono, -3 dBFS |
| `sounds/reload.ogg` (0,58 s) | [Gun Reload Sound Effects](https://opengameart.org/content/gun-reload-sound-effects), `clipload2.wav` + `clipload1.wav` | Brian MacIntosh (BMacZero) | [CC0](https://creativecommons.org/publicdomain/zero/1.0/) (« Optionally credit ») | assemblé (sortie, 0,12 s, entrée du chargeur), -3 dBFS |

Tous en ogg vorbis mono 44,1 kHz (`oggenc -q 5` : l'ffmpeg de Homebrew n'a pas `libvorbis`,
`vorbis-tools` installé). Crêtes mesurées après encodage : -2,9 à -2,2 dBFS.

- **Code** (`crates/game/src/feedback.rs`) : `play_shot_sound_for_new_bullets` lit le mode de tir
  actif de l'arme active du tireur (`WeaponInventory` + `WeaponState.active_mode`) à l'image
  rendue et choisit `shot_shotgun` (`Shotgun`), `shot_smg` (`Automatic`/`Burst`) ou
  `shot_pistol` (`Manual`), repli sur `shot` (`shot_sound_key`, `shot_sound_path`, test
  `cle_de_famille_puis_repli_sur_shot`). Retirés : `SHOT_SOUND_MAX`, `SHOT_SOUND_FADE`,
  `ShotSounds` (dont `playing`) et l'import `Duration` ; la déduplication par (tireur, frame de
  création) reste.
- **Décision** : famille déduite du mode de tir plutôt qu'un champ dans la config d'arme.
  `WeaponConfig` est haché dans le checksum (`Weapon` rollback) et `Bullet` aussi : un champ de
  plus aurait demandé un `Hash` manuel ; le mode de tir ne touche pas la simulation. Une arme
  dont le mode trompe (pistolet `Automatic` de gungeon) reçoit sa clé par `shot_sounds` dans
  `feedback.ron` (lot B, ci-dessous). Le test unitaire du lot A a été remplacé par
  `arme_puis_famille_puis_repli_sur_shot` dans le commit du lot B.
- **Contenu** : `feedback.ron` des 4 jeux (`shot_pistol`, `shot_shotgun`, `shot_smg`, `reload`) ;
  `audio_config` des armes de zombies et testbed (lu par le lint seulement) vers les nouveaux
  fichiers ; `machine-gun.ogg` et `machine-gun-reload.ogg` (Pixabay Content License) supprimés
  des 4 jeux.
- **Registres** : entrées `sound-shots-michel-baradari` et `sound-reload-bmaczero` dans
  `assets.yaml` de zombies, throne, gungeon ; `games/testbed/assets/assets.yaml` **créé**
  (testbed n'en avait pas ; il n'enregistre que les sons, les autres fichiers sont des copies de
  zombies).
- **Docs** : `docs/conventions.md` §7, `docs/plan-assets.md` (inventaire, bilan licences, lot A
  fait), `docs/taches/dettes.md` (D32 fermée), doc de `FeedbackSettings::sounds`.

### Lot B — sons d'événements (commit 03602970)

| Clé / fichier | Source (page lue le 2026-10-14) | Fichier d'origine | Licence | Crête |
|---|---|---|---|---|
| `hit` (0,12 s) | [Kenney Impact Sounds](https://kenney.nl/assets/impact-sounds) | `impactSoft_medium_000.ogg` | [CC0](https://creativecommons.org/publicdomain/zero/1.0/) (page + `License.txt`) | -9 dBFS (son fréquent) |
| `hurt` (0,65 s) | idem | `impactPunch_heavy_000.ogg` | CC0 | -3 dBFS |
| `death` (0,53 s) | idem | `impactSoft_heavy_000.ogg` | CC0 | -6 dBFS |
| `dash` (0,66 s) | [Kenney RPG Audio](https://kenney.nl/assets/rpg-audio) | `cloth1.ogg` | CC0 | -6 dBFS |
| `door` (0,92 s) | idem | `doorOpen_1.ogg` | CC0 | -3 dBFS |
| `blank` (0,95 s, gungeon et testbed) | [Kenney Sci-fi Sounds](https://kenney.nl/assets/sci-fi-sounds) | `forceField_000.ogg` | CC0 | -3 dBFS |

Mono 44,1 kHz, ogg vorbis (`oggenc -q 5`), entrées `sound-impact-kenney`,
`sound-rpg-audio-kenney`, `sound-sci-fi-kenney` dans les quatre `assets.yaml`.

- **Code** (`crates/game/src/feedback.rs`, `FeedbackPlugin`, `PostUpdate`, absent en headless) :
  `FeedbackSounds` (`SystemParam`, joue une clé de `sounds`, journal `feedback f<N> sound <clé>`)
  et six systèmes qui lisent l'état ou les `FrameEvents` de la dernière frame simulée, sans
  rien écrire dans la simulation :
  - `hit` : `DamageEvent` d'un joueur local vers une autre entité, une fois par frame simulée
    (gardé par `DamageEvent.frame`) ;
  - `hurt` : baisse de `Health.current` d'un joueur local à `max` égal, au plus un son toutes les
    15 frames (`HURT_SOUND_MIN_FRAMES` : en gungeon les PV baissaient aux frames 522 et 523,
    deux sons à 16 ms d'écart) ;
  - `death` : ennemi portant `Death` et `RollbackDespawned` (la requête nomme le composant
    désactivant), une fois par `GgrsNetId` ;
  - `dash` : front montant de `DashState::is_dashing` ;
  - `blank` : `FrameEvents<BlankUsed>` d'un joueur local ;
  - `door` : porte fermée à l'image précédente, ouverte maintenant, sauf à l'image où
    `RoomStates::doors_opened` passe à `true` (au premier essai, l'ouverture d'office des portes
    de salle de gungeon sonnait à la frame 1).
- **`shot_sounds`** (ajouté après l'essai en jeu) : le « pistolet » de gungeon est `Automatic` et
  jouait `shot_smg`. `FeedbackSettings::shot_sounds` (nom d'arme → clé de `sounds`) passe avant
  la famille du mode ; gungeon : `shot_sounds: {"pistolet": "shot_pistol"}`. Lint : arme à
  distance connue et clé présente dans `sounds` (fixture `feedback_shot_sounds_unknown`, test
  `feedback_shot_sounds_fixture_reports_weapon_and_key`, ajoutée à la table
  `t3_4_fixtures_have_a_single_rule_failure`). Test unitaire
  `arme_puis_famille_puis_repli_sur_shot` (ordre : arme, famille, `shot`, rien).
- **Pas de son de coffre** : les coffres (M2-T5) ne sont pas dans `main`.

## 2. Vérifié

Toutes les commandes lancées sur le Mac, sur l'état final (commit 03602970) sauf mention.

| Commande | Résultat |
|---|---|
| `make test_scenarios` (complet, sans `BLESS`) sur le lot A (d7618bca) | `test scenarios ... ok`, 250 scénarios joués (159 + générés), 3 réussis / 0 échec / 8 ignorés, 1 193 s ; avertissement de budget `window_repair` 78,2 < 100 fps (machine chargée, non bloquant) |
| `make test_scenarios` (complet, sans `BLESS`) sur l'état final | `test scenarios ... ok`, 250 scénarios, 3 réussis / 0 échec / 8 ignorés, 929 s ; avertissement de budget `ammo_burst` 68,7 < 70 fps (non bloquant) |
| `make test_crates` | 668 réussis / 0 échec / 10 ignorés (42 binaires de test) |
| `cargo test -p game --lib shot_sound` | `arme_puis_famille_puis_repli_sur_shot ... ok` |
| `make lint` (zombies, testbed, throne, gungeon) | « aucune erreur » pour les quatre |
| `cargo fmt --all -- --check` | rien à afficher |
| `./scripts/check-rollback-registration.sh` | OK |
| `./scripts/check-forbidden.sh` | 6 avertissements, tous dans des fichiers non touchés (préexistants) |
| YAML des 4 `assets.yaml` (`ruby -ryaml`) | se chargent ; ids attendus présents |
| Fichiers son (`ffprobe`, `volumedetect`) | vorbis, 1 canal, 44 100 Hz ; crêtes -2,2 à -2,9 dBFS (lot A), -2,6 à -9,0 (lot B, voir tableau) |

**En jeu** (`cargo run -p <jeu> --features native`, rendu sur l'écran du Mac, piloté par
`ALACOD_REMOTE=1` + `scripts/alacod-remote`), journal `feedback f<N> sound …` relu :
- **gungeon** : pistolet → `shot_pistol.ogg` à chaque tir (avant `shot_sounds` : `shot_smg.ogg`,
  corrigé) ; `hit` sur les coups au but, `death` une fois par ennemi tué, `reload`, `dash`,
  `blank`, `hurt` (contact ennemi), `door` à la frame où la dernière créature de la salle meurt
  (f581) ; plus de `door` à la frame 1 après le correctif `RoomStates::doors_opened`.
- **throne** (pilote, `lance_lames` remplacé **temporairement** par `fusil_a_pompe` dans
  `characters/pilote.ron`, remis à l'identique avant commit, `git diff` vide) :
  mitraillette → `shot_smg.ogg` ×5, revolver → `shot_pistol.ogg` ×2, fusil à pompe →
  `shot_shotgun.ogg` ×1 pour 8 plombs.
- **zombies** (1 200 frames, vague 1) : 50 `shot_smg`, 21 `hit`, 3 `death` pour 3 kills,
  6 `hurt` espacés de ≥ 15 frames, 1 `reload`, aucun `door` parasite.
- Aucune erreur de chargement audio (`error`/`failed`/`warn` audio) dans les journaux.

## 3. Non fait / incertain

- **Écoute à l'oreille : non faite.** L'agent lance le jeu et voit l'écran, mais n'entend pas
  la sortie audio : ce qui est vérifié, c'est que chaque son est **déclenché** au bon moment,
  une fois, avec le bon fichier, et que les fichiers sont valides et au bon niveau. Le timbre
  (le coup de pistolet sonne-t-il comme un pistolet, le `dash` en froissement de tissu
  convient-il, `hit` à -9 dBFS est-il audible sous les tirs) est **à écouter par William** :
  `cargo run -p gungeon --features native` puis tirer, dasher, blank (touche du blank), finir
  une salle.
- Choix des échantillons du lot B faits sur le nom et la mesure (durée, crête), pas à
  l'oreille ; chacun se remplace dans `feedback.ron` sans code.
- `play_scenario` n'installe pas de journal (`LogPlugin` désactivé) : les essais par arme sont
  passés par les binaires de jeu, pas par les scénarios générés.
- p2p non joué : le son ne lit que l'état local et rien n'entre dans la simulation ; en p2p, un
  coup ou une mort prédits puis annulés par un rollback peuvent sonner (comme les indices de
  `FeedbackLog`, §31).
- Testbed : sons copiés et lint vert, pas lancé en jeu.

## 4. Dettes, questions

- **Coffre** (lot B) : à brancher quand M2-T5 sera dans `main` (`FrameEvents<ChestOpened>` ou
  le front « vidé » par `GgrsNetId`, même forme que `blank`/`door`).
- Un seul son de rechargement pour toutes les armes ; armes lourdes (`lance_grenades`,
  `roquette`, `mortier` de throne) sur `shot_pistol` faute d'échantillon dédié : à ajouter par
  `shot_sounds` quand un son d'explosif existera.
- `games/testbed/assets/assets.yaml` ne déclare que les sons : les autres fichiers du testbed
  (copies de zombies) restent à enregistrer.
- Question : volumes relatifs (`hit` -9, `death`/`dash` -6, autres -3 dBFS crête) à régler à
  l'écoute ; `feedback.ron` n'a pas de volume par clé (à ajouter si besoin).

## 5. Atteignabilité

Présentation seule, aucune mécanique de simulation. Les sons jouent dans toute partie ordinaire
des quatre jeux : tir, coup au but, kill, dégâts, dash, rechargement dès la première salle ou
vague ; `blank` en gungeon (touche du blank, objet de départ) ; `door` à la première salle
nettoyée de gungeon et à toute porte achetée de zombies (porte de zombies non essayée en jeu).
