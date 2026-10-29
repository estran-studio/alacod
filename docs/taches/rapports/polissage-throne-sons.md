# polissage-throne-sons (S2-11) — rapport

Branche `polissage-throne-sons`, clone `throne`, fiche `docs/taches/polissage-throne-complet.md` point 3, spec `studio/runs/alacod-polissage/b1-s1-9-etape2-s2-11.md` § 2.
Base : `origin/polissage-integration` (418d6e9e, S2-10 `polissage-throne-visuel` y est) **plus** la fusion de `polissage-zombies-sons` (522ebdc5, S1-9 étape 2) : le chargeur et le lint de `ui/feedback.local.ron` (§ 98) n'étaient pas encore dans l'intégration. Présentation seule : aucune trace ne bouge.
Même mécanique que zombies (`docs/conventions.md` § 98) : les sons publics sont dans `games/throne/assets/ui/feedback.ron` et `weapons.ron` (Snake's, Kenney), les extraits Sonniss dans `ui/feedback.local.ron` (gitignoré) + `sounds/sonniss/` (gitignoré).

## 1. Points de la spec

| Point | Issue | Preuve |
|---|---|---|
| Tableau événement × son de throne (13 armes par familles, recharge, coups, morts par famille, boss, mutation, portail, étage, coffre, ramassage, menus) | **fait**, avec 3 limites dites en § 2 | § 2 |
| Mêmes packs, même traitement (script généralisé) | **fait** | `scripts/repack-throne-sons.py` (script frère : importe `process`, `extract`, `TARGET`, `OUT` de `repack-zombies-sons.py`, qui gagne l'option `pitch`) |
| Public (Kenney, Snake's) dans `feedback.ron` / `weapons.ron`, Sonniss dans `feedback.local.ron` de throne | **fait** (armes : public seulement, voir D170) | § 2, § 3 |
| Licences dans `games/throne/assets/assets.yaml` ; D32 vérifié pour throne | **fait** | trois entrées `*-throne` ; D32 : § 3 |
| Écoute de William (1 min, carte `--unread`) | **protocole prêt, écoute non faite** | § 4 |
| Vidéo de 60 s de throne à 2 joueurs, `ALACOD_MUTE=1` | **fait** | § 5 |
| Lint throne avec et sans surcharge, `make test_scenarios` sans `BLESS`, `make check`, `git ls-files \| grep -i sonniss` vide | voir § 6 | § 6 |

## 2. Tableau événement × son

« Public » : commité (CI, amis, web). « Surcharge » : `ui/feedback.local.ron` + `sounds/sonniss/` (poste de William, puis dépôt privé). Tous les choix sont faits **sur le nom et la durée des fichiers, sans écoute** (aucun son sur le Mac) : à juger par William.

### Armes (`audio_config` de `weapons.ron`, un tir par arme, un rechargement par famille)

Le son d'une arme **ne passe pas** par la surcharge : `audio_config` est dans `weapons.ron`, public ; la surcharge ne règle que `sounds` et `volumes`. Les 12 armes du joueur ont donc un son public seulement (colonne surcharge : « sans objet », D170). `arsenal` (projectiles des ennemis) : sans objet, le son d'un tir suit l'arme active du **joueur local**.

| Famille | Arme | Tir (Snake's ; Kenney pour les lames) | Recharge |
|---|---|---|---|
| balles | `revolver` | 9mm Single, −3 demi-tons | 44 Magnum Single Load (`reload_revolver`) |
| balles | `mitraillette` | 556 Single (un coup, jamais l'échantillon soutenu) | AR Reload Full |
| balles | `fusil_a_pompe` | 20 Gauge Single | Pump Reload Full Fast |
| balles | `canon_ricochet` | 762x39 Single | AK Reload Full (`reload_canon`) |
| obus | `lance_grenades` | 20 Gauge Single, −8 demi-tons | Single Shot Reload Full (`reload_lanceur`) |
| obus | `roquette` | 762x54r Single, −5 demi-tons | idem |
| obus | `mortier` | 762x54r Single, −11 demi-tons | idem |
| énergie | `laser` | 22LR Single, +10 demi-tons | AR Charging Handle (`reload_energie`) |
| énergie | `plasma` | 22LR Single, −4 demi-tons | idem |
| énergie | `traqueur` | 22LR Double Tap, +4 demi-tons | idem |
| lames | `lance_lames` | Kenney impactMetal_light_001 | Snake's Lever Reload (`reload_lames`) |
| lames | `disque` | Kenney impactPlate_light_003 | idem |
| ennemis | `arsenal` | — | — (sans objet) |

Lanceurs et énergie n'ont aucun son libre dédié dans les packs de William (Snake's n'a que des armes à feu, Kenney impact que des impacts) : leur tir est un tir de fusil **changé de hauteur** (`sox pitch`), dit comme tel dans `assets.yaml`. Le volume d'un tir est celui de la clé de sa famille (`shot_pistol` −12 pour toute arme à tir manuel) : les fichiers de lanceurs sont donc normalisés 2 dB plus fort, ceux d'énergie 2 dB plus bas (RMS cibles du script).

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
| **mutation** (choix de mutation) | — | — | — : **clé manquante** (aucune clé de `SOUND_KEYS` ne la porte) ; demandée au coordinateur pour b2 (D170) |
| vagues, fenêtres | `wave_*`, `window_*` | — | — : sans objet à throne |

Trois limites, sans case vide mais à dire :
- **Mutation** : pas de clé (message `status` du 2026-10-29), rien ne sonne ; seuls les sons de menu s'appliquent si l'écran de choix passe par `ScreensState` (non vérifié).
- **Boss** : `roi_rat` n'a pas de bloc `boss:` dans son `.ron` (seulement `tags: ["champion"]`) : aucun `BossState`, donc `boss_spawn` / `boss_phase` / `boss_death` **ne partent pas** à throne aujourd'hui (observé : § 2 bis). Les clés et fichiers sont posés, prêts le jour où un boss à phases existe ; la mort du `roi_rat` joue `death_roi_rat`.
- **Armes sur la surcharge** : voir plus haut (D170).

### 2 bis. Observé en jeu (journal `feedback fN sound <clé> <chemin>`, `play_scenario --log`, `ALACOD_MUTE=1`)

Les douze scénarios `weapon_<arme>` (`tests/scenarios/generated/throne/`, jouée jusqu'à la dernière frame) jouent chacun le tir et la recharge **de leur arme** (par exemple `weapon_mortier` : `shot_pistol sounds/shot_mortier.ogg` ×11, `reload … reload_lanceur.ogg` ×2 ; `weapon_disque` : `shot_smg sounds/shot_disque.ogg` ×9, `weapon_fusil_a_pompe` : `shot_shotgun sounds/shot_fusil_a_pompe.ogg` ×3). `plasma` et `disque` n'ont pas rechargé dans leurs 200 frames.
`throne_three_floors` (2 joueurs, 5 000 frames) : 656 sons dans chaque configuration ; ceux qui changent de fichier :

| Clé (lectures) | Public | Surcharge |
|---|---|---|
| mort rat ×4, roi_rat ×1 | `death` (`sounds/death.ogg`) | `death_rat`, `death_roi_rat` → `sonniss/death_rongeur.ogg` |
| mort chien ×4, buffle ×3 | `death` | `death_chien`, `death_buffle` → `sonniss/death_bete.ogg` |
| mort pillard, franc_tireur, rodeur, cracheur (×1, ×1, ×1, ×2) | `death` | `sonniss/death_humain.ogg` |
| mort brute ×1 | `death` | `sonniss/death_brute.ogg` |
| mort arroseur ×2, tourelle ×1 | `death_arroseur`, `death_tourelle` → `sounds/death_machine.ogg` | `sonniss/death_machine.ogg` |
| `floor` ×3, `pickup` ×5 | `sounds/floor.ogg`, `sounds/pickup.ogg` | `sounds/sonniss/floor.ogg`, `sounds/sonniss/pickup.ogg` |
| `hit` ×208, `hurt` ×3, `melee` ×3, tirs ×393, recharge ×20 | identiques | identiques |

Non rencontrées : `death_player`, `boss_*`, `explosion`, `bomb`, `blank`, `room_cleared`, `purchase`, `menu_*` (protocole d'écoute, § 4).

## 3. Licences et D32

`games/throne/assets/assets.yaml` (trois entrées ajoutées avant l'import) : `sound-snake-authentic-gun-sounds-throne` (Snake's : « All sounds can be used commercially, and credit is not needed », crédit gardé), `sound-impact-kenney-throne` (Kenney, CC0, `License.txt` lu), `sound-sonniss-gdc-2026-throne` (Sonniss : royalty-free sans attribution, interdit de vendre les sons tels quels, de s'en attribuer la paternité et d'entraîner une IA ; **jamais dans le dépôt public**, `statut: surcharge locale gitignorée`).
**D32 pour throne** : `sounds/shot_pistol.ogg`, `shot_shotgun.ogg`, `shot_smg.ogg` durent 0,45 / 0,50 / 0,30 s (coups uniques, Michel Baradari CC BY 3.0), `machine-gun.ogg` est absent du dossier et n'est cité nulle part : throne n'utilise pas l'échantillon soutenu, et les nouveaux tirs d'armes sont tous des coups uniques.
Chemins pour b3 (dépôt privé `estran-studio/alacod-assets`), sous `games/throne/assets/` : `ui/feedback.local.ron` et `sounds/sonniss/*.ogg` (19 fichiers : `death_rongeur`, `death_bete`, `death_humain`, `death_brute`, `death_machine`, `death_player`, `boss_spawn`, `boss_phase`, `boss_death`, `floor`, `room_cleared`, `explosion`, `bomb`, `blank`, `purchase`, `pickup`, `menu_navigate`, `menu_confirm`, `menu_back`).

## 4. Volumes et protocole d'écoute (1 min)

Volumes de départ (`feedback.ron`, dB) : tirs −12 (par famille), `hit` −12, `melee` −9, `hurt` −12, `death` −6 (inchangés) ; S2-11 : morts de machines −6, `death_player` −9, `boss_*` −6, `floor` −8, `room_cleared` −9, `explosion` −6, `bomb` −10, `blank` −9, `purchase` et `pickup` −8, `menu_*` −11 ; la surcharge ajoute −6 aux dix `death_<id>` sans repli propre. Points de départ : seule l'oreille de William valide.

**Protocole (1 min, `~/orca/workspaces/alacod/b1`, branche `polissage-throne-sons`, `export CARGO_TARGET_DIR=$HOME/orca/workspaces/alacod/.target-mac`, **sans** `ALACOD_MUTE`)** :
1. Armes : `for w in revolver fusil_a_pompe lance_grenades mortier laser plasma lance_lames disque; do env -u CI make play_scenario SCENARIO=generated/throne/weapon_$w; done` (3 à 10 s chacune, une fenêtre après l'autre).
2. Run : `env -u CI make play_scenario SCENARIO=throne_three_floors` (2 joueurs ; écouter 30 s : morts par famille, portail/étage, ramassage ; Ctrl-C ensuite).
3. Comparer : `mv games/throne/assets/ui/feedback.local.ron{,.off}` (repli public), refaire 2, remettre le fichier.
4. Une ligne par famille dans `docs/taches/rapports/plan-polissage-notes.md` (§ S2-11) : « trop fort / ni trop fort ni muet / trop faible » ; dire surtout si les armes **d'énergie** et **lanceurs** (tirs de fusil changés de hauteur) sont acceptables ou à remplacer.

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
| Écoute de William (§ 4), réglage des `volumes`, remplacement éventuel des tirs d'énergie et de lanceur | 0,25 j |
| D170 : clé de son par arme (`shot_<arme>`) et clé `mutation` côté moteur (b2), puis Sonniss d'armes d'énergie dans la surcharge | 0,5 j |
| b3 : reprendre `ui/feedback.local.ron` et `sounds/sonniss/` de throne dans le dépôt privé (chemins en § 3) | 0,1 j |
