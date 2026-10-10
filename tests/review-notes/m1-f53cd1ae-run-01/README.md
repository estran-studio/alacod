# Première run humaine M1

William, 2026-10-10, `ced52082`, solo throne, graine 123456, 11882 frames.
Les constats et décisions sont dans [le carnet](../m1-f53cd1ae.md).

`original.ron` est une copie sans modification de l'enregistrement natif. Attention :
l'enregistreur l'étiquette `zombies` et omet les réglages throne. Le fichier original est
conservé comme preuve ; il ne faut pas le rejouer tel quel en prétendant reproduire throne.
Le journal natif complet est `/tmp/alacod-m1-ced52082-solo-01.log` ; les événements utiles
sont conservés dans `journal-extraits.log`.

Pour le diagnostic, une copie temporaire a été créée avec les mêmes inputs, la même graine
et la même map, mais `game: "throne"`, `floors: Some("run")`, `clocks: Some(["etage"])`,
`difficulty: Some(true)`, `mode: Some(Floors)`, `progression: Some("run")`.
Chemin : `/tmp/alacod-m1-ced52082-solo-01-diagnostic.ron`.
Aucune modification de code, de contenu du jeu ou de trace de référence pour ce diagnostic.

## Constats reproduits et limites

- `mutation-f720.png` : le choix de mutation masque le combat alors qu'un ennemi est vivant.
  Capture depuis une copie limitée à 1120 frames, avec `play_scenario --capture ... --every 120`.
- `navigation.log` : diagnostic `nav_stats` sur les 11882 frames de la copie temporaire.
  La sonde compte le chevauchement du sprite, pas la pénétration du collider ; son compteur
  « bloqué » utilise l'ancien `MonsterState::Chasing`, qui ne distingue pas le runtime Flee
  et les tourelles stationnaires. Il sert à trouver des candidats, sans prouver huit bugs.
- `pillard-220-f2325.log` et `.json` : sonde ponctuelle du pillard, étage 2. Le sprite
  chevauche la roche de 6 px ; le collider est juste au bord des murs. Les logs natifs
  montrent Flee après f1902. Blocage perçu au coin pendant la fuite à confronter au
  constat humain ; aucune pénétration du collider démontrée par cet instantané.
- Chutes de fluidité : constat humain conservé ; aucune mesure de rendu ni de nombre
  de balles au moment signalé. Les horodatages des logs et le FPS des captures ne sont
  pas une mesure fiable du rendu pendant la partie humaine.

La copie avec métadonnées réparées n'a pas été validée par comparaison de traces avec la
session native. La mutation est sélectionnée à f1053 dans le rejeu contre f1052 dans le
journal natif : les frames des sondes doivent être citées comme celles du rejeu.
Les attentes vertes des sondes signifient qu'elles se sont exécutées, pas que les défauts
rapportés par William sont absents.

Commandes utilisées (binaire de tests compilé sur cette session) :

```sh
ALACOD_NAV=/tmp/alacod-m1-ced52082-solo-01-diagnostic:11882 \
  cargo test -p scenario --profile headless --test scenarios nav_stats -- --ignored --nocapture
ALACOD_PROBE_FILE=/tmp/alacod-m1-ced52082-solo-01-diagnostic.ron \
  ALACOD_PROBE=220:2325 ALACOD_PROBE_JSON=/tmp/pillard-220-f2325.json \
  cargo test -p scenario --profile headless --test scenarios nav_probe -- --ignored --nocapture
```
