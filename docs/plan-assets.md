# Plan des assets (D32) — sons, sprites, tilesets, polices sous licence libre

Établi le 2026-10-14 par la session cloud `plan-assets`. **Limite de cette session** : le réseau
sortant refuse (403, politique de l'environnement) `opengameart.org`, `freesound.org`, `kenney.nl`,
`itch.io`, `archive.org`, `pixabay.com`. Aucune page d'auteur n'a pu être consultée : **toutes les
licences de candidats ci-dessous sont « à vérifier »** (connaissance générale, pas une vérification) et
aucun asset n'a été importé. La Partie 2 (sons de tir) est donc non faite, voir le rapport.

## 1. Inventaire de l'existant

Les trois jeux (et `testbed`) partagent les mêmes fichiers (md5 identiques pour les sons) : le
registre `assets.yaml` de `throne` et `gungeon` est une copie marquée `statut: placeholder`.

### Sons (`games/<jeu>/assets/sounds/`, usage : `ui/feedback.ron` → `sounds`)

| Fichier | Jeux | Usage | Licence enregistrée | Problème |
|---|---|---|---|---|
| `shot_pistol.ogg`, `shot_shotgun.ogg`, `shot_smg.ogg` (0,30 à 0,50 s) | zombies, throne, gungeon, testbed | clés `shot_pistol`/`shot_shotgun`/`shot_smg` (famille du mode de tir actif, ou `shot_sounds` par arme) | CC BY 3.0 (Michel Baradari) | lot A (2026-10-14) ; un lance-grenades `Manual` sonne comme un pistolet sauf `shot_sounds` |
| `reload.ogg` (0,58 s) | idem | clé `reload` | CC0 (BMacZero) | lot A ; un son pour toutes les armes |
| `hit`, `hurt`, `death`, `dash`, `door` `.ogg` (0,12 à 0,92 s) | idem | clés du même nom | CC0 (Kenney) | lot B (2026-10-14) |
| `blank.ogg` (0,95 s) | gungeon, testbed | clé `blank` | CC0 (Kenney) | lot B |
| `loop.ogg` (31 s) | zombies | musique (voir `game.ron`/feedback) | CC BY 4.0 (Jay You) | seule musique ; aucune musique de salle ni de boss |

Avant le lot A : `machine-gun.ogg` (tir soutenu de 17,7 s coupé à 250 ms par `SHOT_SOUND_MAX`) et
`machine-gun-reload.ogg`, sous Pixabay Content License (hors règle) : supprimés des quatre jeux.

Clés de `sounds` connues du code : `shot_pistol`, `shot_shotgun`, `shot_smg`, `shot` (repli),
`reload`, `hit`, `hurt`, `death`, `dash`, `blank`, `door`. Rien pour coffre, pas, UI.

### Sprites et tilesets

| Famille | Fichiers | Licence enregistrée | Jeux | Problème |
|---|---|---|---|---|
| Joueur en calques (Body/Hair/Moustach/Shirt), ombre, `Weapons.png`, zombies `Zombie`/`ZombieHard`, `UI.png`, tuiles herbe/arbres/maisons | pack blackHUNTERdev | CC BY 4.0 | tous | partagé tel quel par les trois jeux ; style « survival top-down » étranger à throne/gungeon |
| `Slash_strip3.png`, `Zombie2.png` | inconnue | **inconnue** | tous | licence non identifiable → à **remplacer** (règle : rien d'ambigu) |
| `SunnyLand*.png` (Ansimuz) | exemples/atlas, `atlas/` | CC0 | tous | décor plateforme, pas top-down |
| `Dungeon_Tileset.png` (Pixel_Poem) | idem | licence propre de l'auteur (hors CC0/CC BY) | tous | **à remplacer** pour la sortie commerciale/console |
| `NuclearBlaze_by_deepnight.png` | zombies | **CC BY-SA 4.0** | zombies | copyleft : hors critère → à remplacer ou retirer |
| `MV Icons Complete Sheet Free - ALL.png` (Caz) | zombies | CC BY 4.0 | zombies | OK pour les icônes (perks, objets) |

Problèmes connus (D32 et registres) : ennemis et boss de `gungeon` (`bullet_kin`, `gatling`,
`pistolero`, boss de `rooms/boss.ron`) sans sprite ; props M2-T11 (`barrel`, `table`) sans sprite ;
les 14 ennemis de `throne` sans sprite propre (seul le zombie existe, dans zombies).

### Polices

`fonts/FiraMono-Medium.ttf` (SIL OFL 1.1, licence embarquée) : conforme, sans accents manquants
côté HUD (T2.12). Rien d'autre.

### Bilan licences

Conformes (CC0 / CC BY / OFL) : blackHUNTERdev, Ansimuz, Caz, Jay You, Fira Mono, Michel Baradari
et BMacZero (lot A), Kenney (lot B).
**Non conformes à la règle CC0/CC BY** : Pixel_Poem (licence propre), deepnight (CC BY-SA), deux
fichiers de source inconnue. À traiter dans les lots ci-dessous. Les deux sons Pixabay ont été
retirés par le lot A.

## 2. Besoins par jeu, par priorité de lecture d'une partie

1. **Sons de tir par famille** (pistolet, fusil à pompe, mitraillette, + sniper/lance-flammes si
   armes présentes) : coup unique court (< 0,5 s). Élimine `SHOT_SOUND_MAX`. *Tous les jeux.*
2. **Sprites d'ennemis et de boss** : lecture de la menace. gungeon (4 personnages + boss), throne
   (14 personnages), zombies (variantes).
3. Sons d'**impact/mort/dégât joueur**, **rechargement par famille**, **dash**, **blank**
   (gungeon/throne), **coffre/pickup**, **porte/fenêtre** (zombies).
4. **Sprites de balles, objets, coffres, props** (M2-T11 : tonneau, table).
5. **Tilesets** top-down cohérents par jeu (zombies : ville/extérieur ; throne : désert/ruines ;
   gungeon : donjon).
6. **UI/HUD** : icônes de perks/objets/power-ups (déjà Caz), cadres.
7. **Musique** : salle, boss (gungeon/throne), menu.
8. Polices : suffisant.

## 3. Contraintes de licence

Acceptées : **CC0**, **CC BY 4.0/3.0** (attribution tenue dans `assets.yaml`), OFL pour les polices.
Refusées : NC, ND, SA (copyleft sur le dérivé : à éviter pour sprites/sons embarqués), « usage
personnel », packs payants non redistribuables, Pixabay Content License (non CC, reprise
restreinte), licences propres d'auteur. Compatibilité console : CC0 et CC BY le sont (attribution
dans les crédits). Un asset dont la page d'auteur n'a pas été lue le jour de l'import n'entre pas.

## 4. Sources candidates (⚠ licences NON vérifiées dans cette session — hôtes bloqués)

Vérifier la page de l'auteur, noter l'URL et la date, avant tout import.

| Besoin | Candidat | Licence attendue | Style / remarque |
|---|---|---|---|
| Sons de tir, impact, UI, dash | Kenney « Impact Sounds », « Sci-fi Sounds », « Digital Audio », « Interface Sounds » (kenney.nl) | CC0 | propre, normalisé, `.ogg` ; peu de vrais tirs d'arme |
| Sons de tir/recharge réalistes | OpenGameArt « Gun sound pack » (ex. de Michel Baradari, « 100 CC0 SFX » de rubberduck) | CC0 / CC BY selon fichier | choisir fichier par fichier |
| Sons de tir | freesound.org, filtre licence CC0 (compte requis pour le téléchargement) | CC0 | auteurs variables ; éviter CC BY-NC |
| Musique | OpenGameArt (Alexandr Zhelanov, CC BY 3.0/4.0) ; Kenney « Music Jingles » | CC BY / CC0 | attribution obligatoire |
| Sprites joueur/ennemis top-down | Kenney « Top-down Shooter », « Tiny Dungeon », « Tiny Town » | CC0 | 16 px (Tiny) cohérent avec cases LDtk de 16 |
| Tilesets donjon | Kenney « Tiny Dungeon », « 1-Bit Pack » ; OpenGameArt « DawnLike » (DragonDePlatino, CC BY 4.0) | CC0 / CC BY | 16 px |
| Icônes | Kenney « Game Icons », Caz (déjà présent) | CC0 / CC BY | |
| Polices | Fira Mono (déjà), Kenney Fonts, Press Start 2P | OFL / CC0 | |
| Sonniss GDC bundle | licence propre (royalty-free, **pas CC**) | refusée par la règle CC0/CC BY | exclure |

## 5. Plan de remplacement en lots (chacun livrable seul)

- **Lot A — sons de tir (ferme D32) : FAIT** (branche `assets-lot-a`, 2026-10-14, rapport
  `docs/taches/rapports/assets-lot-a.md`). 3 coups uniques (pistolet, pompe, mitraillette :
  Michel Baradari, CC BY 3.0) + 1 rechargement (BMacZero, CC0), `.ogg` mono 44,1 kHz normalisés
  à -3 dBFS crête. `feedback.ron` des 4 jeux : `shot_pistol`, `shot_shotgun`, `shot_smg`,
  `reload` ; `play_shot_sound_for_new_bullets` choisit la clé d'après le mode de tir actif de
  l'arme active (`Manual` → pistolet, `Shotgun` → pompe, `Automatic`/`Burst` → mitraillette),
  repli sur `shot` ; `SHOT_SOUND_MAX`/`SHOT_SOUND_FADE`/`ShotSounds::playing` supprimés ;
  `machine-gun*.ogg` (Pixabay) retirés. Présentation seule, traces inchangées.
- **Lot B — sons d'événements : FAIT** (même branche, rapport `assets-lot-a.md`) : `hit`, `hurt`,
  `death`, `dash`, `blank`, `door` (Kenney, CC0), systèmes de présentation dans `feedback.rs` ;
  `shot_sounds` (son de tir imposé par arme). **Reste : coffre**, à brancher quand M2-T5
  (coffres) sera dans `main`.
- **Lot C — gungeon : sprites** ennemis + boss + props (Tiny Dungeon ou équivalent), atlas
  dans `sprites/`, entrées `sprite_config` des `characters/*.ron`.
- **Lot D — throne : sprites** des 14 personnages.
- **Lot E — tilesets** : remplacer Pixel_Poem (throne, gungeon) puis deepnight (zombies) ; les
  cartes LDtk référencent les atlas par chemin, à rebaliser ; contrôle par vidéos de scénarios.
- **Lot F — fichiers de source inconnue** (`Slash_strip3.png`, `Zombie2.png`) : remplacer.
- **Lot G — musique** salle/boss, **Lot H — UI/icônes**.

Chaque lot : un jeu ou une famille, `assets.yaml` à jour, `make lint` (le lint vérifie les sons
de `feedback.ron`), `make test_scenarios` sans bless, vidéo de revue.

## 6. Accès réseau requis pour la suite

Autoriser (Allowed domains) : `kenney.nl`, `opengameart.org`, `freesound.org`, `itch.io`
(et leurs CDN : `*.itch.zone`, `cdn.opengameart.org`).
