# M0 §9.8 sur `test_map` — mesure INCOMPLÈTE (44/200 graines)

**Critère : non conclu.** Le calcul n'a pas pu aller au bout : le conteneur cloud a redémarré
deux fois (19h18 et 21h19 UTC le 2026-10-05), tuant les processus `alacod-sim` en arrière-plan.

- Sha de départ : `a2486fe` ; machine : 4 cœurs, 15 Go ; carte `exemples/test_map.ldtk`.
- Commande (par lot) : `alacod-sim --game zombies --bots 4 --profiles acheteur,acheteur,acheteur,acheteur
  --map exemples/test_map.ldtk --seeds A..B --until-wave 5 --max-frames 20000 --progress --json …`
- Débit : ~10 frames simulées/s par lot (4 lots sur 4 cœurs), 800–950 s par graine.

## Résultats disponibles (`m0-200-test-map/part1/*.log`, lignes `seed N : …`)

Graines 1–10, 51–62, 101–111, 151–161 (44 graines) : **44/44 « fin objectif » (vague 5)**,
0 mort observé sur les lignes lues, aucun desync ni softlock signalé. Aucun JSON écrit
(le JSON n'est produit qu'à la fin d'un lot).

## Reste à jouer

Graines 11–50, 63–100, 112–150, 162–200 (156 graines, ~9 h sur 4 cœurs). Les `sim-*.log` du dossier
sont des lots relancés puis interrompus (sans résultat de graine).
