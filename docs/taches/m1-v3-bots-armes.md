# m1-v3-bots-armes — tirer juste, changer d'arme, ramasser (correctif 2 de l'analyse des 200 graines)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b1, branche `m1-v3-bots-armes`
créée depuis `origin/main` une fois m1-v3-bots-softlocks mergée, même target). Voie V3 bots, 2 j.
Compilation après le **feu vert** d'orch, `CARGO_BUILD_JOBS=2`, « compilation finie » à chaque fin.

## Contexte

`docs/digests/m1-200-graines-throne.md` (b0) : sur les 39 défaites des 200 graines à 2 bots, toutes au
troisième étage, **57 à 80 % des balles consommées ne font aucun dégât** (médiane 74 %, avant les
correctifs de m1-v3-bots-softlocks qui ont retiré le tir vers un ennemi caché), **la mitraillette tire
dans 12/12** rejeux (revolver et lance-lames jamais : `manage_weapon` ne change d'arme qu'à sec),
armes au sol (fusil à pompe, revolver, laser, disque) ignorées dans 5/12, power-ups (munitions, rage,
vitesse) ignorés dans 9/12 ; les tireurs à distance causent la première mise à terre dans 7/12, le boss
dans 5/12. Ce qui existe dans `crates/bots` : `manage_weapon` (recharge, `switch_weapon` à sec),
`view.loot` (butin de munitions, règle « à sec » de softlocks), `line_of_fire`, `steer` (pilotage en
vitesse), `BotView` ; les scénarios `throne_defaite_{tireurs,boss,coequipier}` rejouent des **inputs
enregistrés** : ils ne bougent pas avec les bots, leurs graines (103, 25, 131) servent de témoins en
`alacod-sim`.

## Décisions (fixées ici)

1. **Mesure de départ d'abord**, sur `main` mergé (softlocks + D48), avant toute règle : graines 103,
   25, 131 et témoins 1..20 à 2 bots ; métriques : étage 3 atteint, défaites, soft-locks, et **dégâts
   infligés / balles consommées au troisième étage** (méthode du digest : trace détaillée, **un rejeu
   à la fois**, dump supprimé après). C'est la référence du rapport ; la cible se fixe à partir d'elle,
   pas à partir des 74 % d'avant softlocks.
2. **Choix d'arme par la config, jamais par le nom** : parmi les armes du porteur qui ont des
   munitions, choisir celle qui maximise les dégâts attendus sur la cible courante à sa distance
   (dégâts par tir × projectiles × cadence, nuls hors portée, à lire dans `WeaponConfig` / munitions
   §11 et §29 — expose ce qu'il faut dans `BotView`, calculé hors simulation). Hystérésis : pas de
   nouveau changement avant 120 frames, et seulement si le gain est net (≥ 1,5 ×), pour ne pas
   osciller ; un changement coûte le temps de `switch_weapon`. Pas de cas spécial « boss » : un boss
   à portée courte sélectionne naturellement l'arme à gros dégâts par tir.
3. **Ramasser, règle toujours active** (pas seulement à sec) : une arme ou un power-up au sol à
   moins de 96 px, **sans ennemi visible à moins de 150 px** (même seuil que la réanimation), est
   rejoint par la route (`steer`) ; une arme n'est ramassée que si elle est meilleure (score de la
   décision 2 à la portée moyenne) qu'une des armes portées ou si un emplacement est libre ; un
   power-up l'est toujours. Mesurer que ramasser n'ajoute pas de défaite (décision 5).
4. **Tir** : garder `line_of_fire` ; ne tirer qu'à portée de l'arme choisie (portée lue dans la
   config) ; pas d'anticipation (lead) en v1 — hors périmètre, à noter si la mesure montre que les
   balles perdues restantes viennent des cibles mobiles.
5. **Mesure avant/après** (même binaire de base `main`, même méthode) : témoins 1..20 à 2 et 4 bots +
   graines 103, 25, 131 ; critère de merge : **aucun soft-lock ajouté, défaites non augmentées**,
   balles perdues en baisse, étage 3 en hausse ou égal. Table dans le rapport. Les scénarios à bots
   bougeront (`throne_solo`, `throne_quad`, `throne_three_floors`, `throne_floor_1`,
   `throne_progression`, `bot_floors_three`, `throne_softlock_recul`) : preuve par inputs pour chacun
   (première frame d'input différent, cause : changement d'arme, ramassage, tir retiré). `clone_quad`,
   `bots_four_mixed`, zombies et vagues ne doivent pas bouger (les règles 2 et 3 ne s'activent que
   s'il y a plusieurs armes utiles ou du butin au sol ; si un scénario zombies bouge, preuve et
   justification, sinon restreindre la règle).
6. **Tests unitaires** `crates/bots` : choix d'arme (portée, munitions, hystérésis), ramassage
   (distance, menace, arme meilleure ou non, power-up), tir à portée.
7. **Hors périmètre** : contenu `throne` (D50, arroseur, difficulté : après la remesure des 200
   graines), lead, esquive, fonceur au-delà de ce que les règles partagées lui donnent.

## Livrer

Suite complète, lint des trois jeux, fmt, scripts, `make gen` ×3 sans modification, exemples ;
rapport `docs/taches/rapports/m1-v3-bots-armes.md` (README §7) avec les deux mesures et les
preuves ; purge du target. Merger `origin/main` juste avant. `git push -u origin m1-v3-bots-armes`,
puis `SendMessage` à `orch` : `LIVRÉ m1-v3-bots-armes <sha> : <avant → après, traces qui bougent>`.
Ne merge pas, ne bénis pas.
