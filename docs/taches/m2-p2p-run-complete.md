# m2-p2p-run-complete — une run gungeon complète en ligne (poste d'orchestration, pas le cloud)

Lire d'abord `docs/etat-engine.md` (« Contrat de sortie d'un jalon », points 3 à 5), dette **D59**
(étage de grammaire jamais joué en p2p réel) (D73, fermée, est la fausse alerte x86 / arm64), `scripts/nightly.sh`,
`scripts/check-p2p-traces.py`, `make test_multiplayer`. Estimation : 1,5 j, **par l'orchestrateur ou
William** : demande Docker (allumette locale) et deux machines. **Bloque la sortie de M2.**

## À faire

1. Deux clients (`make test_multiplayer TARGET=gungeon N=2`, inputs scriptés ou bots, pas
   `ALACOD_INPUT=neutral`) qui franchissent une transition d'étage de `etages_deux` : traces
   identiques (`check-p2p-traces.py`), nombre de frames resimulées > 0.
2. Même chose à quatre clients, puis deux machines distinctes (x86 et arm64 ; D73 est fermée, fausse alerte
   : le confirmer en réel).
3. Restart et retour au lobby en ligne après la fin de run ; déconnexion d'un pair en cours d'étage.
4. Réseau dégradé (matrice de `docs/etat-engine.md` point 4) : au moins RTT 120 ms, jitter 20 ms,
   perte 1 %.
5. Intégrer au `nightly` ; consigner commit, commandes, graines, machines, limites.

Ce n'est pas une tâche de session cloud (pas de Docker).
