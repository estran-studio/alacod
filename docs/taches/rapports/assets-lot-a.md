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
- **`shot_sounds`** (ajouté après l'essai en jeu, **remplacé ensuite**, voir « Sons
  paramétrables » ci-dessous) : le « pistolet » de gungeon est `Automatic` et jouait `shot_smg`.
- **Pas de son de coffre** : les coffres (M2-T5) ne sont pas dans `main`.

### Sons paramétrables (demande de William, 2026-10-15)

- **`audio_config` des armes lu par le jeu** : le champ existait dans `weapons.ron` (un tir et
  un rechargement par mode de tir) mais seul le lint le lisait. Il devient
  `combat::weapons::WeaponAudioConfig` sur `WeaponAsset` (`#[serde(default)]`, vide = rien),
  **jamais** copié dans le composant rollback `Weapon` (haché dans le checksum) : la
  présentation le relit dans `Assets<WeaponsConfig>` par le nom de l'arme
  (`game::feedback::WeaponAudio`).
- **Priorité** : tir = `firing` du mode actif de l'arme, sinon la clé de la famille du mode,
  sinon `shot` ; rechargement = `reloading` du mode actif, sinon `reload` (`shot_sound`,
  `reload_sound`, test `arme_puis_famille_puis_repli`). Le rechargement prend maintenant le son
  de l'arme active du joueur qui recharge.
- **`shot_sounds` retiré** (champ, lint, fixture, test) : doublon de `audio_config`. Gungeon :
  `audio_config` sur `pistolet` (`Automatic`) et `pistolet_jumele` (`Burst`) vers
  `sounds/shot_pistol.ogg`.
- **Volumes** : `volumes` dans `feedback.ron`, en dB par clé de `sounds` (plage [-60, 12],
  `SOUND_VOLUME_DB_RANGE`) ; un son d'arme prend le volume de la clé qu'il remplace (famille
  ou `reload`). `volumes: {}` dans les quatre jeux (aucun changement d'écoute).
- **Lint** : mode d'`audio_config` absent de `firing_modes` (fixture `audio_unknown_mode`) ;
  clé de `volumes` absente de `sounds` (fixture `feedback_volume_unknown_key`) ; volume hors
  plage (`FeedbackSettings::problems`, test `volume_hors_plage_signale`).
- Journal : `feedback f<N> sound <clé> <fichier>` pour tous les sons.

## 2. Vérifié

**Après le paramétrage audio (2026-10-15)** :
- `make test_scenarios` complet, sans `BLESS` : 302 scénarios, 3 tests réussis / 0 échec /
  8 ignorés, 613 s ; traces inchangées.
- `make test_crates` : 691 réussis / 0 échec / 10 ignorés (42 binaires).
- `make lint` : aucune erreur dans les quatre jeux ; `cargo fmt --all -- --check` passe.
- Essai du binaire natif gungeon par contrôle remote : tir pendant 40 frames, puis rechargement.
  Journal `logs/game_run_LGfvKV.log` : `feedback f21 sound shot_smg sounds/shot_pistol.ogg`,
  puis `feedback f46 sound reload sounds/reload.ogg`. La clé `shot_smg` désigne le volume de
  la famille `Automatic`, le fichier du pistolet vient de son `audio_config`.
- Limites : écoute à l'oreille non vérifiable par l'agent ; capture remote noire, donc aucune
  validation visuelle revendiquée pour cet essai audio.

`origin/main` a avancé pendant la tâche (M2-T14a, M2-T15, M2-T17, M2-T19 feedback v2,
perf-suite, M2-T16) : fusionné dans la branche (8480d7bd, sans conflit textuel), puis
`shot_sounds` étendu au `pistolet_jumele` de gungeon (`Burst`, arme de M2-T14a, 787e86c1).
Toutes les commandes ont tourné sur le Mac.

| Commande | Résultat |
|---|---|
| `make test_scenarios` (complet, sans `BLESS`) sur le lot A (d7618bca) | `test scenarios ... ok`, 250 scénarios joués (159 + générés), 3 réussis / 0 échec / 8 ignorés, 1 193 s ; avertissement de budget `window_repair` 78,2 < 100 fps (machine chargée, non bloquant) |
| `make test_scenarios` sur les lots A + B avant fusion (03602970) | `ok`, 250 scénarios, 0 échec, 929 s ; avertissement de budget `ammo_burst` 68,7 < 70 fps (non bloquant) |
| `make test_scenarios` **après fusion** (787e86c1) | `ok`, 302 scénarios, 3 réussis / 0 échec / 8 ignorés, 405 s, aucun avertissement de budget |
| `make test_crates` après fusion | 690 réussis / 0 échec / 10 ignorés (42 binaires de test ; 668 avant fusion) |
| `cargo test -p game --lib shot_sound` | `arme_puis_famille_puis_repli_sur_shot ... ok` |
| `make lint` (zombies, testbed, throne, gungeon), après fusion | « aucune erreur » pour les quatre ; avertissements d'atteignabilité de M2-T15 (armes non données, etc.), aucun sur les sons ou `feedback.ron` |
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
- **gungeon après fusion** (787e86c1, feedback v2 de M2-T19 présent) : 15 `shot_pistol`,
  8 `hit`, 2 `death`, 1 `reload`, 1 `dash`, 1 `blank` ; aucune panique.
- Aucune erreur de chargement audio (`error`/`failed`/`warn` audio) dans les journaux.

Note : pendant la première suite (lot A), les `feedback.ron` et les `.ogg` du lot B ont été
ajoutés au disque en cours de route (le binaire était celui du lot A) ; seule la table `sounds`
changeait, lue par la présentation. Les deux suites suivantes ont tourné sur un état figé.

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
- Un seul son de rechargement fourni pour toutes les armes ; armes lourdes (`lance_grenades`,
  `roquette`, `mortier` de throne) sur `shot_pistol` faute d'échantillon dédié : à donner par
  leur `audio_config` quand un son d'explosif existera.
- `games/testbed/assets/assets.yaml` ne déclare que les sons : les autres fichiers du testbed
  (copies de zombies) restent à enregistrer.
- Question : volumes relatifs (`hit` -9, `death`/`dash` -6, autres -3 dBFS crête) à régler à
  l'écoute ; `feedback.ron` n'a pas de volume par clé (à ajouter si besoin).

## 5. Atteignabilité

Présentation seule, aucune mécanique de simulation. Les sons jouent dans toute partie ordinaire
des quatre jeux : tir, coup au but, kill, dégâts, dash, rechargement dès la première salle ou
vague ; `blank` en gungeon (touche du blank, objet de départ) ; `door` à la première salle
nettoyée de gungeon et à toute porte achetée de zombies (porte de zombies non essayée en jeu).
