# m2-full-run — note de reprise (WIP)

Aucun code écrit : seul le plan est prêt. Branche partie de main 85b8b2dd.

## Plan
1. `crates/scenario/src/events.rs` ne détecte que `feedback`, `item_pickup`, `blank`, `room`, `explosion`, `boss_phase` : ajouter les événements coffre ouvert (`ChestOpened`), achat en boutique (log `shop_buy` / `CurrencyEvent` reason `shop`), arène tenue, portail, déblocage, entrée depuis le hub, pour que chaque mécanique de la fiche ait un moment clé.
2. Scénario `gungeon_full_run` : bot `explorateur`, `etages_deux`, 2 joueurs. Le bot ne fait ni coffres ni achats par défaut : poser `bots::explorer::ExplorerOptions { shopping: true }` (option ajoutée par m2-armes-sources, `alacod-sim --shopping`) ; il faudrait un champ de scénario pour l'activer sans code. Le bot meurt tôt sur la plupart des graines d'`etage` (mesuré : 8/8 graines mortes avant f1000 en `etages_grammaire`) : chercher une graine qui traverse deux étages, ou améliorer la survie du bot d'abord.
3. Variante à 4 joueurs (synctest) ; `ALACOD_FUZZ=0:8` sur les deux.
4. `bench_floor` sur la grammaire `grand` (15 à 20 salles) : plancher dans `tests/budgets.ron`, mesuré au calme (machine partagée : le dire si non fait).
5. Commentaire `// À regarder :`, vidéo en rendu logiciel, `events.json`, preuve §5 par dumps pour toute trace qui bouge.

## Outils utiles
`room_probe` (test ignoré de `crates/scenario/tests/scenarios.rs`, `ALACOD_ROOM_PROBE=<scénario>:<frame>`) liste salles, présentoirs et coffres d'un scénario. `alacod-sim --game gungeon --bots 1 --profiles explorateur --floors etages_deux --until-floor 1 --shopping --log`.
