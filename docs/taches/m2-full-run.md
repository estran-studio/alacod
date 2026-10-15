# m2-full-run — `gungeon_full_run` : un scénario qui atteste chaque mécanique de M2

Lire d'abord `docs/taches/README.md`, `docs/taches.md` §7 (« Vague 2 : intégration »),
`docs/taches/rapports/revue-sortie-m2.md`, `crates/scenario/src/events.rs`. Branche `m2-full-run`
depuis `origin/main`, **après** M2-T14d, M2-T12/T13 et `m2-campagne-linux-desync`. Estimation : 2 j.

## À faire

1. Scénario long `gungeon_full_run` (bot explorateur, `etages_deux`, 2 joueurs) dont les attentes
   et les moments clés attestent : salle verrouillée puis nettoyée, objet ramassé, coffre ouvert
   (clé consommée), achat en boutique, blank utilisé, arène tenue, boss et ses trois phases, portail,
   second étage ; plus un déblocage lu depuis un profil (M2-T12) et une entrée depuis le hub
   (M2-T13). Une mécanique absente de ses événements n'est pas livrée (c'est le `clone_quad` de M2).
2. Variante à 4 joueurs en synctest, et `fuzz_inputs` (`ALACOD_FUZZ=0:8`) sur les deux.
3. `bench_floor` sur la grammaire `grand` (15 à 20 salles) rempli par T14d : plancher dans
   `tests/budgets.ron`, mesuré au calme (le dire si non fait).
4. Commentaire `// À regarder :` avec les frames ; vidéo et `events.json` pour la page de revue.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Pas de GPU ni de Docker : vidéo en rendu logiciel
ou laissée à l'orchestrateur, p2p non mesuré (le dire). Pousser la branche régulièrement. Finir par
`LIVRÉ m2-full-run`, rapport `docs/taches/rapports/m2-full-run.md`.
