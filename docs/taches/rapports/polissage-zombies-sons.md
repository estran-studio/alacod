# polissage-zombies-sons (S1-9) — rapport

Branche `polissage-zombies-sons`, clone `zombies`, fiche `docs/taches/polissage-zombies-sons.md`, spec `studio/runs/alacod-polissage/b1-s1-9-zombies-sons.md`.
**Base : `polissage-zombies-visuel` f5fb100c** (S1-8 pas encore dans `main` au départ) ; rebase sur `main` après son merge. Présentation seule : aucune trace ne bouge.

## 1. État (étape 1 faite ; étape 2 attend S1-7 et deux décisions)

- **Branché maintenant** (clés qui existent déjà) : tir et recharge des **quatre armes** (une paire de fichiers par arme, `audio_config` de `weapons.ron`), `hit`, `melee`, `hurt`, `death`, `dash`, `door` inchangés.
- **Prêt, pas branché** : fenêtres (`window_break.ogg`, `window_repair.ogg`, Kenney CC0, **committés**, clés absentes de S1-7 : `window_break` et `window_repair` demandées à b2) ; créatures, mort du joueur, boss, vague, salle, achat, ramassage, menus (**Sonniss, 14 fichiers**, sur le disque sous `games/zombies/assets/sounds/sonniss/`, **gitignoré**, en attente de la décision de William, voir §3).
- **D32** : zombies n'utilise pas l'échantillon soutenu (`machine-gun.ogg` absent du dossier, aucune référence) ; les quatre armes jouent des **coups uniques** (`Single` de Snake's).

## 2. Tableau événement × son

| Événement | Clé (S1-7 : *) | Fichier | Source | État |
|---|---|---|---|---|
| tir pistolet | `audio_config` `pistol` | `shot_pistol.ogg` | Snake's 2, « 9mm Single » | **branché** |
| tir mitrailleuse | `audio_config` `machine_gun` (+ famille `shot_smg`) | `shot_smg.ogg` | Snake's 1, « 556 Single » (un coup, pas le tir soutenu) | **branché** |
| tir fusil à pompe | `audio_config` `shotgun` | `shot_shotgun.ogg` | Snake's 2, « 20 Gauge Single » | **branché** |
| tir fusil | `audio_config` `rifle` | `shot_rifle.ogg` | Snake's 2, « 308 Single » | **branché** |
| recharge pistolet / mitrailleuse / pompe / fusil | `audio_config` par arme | `reload_pistol/smg/shotgun/rifle.ogg` | Snake's 2 « 9mm Pistol Reload 1 » ; Snake's 1 « AR Reload Full », « Pump Reload Full Fast », « 308 Magazine Full » | **branché** ; `reload.ogg` (BMacZero CC0) reste le repli de la clé `reload` |
| coup porté | `hit` | `hit.ogg` | Kenney impactSoft_medium (CC0) | déjà branché |
| mêlée | `melee` | `melee.ogg` | Kenney impactPunch_medium (CC0) | déjà branché |
| dégâts reçus | `hurt` | `hurt.ogg` | Kenney impactPunch_heavy (CC0) | déjà branché |
| mort d'un ennemi (repli) | `death` | `death.ogg` | « Zombie groan » G. Silveira (CC BY 3.0) | déjà branché |
| dash | `dash` | `dash.ogg` | Kenney rpg-audio (CC0) | déjà branché |
| porte | `door` | `door.ogg` | Kenney rpg-audio (CC0) | déjà branché |
| fenêtre cassée | `window_break` * (**absente de `SOUND_KEYS`**) | `window_break.ogg` | Kenney impactPlank_medium (CC0) | fichier prêt ; **clé à ajouter** |
| fenêtre réparée | `window_repair` * (idem) | `window_repair.ogg` | Kenney impactWood_light (CC0) | fichier prêt ; **clé à ajouter** |
| mort zombie / zombie coriace / zombie brute | `death_zombie_1` / `_2` / `_full` * | `sonniss/death_zombie_1/2/full.ogg` | Sonniss : Epic Stock Media Humanoid Creatures Vol 4 « Flutter Death Vocal » ; SoundBits Vox Bestiae « Ethereal Entity Grim Pain » ; Humanoid Creatures Vol 4 « Sea Beast Pain Yell » | prêt (hors dépôt), après S1-7 |
| mort du joueur | `death_player` * | `sonniss/death_player.ogg` | Sonniss : Epic Stock Media AAA British Female Detective « Pain Vocal Dying » | idem |
| boss : apparition, mort | `boss_spawn`, `boss_death` * | `sonniss/boss_spawn.ogg`, `sonniss/boss_death.ogg` | Sonniss : 344 Audio Dinosaurs « T Rex » ; Epic Stock Media Halloween « Werewolf Growl » | idem (phase : sans objet, le boss n'a qu'une phase) |
| début, fin de vague | `wave_start`, `wave_end` * | `sonniss/wave_start.ogg`, `sonniss/wave_end.ogg` | Sonniss : Jake Fielding Cinematic Horn Braams ; Cinematic Sound Design « Button Arp Twinkle » | idem |
| salle nettoyée | `room_cleared` * | `sonniss/room_cleared.ogg` | Sonniss : CSD « Interface Plucks Happy » | idem |
| achat (porte, arme, perk) | `purchase` * | `sonniss/purchase.ogg` | Sonniss : CSD « Ting Coins » | idem |
| ramassage (power-up, arme) | `pickup` * | `sonniss/pickup.ogg` | Sonniss : CSD « Game Entry Happy Short » | idem |
| menus : naviguer, confirmer, retour | `menu_navigate`, `menu_confirm`, `menu_back` * | `sonniss/menu_*.ogg` | Sonniss : CSD « Interface Percussion Snap », « Accept Boing Crunch », « Deny Muted » | idem |
| explosion, bombe, étage, blank | — | — | — | **sans objet** à zombies |

Aucune case vide : chaque ligne a un fichier ; les deux lignes « fenêtre » attendent une clé du moteur, les lignes Sonniss attendent S1-7 **et** la décision de licence.

## 3. Licences (copiées dans `games/zombies/assets/assets.yaml`, avant l'import)

- **Snake's** (deux packs itch, F8 Studios) : « All sounds can be used commercially, and credit is not needed » (pages lues le 2026-10-28) ; « It's public domain! » dit l'auteur en commentaire du second pack ; aucun texte de licence dans les archives. Fichiers **committés**. Remplacent les tirs de Michel Baradari (CC BY 3.0), retirés.
- **Kenney impact** (`License.txt` de l'archive : CC0) : `window_break`, `window_repair` ajoutés à l'entrée existante. Fichiers committés.
- **Sonniss GDC 2026** (`License - GDC Game Audio.pdf` lu) : royalty-free, usage commercial sans attribution, modification permise ; **interdit** de vendre les sons tels quels (incorporés à un projet : permis), de s'en attribuer la paternité, et de les utiliser pour entraîner une IA. Le dépôt étant public, les 14 extraits sont **hors dépôt**
  (`games/*/assets/sounds/sonniss/` gitignoré, règle de `docs/conventions.md` § 92, `scripts/repack-zombies-sons.py` les y écrit) **jusqu'à la décision de William** : dépôt public, ou dépôt privé d'assets (lot S-0b). Le premier commit les avait poussés ; la branche a été réécrite (`--force-with-lease`) à la demande du coordinateur ; l'ancien SHA peut rester atteignable côté GitHub jusqu'à son ramassage.
- Chaque bibliothèque du bundle Sonniss n'est qu'un **échantillon de 3 à 4 fichiers** (347 en tout) : les choix ci-dessus sont ceux qui existent, pas une sélection dans un catalogue ; Leohpaz est absent. Si un son ne convient pas à l'oreille, il n'y a pas de remplaçant dans le bundle pour certains (dire à William, ne rien inventer).

Traitement (`scripts/repack-zombies-sons.py`, un fichier extrait à la fois sous `~/orca/workspaces/alacod/packs/{snake,kenney-impact,sonniss}/`, jamais un bundle entier) : rognage, fondu de sortie, mono 44,1 kHz, gain pour une **RMS cible** (tirs −22 dBFS, recharges −27, créatures −21, boss −18, vague −20, le reste −22 ; crête ≤ −1 dBFS), ogg vorbis q5.

## 4. Volumes (S7 « tirs trop forts ») et protocole d'écoute

Volumes de départ (`ui/feedback.ron`, dB, **inchangés** pour les clés existantes : les fichiers d'armes ont la même RMS cible, −22) : tirs −12, `hit` −12, `melee` −9, `hurt` −12, `reload` −12 (fichiers à −27 : ≈ 5 dB sous un tir), `dash` −12, `door` −12, `death` −6.
À poser avec S1-7 : `death_zombie_*` −6, `boss_*` −6, `death_player` −9, `wave_start` −6, `wave_end` −9, `room_cleared` −9, `purchase` −8, `pickup` −8, `menu_*` −11, `window_*` −6.

**Protocole d'écoute (2 minutes, avec William)** : deux terminaux dans `~/orca/workspaces/alacod/b1` (branche `polissage-zombies-sons`), `export CARGO_TARGET_DIR=$HOME/orca/workspaces/alacod/.target-mac`.
1. `cargo run -q -p content --bin alacod --profile headless -- sandbox games/zombies --profile lourd` : la fenêtre s'ouvre (mitrailleuse, trois zombies à droite). Volume du Mac à son niveau d'habitude.
2. Mitrailleuse : tirer 5 s (clic maintenu), recharger (R). Zombies : les tuer (coup + mort), en laisser un te toucher (dégâts reçus).
3. Second terminal : `… alacod remote give --weapon shotgun`, puis `--weapon rifle`, `--weapon pistol` (ou `alacod remote run games/zombies/scripts/ecoute_sons.alacod` pour reposer le décor) : un tir et une recharge par arme.
4. Noter une ligne par arme dans `docs/taches/rapports/plan-polissage-notes.md` (§ S1-9) : « trop fort / ni trop fort ni muet / trop faible », et la même chose pour hit, mort, hurt. Je règle les `volumes` d'après tes notes (pas besoin de recompiler : `ui/feedback.ron` se relit au `alacod remote reload` puis `restart`).
Les sons de vague, d'achat, de ramassage, de menu et de mort par famille s'écoutent **après** S1-7 et la décision Sonniss (deuxième écoute, une minute).

## 5. Captures et vidéos de 60 s

Scénario `clone_duo` (deux joueurs, 3 600 premières frames, une image sur deux, 30 i/s) ; `clone_quad` pour quatre joueurs ; `ALACOD_MUTE=1`, `--follow 0`. Muet (la vidéo n'a pas de piste son).
- Repli public (**commité**) : `docs/captures/polissage-zombies-sons/repli-public_2j_f{00598,01198,01798}.png`, `repli-public_4j_f{00700,01500}.png`.
- Hors dépôt (`~/orca/workspaces/alacod/captures/polissage-zombies-sons/`) : `repli-public_2j_60s.mp4`, `surcharge-locale_2j_60s.mp4` (60 s chacune), `surcharge-locale_2j_f*.png`, `surcharge-locale_4j_f*.png`.

## 6. Preuves

```
$ make format check_forbidden check_rollback_registration check_sim_boundary test_crates test_scenarios_par lint     (EXIT=0)
93 « test result: ok » (test_crates), 0 failed ; tranche 0/4 … 3/4 : ok ; suite en 4 tranches : 259 s (sans BLESS : aucune trace n'a bougé)
games/{zombies,testbed,throne,gungeon,isaac} : aucune erreur
```
`make test_tools` : 3 échecs d'environnement sur le Mac (`test_claude_say.py`, Python 3.9.6 du système), sans rapport. Le journal `feedback … sound` d'un scénario change de **chemin** pour la recharge (`sounds/reload.ogg` → `sounds/reload_<arme>.ogg`) et le tir du fusil (`shot_pistol.ogg` → `shot_rifle.ogg`) ; aucun test ne lit ces lignes et les traces n'y sont pas.
Sandbox lancé avec rendu et son (`alacod sandbox games/zombies --profile lourd --detach`) : aucune erreur de chargement, `feedback f20 sound melee sounds/melee.ogg` au journal ; arrêté ensuite.

## 7. Ce qui reste

| Reste | Estimation |
|---|---|
| Écoute de William sur les armes (§ 4) et réglage des `volumes` | 0,25 j |
| Décision Sonniss : public / dépôt privé / surcharge locale ; si surcharge locale, un `feedback.local.ron` (même principe que `hud.local.ron`) à ajouter au chargeur de `feedback.ron` | 0,5 j si surcharge |
| Après S1-7 : `window_break` / `window_repair` (clés à ajouter), câblage de `death_zombie_*`, `death_player`, `boss_*`, `wave_*`, `room_cleared`, `purchase`, `pickup`, `menu_*`, volumes, seconde écoute | 0,5 j |
| `LIVRÉ polissage-zombies-sons` dans `docs/taches.md` : à la fin du lot | — |
