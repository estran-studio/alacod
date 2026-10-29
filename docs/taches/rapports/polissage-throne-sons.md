# polissage-throne-sons (S2-11) — rapport

Branche `polissage-throne-sons`, clone `throne`, fiche `docs/taches/polissage-throne-complet.md` point 3, spec `studio/runs/alacod-polissage/b1-s1-9-etape2-s2-11.md` § 2.
Base : `origin/polissage-integration` (418d6e9e puis, à la relecture du coordinateur, 4cea5046 : S-0b et S1-9 étape 2 y sont) ; la branche contient deux fusions de l'intégration (merge, pas de rebase). Présentation seule : aucune trace ne bouge.
**Fichiers de b2 touchés, avec l'accord du coordinateur (2026-10-29)** : `crates/content/src/feedback.rs` (clés `shot_<arme>`, `reload_<arme>`, `mutation`), `crates/content/src/lint.rs` (arme inconnue = erreur), `crates/game/src/feedback.rs` (`shot_sound`, `reload_sound`, volume de repli), `crates/game/src/feedback_sounds.rs` (`play_mutation_sound`), `crates/game/src/ui/hud.rs` (`hud_overlay_wanted` en `pub(crate)`, partagé avec le chargeur des sons). `crates/game/src/lib.rs` : une ligne permutée par `cargo fmt` (l'intégration échouait `make format`).
Même mécanique que zombies (`docs/conventions.md` § 98) : les sons publics sont dans `games/throne/assets/ui/feedback.ron` et `weapons.ron` (Snake's, Kenney), les extraits Sonniss dans `ui/feedback.local.ron` (gitignoré) + `sounds/sonniss/` (gitignoré).

## 1. Points de la spec

| Point | Issue | Preuve |
|---|---|---|
| Tableau événement × son de throne (13 armes par familles, recharge, coups, morts par famille, boss, mutation, portail, étage, coffre, ramassage, menus) | **fait**, avec 3 limites dites en § 2 | § 2 |
| Mêmes packs, même traitement (script généralisé) | **fait** | `scripts/repack-throne-sons.py` (script frère : importe `process`, `extract`, `TARGET`, `OUT` de `repack-zombies-sons.py`, qui gagne l'option `pitch`) |
| Public (Kenney, Snake's) dans `feedback.ron` / `weapons.ron`, Sonniss dans `feedback.local.ron` de throne | **fait**, armes comprises (clés `shot_<arme>`, § 2) | § 2, § 3 |
| Clés moteur `shot_<arme>` / `reload_<arme>` et `mutation` (demandées au coordinateur, réponse 2 du 2026-10-29) | **faites**, avec tests et lint | § 2 ter, `docs/conventions.md` § 98 |
| `feedback.local.ron` aussi en wasm sous `ALACOD_WEB_PRIVATE_ASSETS=1` (message du coordinateur) | **fait** (même prédicat que `hud.local.ron`) ; **non vérifié** dans un build web (aucun build wasm lancé) | `ui::hud::hud_overlay_wanted`, § 98 |
| Licences dans `games/throne/assets/assets.yaml` ; D32 vérifié pour throne | **fait** | trois entrées `*-throne` ; D32 : § 3 |
| Écoute de William (1 min, carte `--unread`) | **protocole prêt, écoute non faite** | § 4 |
| Vidéo de 60 s de throne à 2 joueurs, `ALACOD_MUTE=1` | **fait** | § 5 |
| Lint throne avec et sans surcharge, `make test_scenarios` sans `BLESS`, `make check`, `git ls-files \| grep -i sonniss` vide | voir § 6 | § 6 |

## 2. Tableau événement × son

« Public » : commité (CI, amis, web). « Surcharge » : `ui/feedback.local.ron` + `sounds/sonniss/` (poste de William, puis dépôt privé). Tous les choix sont faits **sur le nom et la durée des fichiers, sans écoute** (aucun son sur le Mac) : à juger par William.

### Armes (`audio_config` de `weapons.ron` public, un tir par arme, un rechargement par famille ; clés `shot_<arme>` de la surcharge pour énergie, lanceurs, lames)

Le son d'une arme vient, par priorité, de la clé `shot_<arme>` de `sounds` (la surcharge locale la pose, § 2 ter), puis de l'`audio_config` public, puis de la famille. Les 12 armes du joueur ont un son public ; énergie, lanceurs et lames ont **en plus** un son Sonniss dans la surcharge (colonne « Surcharge »). `arsenal` (projectiles des ennemis) : sans objet, le son d'un tir suit l'arme active du **joueur local**.

| Famille | Arme | Tir public (Snake's ; Kenney pour les lames) | Tir de la surcharge (Sonniss) | Recharge (public) |
|---|---|---|---|---|
| balles | `revolver` | 9mm Single, −3 demi-tons | — (balles : public seulement) | 44 Magnum Single Load (`reload_revolver`) |
| balles | `mitraillette` | 556 Single (un coup, jamais l'échantillon soutenu) | — | AR Reload Full |
| balles | `fusil_a_pompe` | 20 Gauge Single | — | Pump Reload Full Fast |
| balles | `canon_ricochet` | 762x39 Single | — | AK Reload Full (`reload_canon`) |
| obus | `lance_grenades` | 20 Gauge Single, −8 demi-tons | Federico Soler Effective Trailer Booms Vol 2 n° 011 | Single Shot Reload Full (`reload_lanceur`) |
| obus | `roquette` | 762x54r Single, −5 demi-tons | Booms Vol 2 n° 075 | idem |
| obus | `mortier` | 762x54r Single, −11 demi-tons | Booms Vol 2 n° 214 | idem |
| énergie | `laser` | 22LR Single, +10 demi-tons | InMotionAudio Arc « ArcDesign15 » | AR Charging Handle (`reload_energie`) |
| énergie | `plasma` | 22LR Single, −4 demi-tons | Epic Stock Media Elemental Mutation « Impact Electric Tonal Deep » | idem |
| énergie | `traqueur` | 22LR Double Tap, +4 demi-tons | Epic Stock Media Fantasy Game 2 « Collect Scifi Electronic Bass Burst Sweep Heavy 04 » | idem |
| lames | `lance_lames` | Kenney impactMetal_light_001 | David Dumais Audio Melee Weapons Pack 2 « Swing Scrape Swift Long Blade 14 » | Snake's Lever Reload (`reload_lames`) |
| lames | `disque` | Kenney impactPlate_light_003 | Epic Stock Media Tower Defense « Sword Slice Special Move Layered Swish 04 » | idem |
| ennemis | `arsenal` | — | — | — (sans objet) |

Lanceurs et énergie n'ont aucun son libre dédié dans les packs de William (Snake's n'a que des armes à feu, Kenney impact que des impacts) : leur tir **public** est un tir de fusil **changé de hauteur** (`sox pitch`), dit comme tel dans `assets.yaml` (D171) ; la surcharge locale a de vrais sons dédiés. Les fichiers Sonniss de lanceurs ont du silence au départ (jusqu'à 270 ms) : coupé (`skip`) pour que le tir parte tout de suite. Le volume d'un tir est celui de la clé de sa famille (`shot_pistol` −12 pour toute arme à tir manuel) : les fichiers de lanceurs sont donc normalisés 2 dB plus fort, ceux d'énergie 2 dB plus bas (RMS cibles du script).

### Événements

| Événement | Clé | Public (commité) | Surcharge locale (Sonniss) |
|---|---|---|---|
| coup porté, mêlée, dégâts reçus, dash, porte | `hit`, `melee`, `hurt`, `dash`, `door` | inchangés (Kenney) | idem |
| mort d'un ennemi (repli) | `death` | gémissement G. Silveira (CC BY 3.0), inchangé | idem |
| mort : rongeurs (`rat`, `roi_rat`) | `death_rat`, `death_roi_rat` | **repli : `death`** | 344 Audio Dinosaurs Vol 1 « Hatchling Calling Out » |
| mort : bêtes (`chien`, `buffle`) | `death_chien`, `death_buffle` | **repli : `death`** | 344 Audio Dinosaurs Vol 2 « Large Herbivore Roar 01 » |
| mort : humains (`pillard`, `franc_tireur`, `rodeur`, `cracheur`) | `death_<id>` | **repli : `death`** | Epic Stock Media Humanoid Creatures Vol 4 « Flutter Death Vocal » |
| mort : `brute` | `death_brute` | **repli : `death`** | Epic Stock Media Humanoid Creatures Vol 4 « Orc Male Attack Long Heavy Hit » |
| mort : machines (`tourelle`, `arroseur`, `mannequin`, `cible`) | `death_<id>` | Kenney impactMetal_heavy_003 | Epic Stock Media Tower Defense « Tower Deploy Hitech Robot Motor Dark Thump Servo Whine » |
| mort du joueur | `death_player` | Kenney impactSoft_heavy_003 | Epic Stock Media AAA British Female Detective « Pain Vocal Dying » |
| boss : apparition, phase, mort | `boss_spawn`, `boss_phase`, `boss_death` | Kenney impactMetal_heavy_001, impactMining_001, impactPlate_heavy_004 | 344 Audio « T Rex » ; Epic Halloween « Jumpscare Vocal » ; Epic Halloween « Werewolf Growl » |
| portail, étage suivant | `floor` | Kenney impactPlate_medium_003 | Cinematic Sound Design « Transition Braam Slow Dark Creepy » |
| salle nettoyée | `room_cleared` | Kenney impactBell_heavy_004 | CSD « Interface Plucks Happy » |
| coffre, objet ramassé, power-up | `pickup` | Kenney impactGlass_light_002 | CSD « Game Entry Happy Short » |
| achat | `purchase` | Kenney impactMetal_light_003 | CSD « Ting Coins » |
| explosion (bombe, baril, champion explosif) | `explosion` | Snake's 762x54r Single, −12 demi-tons | Epic Stock Media Anime Game « Explosion Small Blast Enemy Death » |
| mèche de bombe | `bomb` | Kenney impactTin_medium_001 | CSD « Interface Pop High Short » |
| blank | `blank` | Kenney impactGlass_heavy_000 | InMotionAudio Arc « ArcPowerUpDesign04 » |
| menus : naviguer, confirmer, retour | `menu_*` | Kenney (mêmes fichiers qu'à zombies) | CSD (mêmes fichiers qu'à zombies) |
| choix de mutation pris | `mutation` (nouvelle, § 2 ter) | Kenney impactBell_heavy_002 | Epic Stock Media Fantasy Game 2 « Magic Light Spell Enchantment Potion Effect Tonal Bright 03 » |
| vagues, fenêtres | `wave_*`, `window_*` | — | — : sans objet à throne |

Deux limites, sans case vide mais à dire :
- **Boss** : `roi_rat` n'a pas de bloc `boss:` dans son `.ron` (seulement `tags: ["champion"]`) : aucun `BossState`, donc `boss_spawn` / `boss_phase` / `boss_death` **ne partent pas** à throne (observé : § 2 bis ; décision du coordinateur : on n'y touche pas, D170). Les clés et fichiers sont posés, prêts le jour où un boss à phases existe ; la mort du `roi_rat` joue `death_roi_rat`. Apparition et phase : « vide » justifié.
- **Tirs publics de lanceurs et d'énergie** : tirs de fusil changés de hauteur, sans écoute (D171).

### 2 ter. Clés moteur ajoutées (avec l'accord du coordinateur)

- `shot_<arme>` / `reload_<arme>` (`content::feedback::weapon_sound_key`, `weapon_of_sound_key`) : acceptées par `SOUND_KEYS` comme `death_<id>` ; lint : arme à distance inconnue = `BrokenReference` (fixture `feedback_weapon_key_unknown`) ; une clé qui coïncide avec une clé fixe (`shot_pistol`…) reste celle de la famille. Résolution (`game::feedback::shot_sound`, `reload_sound`) : clé propre, puis `audio_config`, puis famille, puis `shot` / `reload`. Volume : celui de la clé propre si `volumes` la règle, sinon celui de la famille. Tests : `shot_sound_tests` (3), `content::feedback::tests::cles_propres_a_une_arme`.
- `mutation` : `play_mutation_sound` (`Update`, dérivé de `Mutations`, un son par image quand la liste d'un joueur local s'allonge ; première image et retour en arrière : rien). Test : `mutation_prise_seulement_quand_la_liste_s_allonge`. Aucune écriture dans la simulation, rien au rollback.

### 2 bis. Observé en jeu (journal `feedback fN sound <clé> <chemin>`, `play_scenario --log`, `ALACOD_MUTE=1`)

Scénarios `weapon_<arme>` (`tests/scenarios/generated/throne/`, jouée jusqu'à la dernière frame), journal du tir et de la recharge **de chaque arme**, une fois sans la surcharge, une fois avec :

| Arme | Public (sans `feedback.local.ron`) | Surcharge locale |
|---|---|---|
| `revolver` (6 tirs) | `shot_pistol sounds/shot_revolver.ogg`, `reload … reload_revolver.ogg` | idem (balles : public seulement) |
| `lance_grenades` (24), `mortier` (11) | `shot_pistol sounds/shot_lance_grenades.ogg` / `shot_mortier.ogg`, `reload_lanceur.ogg` | `shot_lance_grenades sounds/sonniss/shot_lance_grenades.ogg` / `shot_mortier … sonniss/shot_mortier.ogg` |
| `laser` (12), `plasma` (6), `traqueur` (10) | `shot_pistol sounds/shot_laser.ogg` … `reload_energie.ogg` | `shot_laser sounds/sonniss/shot_laser.ogg` … |
| `lance_lames` (8), `disque` (9) | `shot_pistol sounds/shot_lance_lames.ogg` ; `shot_smg sounds/shot_disque.ogg` | `shot_lance_lames sounds/sonniss/shot_lance_lames.ogg`, `shot_disque … sonniss/shot_disque.ogg` |

(avant les clés `shot_<arme>`, les douze armes jouaient déjà chacune leur fichier public : `fusil_a_pompe`, `mitraillette`, `canon_ricochet`, `roquette` ont le leur aussi.) Les clés de la surcharge prennent le volume de la famille (`shot_pistol` −12) tant que `volumes` ne les règle pas.
`throne_mutation_choice` (avec la surcharge) : `f784 mutation sounds/sonniss/mutation.ogg` (l'attente `Event(kind: "mutation")` du scénario est à la frame 786) ; `throne_three_floors` : `mutation` aux frames 1421 et 3303, avec `floor` aux frames 232, 1421 et 3303.
`throne_three_floors` (2 joueurs, 5 000 frames) : 656 sons dans chaque configuration ; ceux qui changent de fichier :

| Clé (lectures) | Public | Surcharge |
|---|---|---|
| mort rat ×4, roi_rat ×1 | `death` (`sounds/death.ogg`) | `death_rat`, `death_roi_rat` → `sonniss/death_rongeur.ogg` |
| mort chien ×4, buffle ×3 | `death` | `death_chien`, `death_buffle` → `sonniss/death_bete.ogg` |
| mort pillard, franc_tireur, rodeur, cracheur (×1, ×1, ×1, ×2) | `death` | `sonniss/death_humain.ogg` |
| mort brute ×1 | `death` | `sonniss/death_brute.ogg` |
| mort arroseur ×2, tourelle ×1 | `death_arroseur`, `death_tourelle` → `sounds/death_machine.ogg` | `sonniss/death_machine.ogg` |
| `floor` ×3, `mutation` ×2, `pickup` ×5 | `sounds/floor.ogg`, `sounds/mutation.ogg`, `sounds/pickup.ogg` | `sounds/sonniss/floor.ogg`, `…/mutation.ogg`, `…/pickup.ogg` |
| `hit` ×208, `hurt` ×3, `melee` ×3, tirs ×393, recharge ×20 | identiques | identiques |

Non rencontrées : `death_player`, `boss_*`, `explosion`, `bomb`, `blank`, `room_cleared`, `purchase`, `menu_*` (protocole d'écoute, § 4).

## 3. Licences et D32

`games/throne/assets/assets.yaml` (trois entrées ajoutées avant l'import) : `sound-snake-authentic-gun-sounds-throne` (Snake's : « All sounds can be used commercially, and credit is not needed », crédit gardé), `sound-impact-kenney-throne` (Kenney, CC0, `License.txt` lu), `sound-sonniss-gdc-2026-throne` (Sonniss : royalty-free sans attribution, interdit de vendre les sons tels quels, de s'en attribuer la paternité et d'entraîner une IA ; **jamais dans le dépôt public**, `statut: surcharge locale gitignorée`).
**D32 pour throne** : `sounds/shot_pistol.ogg`, `shot_shotgun.ogg`, `shot_smg.ogg` durent 0,45 / 0,50 / 0,30 s (coups uniques, Michel Baradari CC BY 3.0), `machine-gun.ogg` est absent du dossier et n'est cité nulle part : throne n'utilise pas l'échantillon soutenu, et les nouveaux tirs d'armes sont tous des coups uniques.
Chemins pour b3 (dépôt privé `estran-studio/alacod-assets`), sous `games/throne/assets/` : `ui/feedback.local.ron` et `sounds/sonniss/*.ogg` (28 fichiers : `death_rongeur`, `death_bete`, `death_humain`, `death_brute`, `death_machine`, `death_player`, `boss_spawn`, `boss_phase`, `boss_death`, `floor`, `mutation`, `room_cleared`, `explosion`, `bomb`, `blank`, `purchase`, `pickup`, `menu_navigate`, `menu_confirm`, `menu_back`, `shot_laser`, `shot_plasma`, `shot_traqueur`, `shot_lance_grenades`, `shot_roquette`, `shot_mortier`, `shot_lance_lames`, `shot_disque`).

## 4. Volumes et protocole d'écoute (1 min)

Volumes de départ (`feedback.ron`, dB) : tirs −12 (par famille), `hit` −12, `melee` −9, `hurt` −12, `death` −6 (inchangés) ; S2-11 : morts de machines −6, `death_player` −9, `boss_*` −6, `floor` −8, `room_cleared` −9, `explosion` −6, `bomb` −10, `blank` −9, `purchase` et `pickup` −8, `menu_*` −11 ; la surcharge ajoute −6 aux neuf `death_<id>` sans repli propre. Points de départ : seule l'oreille de William valide.

**Protocole (1 min, `~/orca/workspaces/alacod/b1`, branche `polissage-throne-sons`, `export CARGO_TARGET_DIR=$HOME/orca/workspaces/alacod/.target-mac`, **sans** `ALACOD_MUTE`)** :
1. Armes (avec la surcharge : énergie, lanceurs et lames en Sonniss) : `for w in revolver fusil_a_pompe lance_grenades mortier laser plasma lance_lames disque; do env -u CI make play_scenario SCENARIO=generated/throne/weapon_$w; done` (3 à 10 s chacune, une fenêtre après l'autre).
2. Run : `env -u CI make play_scenario SCENARIO=throne_three_floors` (2 joueurs ; écouter 30 s : morts par famille, portail/étage, ramassage ; `mutation` à 24 s environ (f1421) ; Ctrl-C ensuite).
3. Comparer : `mv games/throne/assets/ui/feedback.local.ron{,.off}` (repli public), refaire 2, remettre le fichier.
4. Une ligne par famille dans `docs/taches/rapports/plan-polissage-notes.md` (§ S2-11) : « trop fort / ni trop fort ni muet / trop faible » ; dire surtout, pour les armes d'**énergie** et les **lanceurs**, si le Sonniss de la surcharge et le repli public (tirs de fusil changés de hauteur, D171) conviennent ou sont à remplacer.

## 5. Captures et vidéo de 60 s

`throne_three_floors` (2 joueurs, 3 600 premières frames, une image sur deux, 30 i/s, 60,0 s) ; `throne_quad` pour quatre joueurs ; `ALACOD_MUTE=1`, `--follow 0`. Muet : la preuve du câblage est le journal (§ 2 bis). « Repli public » écarte aussi les surcharges visuelles (`sprites.local.ron`, `hud.local.ron`).
- Repli public (**commité**) : `docs/captures/polissage-throne-sons/repli-public_2j_f{00598,01198,01798}.png`, `repli-public_4j_f{00300,01500}.png`.
- Hors dépôt : `~/orca/workspaces/alacod/captures/polissage-throne-sons/` : `repli-public_2j_60s.mp4`, `surcharge-locale_2j_60s.mp4`, `surcharge-locale_{2j,4j}_f*.png`.

## 6. Preuves

```
$ alacod lint games/throne                       (avec feedback.local.ron)
games/throne : aucune erreur (14 personnages, 13 armes, 4 armes de corps à corps, 0 vagues, 2 cartes)
$ mv ui/feedback.local.ron ui/feedback.local.ron.off && alacod lint games/throne     (sans)
games/throne : aucune erreur (14 personnages, 13 armes, 4 armes de corps à corps, 0 vagues, 2 cartes)
$ git ls-files | grep -i sonniss                 (vide)
```
```
$ make format check_forbidden check_rollback_registration check_sim_boundary test_crates test_scenarios_par lint     (EXIT=0, ALACOD_MUTE=1)
test_crates : tous « test result: ok », 0 failed ; test_scenarios_par : tranches 0/4 … 3/4 ok, sans BLESS (aucune trace n'a bougé)
lint : zombies, testbed, throne, gungeon, isaac : aucune erreur
```
`make format` échouait sur `crates/game/src/lib.rs` **avant** ce lot (ordre de `pub mod feedback_shots` / `feedback_sounds` après la fusion de S1-6 et S1-7 dans l'intégration) : une ligne permutée par `cargo fmt`, seule modification Rust de la branche. `make test_tools` : 3 échecs d'environnement sur le Mac (`test_claude_say.py`), sans rapport.
Le lint de throne vérifie aussi chaque fichier cité par un `audio_config` (12 armes) et chaque `death_<id>` (personnage connu).

## 7. Ce qui reste

| Reste | Estimation |
|---|---|
| Écoute de William (§ 4), réglage des `volumes`, remplacement éventuel d'un repli ou d'un Sonniss | 0,25 j |
| D170 : `roi_rat` sans `BossState` (`boss_*` ne partent pas) : contenu + simulation, hors S2-11 | S2-7 / S2-12 |
| D171 : un pack CC0 de sons d'énergie / lanceurs si le repli public ne passe pas | selon pack |
| b3 : reprendre `ui/feedback.local.ron` et `sounds/sonniss/` de throne dans le dépôt privé (chemins en § 3) ; vérifier dans un build web avec `ALACOD_WEB_PRIVATE_ASSETS=1` (non fait ici : aucun build wasm lancé) | 0,1 j |
