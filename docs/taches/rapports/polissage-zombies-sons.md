# polissage-zombies-sons (S1-9) — rapport

Branche `polissage-zombies-sons`, clone `zombies`, fiche `docs/taches/polissage-zombies-sons.md`, specs `studio/runs/alacod-polissage/b1-s1-9-zombies-sons.md` (étape 1) et `b1-s1-9-etape2-s2-11.md` (étape 2).
Étape 1 (`0355a2f7`) : sons d'armes Snake's, fenêtres Kenney, extraits Sonniss hors dépôt. **Étape 2** (ce document, § 98 de `docs/conventions.md`) : surcharge locale `ui/feedback.local.ron`, câblage des 28 clés de S1-7, replis publics, protocole d'écoute, vidéo de 60 s.
Base : fusion de `origin/polissage-integration` (S1-7, b2 ; merge, pas de rebase). Présentation seule : aucune trace ne bouge.

## 1. Points de la fiche et de la spec

| Point | Issue | Preuve |
|---|---|---|
| `feedback.local.ron` : surcharge locale gitignorée de `feedback.ron` (`sounds`, `volumes`), chargeur en présentation | **fait** | `game::feedback::FeedbackConfigLoader` (remplace `RonAssetPlugin<FeedbackConfig>`), `content::feedback::FeedbackOverlay` ; § 98 |
| Lint avec **et** sans surcharge (clé inconnue du moteur = erreur ; fichier absent = erreur seulement si la surcharge est présente) | **fait** | § 6 : `alacod lint games/zombies` des deux façons, plus une surcharge cassée volontairement ; fixtures `feedback_local_overlay_ok` / `_bad` |
| Tests unitaires de la fusion | **fait** | `content::feedback::tests::surcharge_locale_*` (2), `lint_fixtures` (2) |
| Câblage `window_*` (Kenney, public) | **fait** | `feedback.ron`, `window_break` observé en jeu (§ 5) |
| Câblage `death_zombie_*`, `death_player`, `boss_*`, `wave_*`, `room_cleared`, `purchase`, `pickup`, `menu_*` (Sonniss) dans la surcharge, avec un repli public quand un fichier convient | **fait** | § 2 (deux colonnes) ; `feedback.local.ron` (hors dépôt) |
| Volumes de départ (RMS de l'étape 1) | **fait** | § 4 |
| Écoute de William (protocole, carte `--unread`) | **protocole prêt, écoute non faite** (je n'écoute rien) | § 4 |
| Captures + vidéo 60 s à 2 joueurs (+ une à 4), `ALACOD_MUTE=1` | **fait** | § 5 |
| `git ls-files \| grep -i sonniss` vide | **fait** | § 6 |

D32 : zombies n'utilise pas l'échantillon soutenu (`machine-gun.ogg` absent du dossier, aucune référence) ; les quatre armes jouent des **coups uniques** (`Single` de Snake's).

## 2. Tableau événement × son (public / surcharge)

« Public » : `feedback.ron` commité, ce que voient la CI, les amis et le build web. « Surcharge » : `ui/feedback.local.ron` + `sounds/sonniss/` (hors dépôt, poste de William puis dépôt privé `estran-studio/alacod-assets`).
Les replis Kenney ont été choisis **sur le nom et la durée des fichiers, sans écoute** (aucun son sur le Mac) : ils sont à juger par William et remplaçables un par un (`scripts/repack-zombies-sons.py`).

| Événement | Clé | Public (commité) | Surcharge locale (Sonniss) |
|---|---|---|---|
| tir pistolet / mitrailleuse / fusil à pompe / fusil | `audio_config` par arme (+ familles `shot_*`) | Snake's 2 « 9mm Single » ; Snake's 1 « 556 Single » ; Snake's 2 « 20 Gauge Single », « 308 Single » | idem |
| recharge (4 armes) | `audio_config` par arme, repli `reload` | Snake's `reload_{pistol,smg,shotgun,rifle}.ogg` ; `reload.ogg` (BMacZero CC0) en repli | idem |
| coup porté, mêlée, dégâts reçus | `hit`, `melee`, `hurt` | Kenney impactSoft_medium, impactPunch_medium, impactPunch_heavy (CC0) | idem |
| dash, porte | `dash`, `door` | Kenney rpg-audio (CC0) | idem |
| fenêtre cassée / réparée | `window_break`, `window_repair` | Kenney impactPlank_medium, impactWood_light (CC0) | idem (pas de Sonniss) |
| mort d'un ennemi (repli) | `death` | « Zombie groan » G. Silveira (CC BY 3.0) | idem |
| mort zombie / coriace / brute | `death_zombie_1` / `_2` / `_full` | **repli : `death`** (le gémissement) | Epic Stock Media Humanoid Creatures Vol 4 « Flutter Death Vocal » ; SoundBits Vox Bestiae « Ethereal Entity Grim Pain » ; Humanoid Creatures Vol 4 « Sea Beast Pain Yell » |
| mort du joueur | `death_player` | Kenney impactSoft_heavy_003 | Epic Stock Media AAA British Female Detective « Pain Vocal Dying » |
| boss : apparition | `boss_spawn` | Kenney impactMetal_heavy_001 | 344 Audio Dinosaurs « T Rex » |
| boss : mort | `boss_death` | Kenney impactPlate_heavy_004 | Epic Stock Media Halloween « Werewolf Growl » |
| début de vague | `wave_start` | Kenney impactBell_heavy_000 | Jake Fielding Cinematic Horn Braams |
| fin de vague | `wave_end` | Kenney impactBell_heavy_003 | CSD « Button Arp Twinkle » |
| salle nettoyée | `room_cleared` | Kenney impactBell_heavy_004 | CSD « Interface Plucks Happy » (jamais jouée à zombies : aucune salle à nettoyer dans `clone_duo`) |
| achat (porte, arme, perk) | `purchase` | Kenney impactMetal_light_003 | CSD « Ting Coins » |
| ramassage (power-up, arme) | `pickup` | Kenney impactGlass_light_002 | CSD « Game Entry Happy Short » |
| menus : naviguer / confirmer / retour | `menu_navigate` / `menu_confirm` / `menu_back` | Kenney impactGeneric_light_001 / impactTin_medium_003 / impactPlate_light_002 | CSD « Interface Percussion Snap » / « Accept Boing Crunch » / « Deny Muted » |
| étage, blank, explosion, bombe, phase de boss | `floor`, `blank`, `explosion`, `bomb`, `boss_phase` | — | — : **sans objet** à zombies (pas d'étage, de blank ni de bombe ; le boss n'a qu'une phase) |

Aucune case vide. **Observé en jeu** (`clone_duo`, 3 600 frames, journal `feedback fN sound <clé> <chemin>`, `ALACOD_MUTE=1`, `play_scenario --log`) : 181 sons joués dans chaque configuration ; les clés qui changent de fichier :

| Clé (nombre de lectures) | Public | Surcharge |
|---|---|---|
| `wave_start` (4), `wave_end` (3), `purchase` (2), `pickup` (2) | `sounds/<clé>.ogg` | `sounds/sonniss/<clé>.ogg` |
| mort d'un zombie (8) | `death` ×8 (`sounds/death.ogg`) | `death_zombie_full` ×7, `death_zombie_1` ×1 |
| `window_break` (1), `hit` (28), `melee` (35), `hurt` (5), tirs (83), recharge (10) | identiques | identiques |

Non rencontrées dans ce scénario : `death_player`, `boss_*`, `menu_*`, `room_cleared`, `window_repair` (écoute dédiée, § 4).

## 3. Licences (copiées dans `games/zombies/assets/assets.yaml`, avant l'import)

- **Snake's** (deux packs itch, F8 Studios) : « All sounds can be used commercially, and credit is not needed » (pages lues le 2026-10-28) ; « It's public domain! » dit l'auteur en commentaire du second pack ; aucun texte de licence dans les archives. Fichiers **committés**. Remplacent les tirs de Michel Baradari (CC BY 3.0), retirés.
- **Kenney impact** (`License.txt` de l'archive : CC0) : `window_break`, `window_repair` et, étape 2, les **onze replis publics** (`death_player`, `wave_*`, `room_cleared`, `boss_*`, `purchase`, `pickup`, `menu_*`) ajoutés à l'entrée `sound-impact-kenney`. Fichiers committés.
- **Sonniss GDC 2026** (`License - GDC Game Audio.pdf` lu) : royalty-free, usage commercial sans attribution, modification permise ; **interdit** de vendre les sons tels quels (incorporés à un projet : permis), de s'en attribuer la paternité, et de les utiliser pour entraîner une IA. **Décision de William (2026-10-29)** : jamais dans le dépôt public ; ils vont au dépôt privé `estran-studio/alacod-assets` (lot S-0b de b3) avec les images payantes. Les 14 extraits restent sous `games/zombies/assets/sounds/sonniss/` (gitignoré), branchés par `feedback.local.ron`. L'ancien SHA poussé à l'étape 1 (branche réécrite) peut rester atteignable côté GitHub jusqu'à son ramassage.
- Chaque bibliothèque du bundle Sonniss n'est qu'un **échantillon de 3 à 4 fichiers** (347 en tout) : les choix sont ceux qui existent ; Leohpaz est absent. Si un son ne convient pas à l'oreille, il n'y a pas de remplaçant dans le bundle pour certains (à dire à William, ne rien inventer).

Chemins pour b3 (dépôt privé, à reprendre tels quels, sous `games/zombies/assets/`) : `ui/feedback.local.ron` et `sounds/sonniss/*.ogg` (14 fichiers : `death_zombie_1`, `death_zombie_2`, `death_zombie_full`, `death_player`, `boss_spawn`, `boss_death`, `wave_start`, `wave_end`, `room_cleared`, `purchase`, `pickup`, `menu_navigate`, `menu_confirm`, `menu_back`).

Traitement (`scripts/repack-zombies-sons.py`, un fichier extrait à la fois sous `~/orca/workspaces/alacod/packs/{snake,kenney-impact,sonniss}/`, jamais un bundle entier) : rognage, fondu de sortie, mono 44,1 kHz, gain pour une **RMS cible** (tirs −22 dBFS, recharges −27, créatures −21, boss −18, vague −20, le reste −22 ; crête ≤ −1 dBFS), ogg vorbis q5.

## 4. Volumes et protocole d'écoute

Volumes (`ui/feedback.ron`, dB ; la surcharge ne pose que `death_zombie_*`, absents du fichier public) : tirs −12, `hit` −12, `melee` −9, `hurt` −12, `reload` −12, `dash` −12, `door` −12, `death` −6 (inchangés) ; étape 2 : `death_zombie_*` −6, `boss_*` −6, `death_player` −9, `wave_start` −6, `wave_end` −9, `room_cleared` −9, `purchase` −8, `pickup` −8, `menu_*` −11, `window_*` −6. Les replis publics et les extraits Sonniss ont les mêmes RMS cibles, donc les mêmes volumes.
Les valeurs sont des **points de départ** : seule l'oreille de William les valide (« ni trop fort ni muet »). Le jeu natif recharge `ui/feedback.ron` et `ui/feedback.local.ron` à chaud.

**Protocole d'écoute n° 1 : armes (2 min, étape 1 inchangée)** — deux terminaux dans `~/orca/workspaces/alacod/b1` (branche `polissage-zombies-sons`), `export CARGO_TARGET_DIR=$HOME/orca/workspaces/alacod/.target-mac`, **sans** `ALACOD_MUTE` :
1. `cargo run -q -p content --bin alacod --profile headless -- sandbox games/zombies --profile lourd` : mitrailleuse, trois zombies à droite.
2. Mitrailleuse : tirer 5 s (clic maintenu), recharger (R) ; tuer les zombies ; en laisser un te toucher. Second terminal : `… alacod remote give --weapon shotgun`, `rifle`, `pistol` : un tir et une recharge par arme (ou `… alacod remote run games/zombies/scripts/ecoute_sons.alacod` pour reposer le décor).

**Protocole d'écoute n° 2 : événements, avec la surcharge puis sans (3 min)** :
1. `env -u CI make play_scenario SCENARIO=clone_duo` (même `CARGO_TARGET_DIR`, **sans** `ALACOD_MUTE` ; 78 s, Ctrl-C pour arrêter). Écouter aux secondes : **0** début de vague · **9** achat · **13** fenêtre cassée, puis ramassage · **16** fin de vague et mort d'une brute · **54** mort d'un zombie.
2. `env -u CI make play_scenario SCENARIO=downed_all_lose` : joueur à terre puis mort (`death_player`), écran de défaite.
3. Boss et menus : bac à sable (étape 1, point 1), puis `… alacod remote run games/zombies/scripts/ecoute_sons_boss.alacod` (apparition du boss ; le tuer : `boss_death`) ; Échap, ↑ ↓, Entrée, Retour dans la pause (`menu_navigate`, `menu_confirm`, `menu_back`).
4. Comparer : `mv games/zombies/assets/ui/feedback.local.ron{,.off}` (repli public Kenney), refaire 1, remettre le fichier.
5. Une ligne par son dans `docs/taches/rapports/plan-polissage-notes.md` (§ S1-9) : « trop fort / ni trop fort ni muet / trop faible », et si le repli Kenney est acceptable ou à remplacer. Je règle les `volumes` d'après les notes.

## 5. Captures et vidéos de 60 s

Scénario `clone_duo` (deux joueurs, 3 600 premières frames, une image sur deux, 30 i/s, 60,0 s) ; `clone_quad` pour quatre joueurs ; `ALACOD_MUTE=1`, `--follow 0`. Muet : la vidéo n'a pas de piste son, la preuve du câblage est le journal de `play_scenario --log` (§ 2). Les deux jeux de captures sont rendus avec le **même** code ; « repli public » écarte aussi les surcharges visuelles (`sprites.local.ron`, `hud.local.ron`, pack payant), seules images publiables.
- Repli public (**commité**) : `docs/captures/polissage-zombies-sons/repli-public_2j_f{00598,01198,01798}.png`, `repli-public_4j_f{00700,01500}.png`.
- Hors dépôt (`~/orca/workspaces/alacod/captures/polissage-zombies-sons/`) : `repli-public_2j_60s.mp4`, `surcharge-locale_2j_60s.mp4`, `surcharge-locale_2j_f*.png`, `surcharge-locale_4j_f*.png`.
- Moments clés identiques dans les deux rendus (230 événements à 2 joueurs, 52 à 4) : la surcharge ne change que le choix des fichiers sonores.

## 6. Preuves

```
$ alacod lint games/zombies                      (avec feedback.local.ron)
games/zombies : aucune erreur (6 personnages, 4 armes, 6 armes de corps à corps, 1 vagues, 6 cartes)
$ mv ui/feedback.local.ron ui/feedback.local.ron.off && alacod lint games/zombies     (sans)
games/zombies : aucune erreur (6 personnages, 4 armes, 6 armes de corps à corps, 1 vagues, 6 cartes)
$ surcharge cassée (menu_back → sounds/sonniss/absent.ogg, clé « vague »)
ui/feedback.local.ron: feedback : champ sounds.menu_back = « sounds/sonniss/absent.ogg » : fichier absent de assets/
ui/feedback.local.ron: feedback : champ sounds.vague = clé de son inconnue du moteur (voir SOUND_KEYS)
games/zombies : 2 erreur(s) de contenu
$ git ls-files | grep -i sonniss                 (vide)
```
Tests : `cargo test -p content --lib feedback` (10 dont 2 de fusion), `cargo test -p content --test lint_fixtures feedback` (7 dont 2 de la surcharge).
```
$ make format check_forbidden check_rollback_registration check_sim_boundary test_crates test_scenarios_par lint     (EXIT=0, ALACOD_MUTE=1)
test_crates : tous « test result: ok », 0 failed ; test_scenarios_par : tranches 0/4 … 3/4 ok, 608 s, sans BLESS (aucune trace n'a bougé)
lint : zombies, testbed, throne, gungeon, isaac : aucune erreur
```
`make test_tools` : 3 échecs d'environnement sur le Mac (`test_claude_say.py` : `test_reads_last_assistant_and_cleans_markdown`, `test_say_ignores_unlisted_voice`, `test_say_uses_system_voice_when_name_absent`), sans rapport avec ce lot ; le reste de `make check` est vert. Le journal `feedback … sound` d'un scénario porte le chemin **fusionné** (Sonniss sur un poste qui a la surcharge) ; aucun test ne le lit et les traces n'y sont pas.

## 7. Ce qui reste

| Reste | Estimation |
|---|---|
| Écoute de William (protocoles n° 1 et 2 de § 4) et réglage des `volumes`, remplacement éventuel d'un repli Kenney | 0,25 j |
| b3 : reprendre `ui/feedback.local.ron` et `sounds/sonniss/` dans le dépôt privé (chemins en § 3) | 0,1 j |
| Écrire `docs/taches/rapports/plan-polissage-notes.md` § S1-9 d'après l'écoute | — |
