# m2-campagne-linux-desync — desyncs et soft-locks du bot explorateur sur `main` (Linux x86)

Lire d'abord `docs/taches/README.md`, `docs/taches/rapports/revue-sortie-m2.md` (§2.2, la mesure),
dettes D65, D68, `docs/conventions.md` §47 (bot explorateur). Branche `m2-campagne-linux-desync`
depuis `origin/main`. Estimation : 3 j. **Bloque la sortie de M2** (critère « bots sans soft-lock ni
desync »).

## Contexte

La revue de sortie a rejoué `alacod-sim --game gungeon --bots 2 --profiles explorateur,explorateur
--floors etages_deux --seeds 1..20 --until-floor 1` sur Linux x86-64 (nightly 2026-10-14) : **6
`DESYNC` (graines 2, 5, 8, 12, 17, 18), 11 soft-locks, 3 morts, 0 objectif**. Les campagnes
committées (`rapports/bot-explorateur-v2/e2_*.json`, macOS arm64) donnaient 18/20 sur les mêmes
graines, 0 desync. Le résultat est identique sur `2e87834` et sur `main` : ce n'est pas une
régression récente du contenu. Les scénarios de la suite rejouent leur trace sur ce poste (6
scénarios gungeon vérifiés), donc le poste n'est pas en cause pour la simulation scriptée.

## À faire

1. Établir la cause du **desync** (graine 2 : arène verrouillée f57, vague de deux `bullet_kin`,
   avertissements `Entity despawned` dans `LoadWorld`/`AdvanceWorld`) : `--log`, `--save-scenario`,
   `ALACOD_STATE_TRACE_FULL=1`, trouver le premier état divergent. Règle : un desync de synctest est un
   bug de déterminisme à corriger (`CLAUDE.md` §9 : `despawn_rollback`), pas à masquer.
2. Établir pourquoi, sur x86, les bots **s'arrêtent sans dégâts ni kills** (graines 3, 7, 11 : 0 PV
   perdu, 0 salle nettoyée, joueurs à la même case sur plusieurs graines, ennemis « en vue » à 25 px) :
   le bot ne tire pas, la salle n'est pas activée, ou la navigation diverge d'une plate-forme à l'autre
   ? D73 est fermée (fausse alerte, déterminisme x86/arm64 confirmé à graine fixe) : l écart avec les campagnes macOS reste donc à expliquer (toolchain nightly non épinglée, chemin des assets, build sans `dev-dylib`) ; comparer avec une exécution arm64 de la même commande.
3. Corriger (bot, règle de jeu ou déterminisme), avec preuve, puis **refaire la campagne** sur les
   deux architectures : `etages_deux` 1 et 2 bots, 200 graines, `etages_court_deux` 2 bots ; 0 desync,
   0 soft-lock. Joindre le tableau avant/après au rapport.
4. Les traces gungeon qui bougent : preuve §5 contre le `main` du moment.

## Session cloud

Tâche prévue pour une session Claude Code cloud (x86). Pas de Docker : le p2p n'est pas rejoué ici.
Pousser la branche régulièrement, campagnes par lots de 10 graines. Finir par `LIVRÉ
m2-campagne-linux-desync` et le rapport `docs/taches/rapports/m2-campagne-linux-desync.md`.
Prérequis système (cloud vierge) : `apt-get install pkg-config libasound2-dev libudev-dev
libwayland-dev libxkbcommon-dev libx11-dev` ; premier build de `alacod-sim` : ~16 min à 4 cœurs.
