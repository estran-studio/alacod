# m1-v3-bots-lead — anticiper la cible (correctif 2, suite de m1-v3-bots-armes)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b1, branche `m1-v3-bots-lead`
créée depuis `origin/main` une fois m1-v3-bots-armes mergée, même target). Voie V3 bots, 1 j.
Compilation après le **feu vert** d'orch, `CARGO_BUILD_JOBS=2`, un dump à la fois, sims une à la fois
tant que b0 joue `test_map`.

## Contexte

Après D51 (dispersion enfin appliquée) et m1-v3-bots-armes (choix d'arme par la config, ramassage,
tir à portée), les bots `prudent` perdent encore **54 à 62 % de leurs balles** au troisième étage de
`throne` (graines 25, 103, 131, méthode du digest). Le rapport de m1-v3-bots-armes l'attribue : à
250 px, la gerbe de la mitraillette (±19 px) couvre ~64 % d'une cible de rayon 12 ; le reste se
perd sur des **cibles mobiles** (balles à ~300 px/s, visée sur la position courante). Les bots
finissent déjà 20/20 à 2 et 4 bots sur les témoins : cette tâche vise la **qualité du tir**,
mesurée par les balles perdues, pas le taux de réussite.

## Décisions (fixées ici)

1. **Anticipation (lead)** dans `crates/bots` seulement : viser `cible + vitesse_cible × t` avec
   `t = distance / vitesse_balle` (une itération suffit ; vitesse de la balle lue dans la config de
   l'arme en main, vitesse de la cible = différence de position entre deux frames, exposée dans
   `BotView`, hors rollback comme `WeaponChoices`, remise à zéro à l'entrée en partie). Pas de lead
   sur une cible immobile ni au-delà de la portée ; plafonner `t` (par exemple 1 s) pour ne pas
   viser dans la roche : si le point anticipé n'est pas en ligne de tir (`line_of_fire`), viser la
   cible.
2. **Mesure avant/après**, même base `main`, même méthode que m1-v3-bots-armes : balles perdues au
   3e étage sur 25, 103, 131 (un dump à la fois) ; témoins 1..20 à 2 et 4 bots (étage 3, défaites,
   soft-locks, desync, frames moyennes). Critère de merge : balles perdues **en baisse nette**
   (médiane), aucun soft-lock ajouté, défaites non augmentées. Si le lead n'améliore pas la
   médiane, ne pas livrer la règle : rapport avec la mesure et la raison.
3. **Scénarios** : les 7 scénarios à bots bougeront (preuve par inputs : première frame où la visée
   diffère) ; `clone_quad`, `bots_four_mixed`, zombies et vagues : le lead s'applique aussi aux
   zombies (mêmes profils ?) — si oui, dire quelles traces bougent et pourquoi c'est juste ; si tu
   veux limiter à `Floors` pour protéger M0, dis-le et justifie. Décision par défaut : **partout**
   (le tir juste n'est pas propre à throne), avec preuve.
4. **Tests unitaires** : lead nul sur cible immobile, lead = v × d / vb sur cible en translation,
   plafond, repli sur la cible si le point anticipé est hors ligne de tir, déterminisme (deux
   suites de frames identiques → mêmes visées).
5. **Hors périmètre** : contenu, esquive, choix d'arme (déjà livré), tir des ennemis.

## Livrer

Suite complète, lint ×3, fmt, scripts, `make gen` ×3, exemples ; rapport
`docs/taches/rapports/m1-v3-bots-lead.md` (README §7) avec les deux mesures et les preuves ; purge du
target. Merger `origin/main` juste avant. `git push -u origin m1-v3-bots-lead`, puis `SendMessage` à
`orch` : `LIVRÉ m1-v3-bots-lead <sha> : <balles perdues avant → après, traces qui bougent>`.
