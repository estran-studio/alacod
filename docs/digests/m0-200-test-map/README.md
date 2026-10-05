# m0-200-test-map — calcul en cours (partiel)

Sorties brutes de `alacod-sim` (4 lots parallèles, 4 cœurs, 15 Go). Les JSON ne sont écrits qu'à la
fin de chaque lot ; les lignes `seed N : …` des `.log` donnent déjà le résultat par graine.
Débit mesuré : ~10 fps simulées par lot (synctest, 4 lots sur 4 cœurs), ~800–950 s par graine,
soit ~12 h pour 50 graines par lot. Digest final à venir (`docs/digests/m0-200-graines-test-map.md`).
