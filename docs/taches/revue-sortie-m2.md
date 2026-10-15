# revue-sortie-m2 — ce qui manque pour sortir de M2 (audit, liste de tâches)

Lire d'abord `docs/taches/README.md`, `docs/plan-engine.md` (tableau des jalons, ligne M2 ; §9.8),
`docs/taches.md` §7 (M2 en tâches) et §10 (journal), `docs/taches/dettes.md`, `docs/etat-engine.md`, et
les rapports `docs/taches/rapports/m2-*.md`. Branche `revue-sortie-m2` depuis `origin/main`.
Estimation : 1 j. **Documentation et mesures seulement, aucun code de jeu.**

## À faire

1. **Critères de sortie de M2**, un par un (plan §9.8 et ligne M2 du tableau : étages de salles typées
   et verrouillées, bullet hell, roulade, blanks, coffres, clés, boutique, objets passifs et actifs,
   synergies, boss à phases et arènes, hub et déblocages ; 1 à 4 joueurs en ligne ; scénarios verts ;
   vidéos ; contenu en RON).
   - Pour chacun : **fait / partiel / manquant**, avec la preuve (scénario, rapport, commit) ou le trou.
2. **Mesures** sur `main` :
   - `make lint GAME=gungeon` (avertissements) ;
   - une campagne `alacod-sim` à 2 bots sur la séquence par défaut de gungeon (`etages_deux`),
     20 graines ;
   - liste des scénarios `gungeon` et de ce qu'ils couvrent.
3. **Liste ordonnée des tâches restantes**, dans `docs/taches.md` §7, chacune avec :
   - une estimation ;
   - une fiche courte `docs/taches/m2-…` ;
   - sa section « Session cloud » quand elle s'y prête.

   Distinguer ce qui bloque la sortie de ce qui est du polissage (assets, équilibrage fin).
4. **Questions pour William** (validation humaine attendue : vidéos, sons, partie en ligne) : liste
   courte au rapport.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Pas de Docker : le p2p n'est pas mesuré ici, le dire.

Pousser la branche régulièrement et finir par un commit `LIVRÉ revue-sortie-m2`, avec le rapport
`docs/taches/rapports/revue-sortie-m2.md`.
