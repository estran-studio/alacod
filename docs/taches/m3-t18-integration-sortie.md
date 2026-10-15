# m3-t18-integration-sortie — intégration de `isaac` et dossier de sortie de M3 (M3, vague 2)

Lire d'abord `docs/taches/README.md`, `docs/plan-engine.md` §9.8, `docs/etat-engine.md` (contrat de sortie),
`docs/taches.md` §8.1 (atteignabilité, critères de sortie de M3), `docs/taches/dettes.md`, et les rapports
`docs/taches/rapports/m4-*.md`. Branche `m3-t18-integration-sortie` depuis `origin/main`, **après le merge de
toute la vague 1, de T11d, T14 et T15**. Estimation : 5 j (orchestration et mesures, peu de code).

## À faire

1. **Scénarios du clone** : `isaac_duo`, `isaac_quad` (synctest à 2 et 4), `isaac_full_run`,
   `isaac_devil_run`, `isaac_curse_run`, avec les moments clés listés au critère 3 du §8.1.
2. **Campagnes** : critère 4 du §8.1 sur `main` (200 graines à 1 et 2 bots, 20 à 4).
3. **Bench** : critère 6 sur une machine nommée au calme.
4. **p2p réel** : critère 7 (orchestrateur, Docker ; deux machines si D73 le permet).
5. **Vidéos** et page de revue ; **revue humaine** de William (critère 8), notes transformées en attentes ou
   en dettes ; **porte des dettes** (§4 de `docs/taches.md`) pour toutes les dettes ouvertes par M3.
6. **Dossier de sortie** : un tableau critère par critère (fait / partiel / manquant, preuve), dans le
   rapport `docs/taches/rapports/m3-t18-integration-sortie.md`.

## Règles

Une mécanique absente des moments clés de `isaac_full_run` n'est pas livrée. Un point non exécuté reste
« non vérifié ».

## Session cloud

Les points 1, 2 et 6 se prêtent à une session cloud ; les points 3 à 5 demandent `orca`, Docker et William.
Commit final `LIVRÉ m3-t18-integration-sortie` avec le rapport.
