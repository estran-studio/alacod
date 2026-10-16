# Crédits des assets du gabarit

Copiés depuis `games/testbed` et `games/zombies` de l'engine (registre : `games/*/assets/assets.yaml`).
Un jeu créé par `alacod new` hérite de ces fichiers : **garder ce fichier** (les licences CC BY exigent
l'attribution) et le compléter pour chaque asset ajouté. Chemins relatifs à `assets/`.

## sound-shots-michel-baradari

- Fichiers : `sounds/shot_pistol.ogg`, `sounds/shot_shotgun.ogg`, `sounds/shot_smg.ogg`
- Auteur : Michel Baradari (apollo-music.de, aujourd'hui ultimate-sounds.de), publié sur OpenGameArt par qubodup
- Source : https://opengameart.org/content/chaingun-pistol-rifle-shotgun-shots (archive https://opengameart.org/sites/default/files/shots.7z, fichiers pistol.wav, shotgun.wav, cg1.wav ; page lue le 2026-10-14, lot A de docs/plan-assets.md)
- Licence : CC BY 3.0 (https://creativecommons.org/licenses/by/3.0/) — affichée sur la page (« License(s) CC-BY 3.0 ») et dans le info.txt de l'archive (« Sounds (c) by Michel Baradari apollo-music.de / Licensed under CC BY 3.0 »)
- Attribution : Sounds (c) by Michel Baradari apollo-music.de, CC BY 3.0
- Modifications : rogné (pistol.wav → shot_pistol 0,45 s ; shotgun.wav → shot_shotgun 0,50 s, avant le bruit de pompe ; cg1.wav → shot_smg 0,30 s), fondu de sortie (0,15 / 0,15 / 0,12 s), stéréo mixée en mono, 44,1 kHz, normalisé à -3 dBFS crête, ré-encodé en ogg vorbis (oggenc -q 5)

## sound-reload-bmaczero

- Fichiers : `sounds/reload.ogg`
- Auteur : Brian MacIntosh (BMacZero)
- Source : https://opengameart.org/content/gun-reload-sound-effects (fichiers clipload2.wav et clipload1.wav ; page lue le 2026-10-14, lot A de docs/plan-assets.md)
- Licence : CC0 1.0 (https://creativecommons.org/publicdomain/zero/1.0/) — affichée sur la page, avec « Optionally credit Brian MacIntosh » (crédit gardé ici)
- Modifications : assemblé (clipload2.wav, 0,12 s de silence, clipload1.wav ; 0,58 s en tout), normalisé à -3 dBFS crête, ré-encodé en ogg vorbis (oggenc -q 5)

## sound-impact-kenney

- Fichiers : `sounds/death.ogg`, `sounds/hit.ogg`, `sounds/hurt.ogg`
- Auteur : Kenney (www.kenney.nl)
- Source : https://kenney.nl/assets/impact-sounds (archive kenney_impact-sounds.zip, fichiers impactSoft_medium_000.ogg → hit, impactPunch_heavy_000.ogg → hurt, impactSoft_heavy_000.ogg → death ; page lue le 2026-10-14, lot B de docs/plan-assets.md)
- Licence : CC0 1.0 (https://creativecommons.org/publicdomain/zero/1.0/) — affichée sur la page (« License Creative Commons CC0 ») et dans le License.txt de l'archive
- Modifications : stéréo mixée en mono, 44,1 kHz, normalisé en crête (hit -9 dBFS, son fréquent ; death -6 dBFS ; hurt -3 dBFS), ré-encodé en ogg vorbis (oggenc -q 5)

## sound-rpg-audio-kenney

- Fichiers : `sounds/dash.ogg`, `sounds/door.ogg`
- Auteur : Kenney (www.kenney.nl)
- Source : https://kenney.nl/assets/rpg-audio (archive kenney_rpg-audio.zip, fichiers cloth1.ogg → dash, doorOpen_1.ogg → door ; page lue le 2026-10-14, lot B)
- Licence : CC0 1.0 (https://creativecommons.org/publicdomain/zero/1.0/) — affichée sur la page et dans le License.txt de l'archive
- Modifications : stéréo 48 kHz mixée en mono 44,1 kHz, normalisé en crête (dash -6 dBFS, door -3 dBFS), ré-encodé en ogg vorbis (oggenc -q 5)

## sound-sci-fi-kenney

- Fichiers : `sounds/blank.ogg`
- Auteur : Kenney (www.kenney.nl)
- Source : https://kenney.nl/assets/sci-fi-sounds (archive kenney_sci-fi-sounds.zip, fichier forceField_000.ogg ; page lue le 2026-10-14, lot B)
- Licence : CC0 1.0 (https://creativecommons.org/publicdomain/zero/1.0/) — affichée sur la page et dans le License.txt de l'archive
- Modifications : normalisé à -3 dBFS crête, ré-encodé en ogg vorbis (oggenc -q 5)

## basic-zombie-shooter-blackhunterdev

- Fichiers : `sprites/characters/player/Body/Body.png`, `sprites/characters/player/Hair/Hair1.png`, `sprites/characters/player/Hair/Hair2.png`, `sprites/characters/player/Hair/Hair3.png`, `sprites/characters/player/Hair/Hair4.png`, `sprites/characters/player/Moustach/Moustach1.png`, `sprites/characters/player/Moustach/Moustach2.png`, `sprites/characters/player/Moustach/Moustach3.png`, `sprites/characters/player/Moustach/Moustach4.png`, `sprites/characters/player/Shirt/Shirt1.png`, `sprites/characters/player/Shirt/Shirt2.png`, `sprites/characters/player/Shirt/Shirt3.png`, `sprites/characters/player/Shirt/Shirt4.png`, `sprites/characters/shadow/Shadow.png`, `sprites/weapons/Weapons.png`
- Auteur : blackHUNTERdev
- Source : https://blackhunterdev.itch.io/basic-zombie-shooter-game-asset
- Licence : CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)
- Modifications : aucune (fichiers identiques au pack, preuves md5)

## zombie-shooter-source-inconnue

- Fichiers : `sprites/effects/slash/Slash_strip3.png`
- Auteur : inconnu
- Source : inconnue (ajoutés après coup, absents du pack blackHUNTERdev ; aucune page source identifiée de façon fiable au 2026-10-02)
- Licence : inconnue — à identifier ou remplacer (dette de la voie V6)
- Modifications : inconnues

## sunnyland-ansimuz

- Fichiers : `maps/atlas/SunnyLand-player.png`, `maps/atlas/SunnyLand_by_Ansimuz-extended.png`
- Auteur : Ansimuz
- Source : https://ansimuz.itch.io/sunny-land-pixel-game-art
- Licence : CC0 1.0 (https://creativecommons.org/publicdomain/zero/1.0/)
- Modifications : aucune (repris des assets d'exemple de bevy_ecs_ldtk)

## pixel-poem-dungeon-tileset

- Fichiers : `maps/atlas/Dungeon_Tileset.png`
- Auteur : Pixel_Poem
- Source : https://pixel-poem.itch.io/dungeon-assetpuck
- Licence : licence propre de l'auteur (pas CC0) — « This asset pack can be used in free and commercial projects. You can modify it to suit your own needs. Credit is not necessary, but appreciated. You may not redistribute it or resell it. » L'auteur a explicitement autorisé en commentaire de la page l'hébergement des sprites dans un dépôt open source (« Yes, you can. »).
- Modifications : aucune (fichier identique au pack, preuve md5)
- Note : licence hors CC0/CC-BY — à valider par l'équipe (dette de la voie V6)

## nuclear-blaze-tileset-deepnight

- Fichiers : `maps/atlas/NuclearBlaze_by_deepnight.png`
- Auteur : Sébastien Benard (deepnight)
- Source : https://github.com/Trouv/bevy_ecs_ldtk (README, section « Asset Credits ») et https://raw.githubusercontent.com/deepnight/ldtk/master/app/extraFiles/samples/README.md
- Licence : CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/)
- Modifications : « Tileset was exported from aseprite to png, but no other modifications were made. » (README de bevy_ecs_ldtk, verbatim)
- Note : ShareAlike — toute version modifiée du tileset doit rester CC BY-SA

## caz-pixel-fantasy-icons

- Fichiers : `maps/atlas/MV Icons Complete Sheet Free - ALL.png`
- Auteur : Caz (cazwolf)
- Source : https://cazwolf.itch.io/caz-pixel-free
- Licence : CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/) — « They're CC by 4.0 » (commentaire de l'auteure sur la page ; le README de bevy_ecs_ldtk crédite « PIXEL FANTASY RPG ICONS … by Caz, CC BY 4.0 »)
- Modifications : aucune connue (repris des assets d'exemple de bevy_ecs_ldtk)

## fira-mono

- Fichiers : `fonts/FiraMono-LICENSE`, `fonts/FiraMono-Medium.ttf`
- Auteur : The Mozilla Foundation and Telefonica S.A.
- Source : https://github.com/mozilla/Fira (texte de licence embarqué dans le dépôt, fichier fonts/FiraMono-LICENSE)
- Licence : SIL Open Font License 1.1 (https://openfontlicense.org/)
- Modifications : aucune

