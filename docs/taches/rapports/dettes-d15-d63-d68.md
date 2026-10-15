# dettes-d15-d63-d68 — travail en cours

- Branche créée depuis `origin/main` `77c145b7`.
- Fiche, README, CLAUDE et rapports de référence lus.
- D15 terminée : candidats de collision en listes, mêmes reconstructions et tri par net_id ; grille de séparation des ennemis conservée (usage indépendant). `make test_scenarios` : 330 scénarios, zéro différence, sans bless (214,38 s).
- Mesures D15 : 3 passes alternées, médianes avant/après : balles 474,6/524,2 fps ; horde 226,5/246,7 ; salle 226,6/280,5. Temps −9,5 %, −8,2 %, −19,2 %. Données `dettes-d15-d63-d68/bench.json`. Premier essai contendu par fin de suite écarté, deuxième essai retenu. Bench strict au calme non garanti (machine partagée).
- D63 terminée : `zombie_scaled` test_only ; armes des Shoot et SpawnPattern des ennemis atteints (phases, entrées, frises, effets). Fixture atteinte / inatteinte / test_only : 124 tests lint verts. Lint des quatre jeux sans erreur, avertissements restants dans `dettes-d15-d63-d68/lint.txt`.
- D68 : déplacement à terre autorisé par le jeu (`downed_speed_mult`) ; diagnostic de la graine 90 à faire.
- Aucun bless, aucune trace modifiée.
- Vérification finale et fusion avec `origin/main` à faire avant livraison.
- p2p à rejouer par l'orchestrateur conformément à la fiche (pas de Docker ici).
