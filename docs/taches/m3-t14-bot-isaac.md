# m3-t14-bot-isaac — bot explorateur v3 pour `isaac` (M3, V3, K2)

Lire d'abord `docs/taches/README.md`, `docs/taches.md` §8.1, `docs/conventions.md` §24 et §47 (bots, bot
explorateur), §62, §69, §70, les rapports `docs/taches/rapports/bot-explorateur-v2.md` et ses scripts
(`campagne.sh`, `analyse.py`, `morts.py`), et le code `crates/bots/src/explorer.rs`. Branche
`m3-t14-bot-isaac` depuis `origin/main`, **après le merge de M3-T3, M3-T4 et M3-T5**. Estimation : 4 j.
Section de conventions réservée : **§79**.

## À faire

1. **Grille** : l'explorateur parcourt un étage `Grid` (portes sur les quatre côtés, boucles).
2. **Bombes** : pose une bombe devant un rocher qui bloque un chemin vers un but, et contre le mur d'une
   salle secrète probable (règle en données : voisinage d'au moins deux salles, comme T5) ; s'écarte de la
   mèche.
3. **Clés et sous** : ouvre le trésor fermé s'il a une clé, achète en boutique selon une politique en données
   (priorité aux cœurs à vie basse, sinon objet le moins cher).
4. **Diable** : politique en données (`devil: Never | IfHealthAbove(n)`) ; défaut à décider avec William au
   moment des campagnes (**Q14**).
5. **Campagnes** : `alacod-sim --game isaac` sur la séquence par défaut, 200 graines à 1 et 2 bots, 20 à 4 ;
   tableau au rapport (étage atteint, morts et tueurs, soft-locks, desyncs, salles secrètes ouvertes,
   marchés pris).
6. Scénarios `bot_isaac_floor`, `bot_isaac_secret`, `bot_isaac_two`.

## Règles

Bot seulement (`crates/bots`) : les campagnes des autres jeux ne bougent pas (rejouer `bot_explorer_*`).
Une décision du bot vient de l'état et de son flux RNG dédié, jamais de l'horloge.

## Vérification

README §4 ; campagnes au rapport `docs/taches/rapports/m3-t14-bot-isaac.md` (scripts versionnés à côté),
conventions §79.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de Docker.
Campagnes au premier plan, par lots de 10 graines poussés un par un. Voie rapide du README §6 : fusionner
`origin/main` et rejouer la suite juste avant le commit `LIVRÉ m3-t14-bot-isaac`, qui contient le rapport.
