# dettes-d15-d63-d68

Branche depuis `origin/main` `77c145b7`. Vérification finale sur fusion et campagne e2 : en cours.

## Fait

### D15 — candidats de collision en balayage

`crates/combat/src/collision_grid.rs` : listes d'AABB, sans cellules ni arbre à l'insertion ;
`query_aabb` balaie, filtre les bords inclus et trie par `net_id`. Reconstructions, cache des murs,
candidats et ordre conservés. Les noms historiques `CollisionGrids` restent ; la grille indépendante
qui sert à la séparation des ennemis (`pathing`) reste en place.

**Aucune trace bougée par D15** : `make test_scenarios` complet sans bless, 330 scénarios (214,38 s).
Test d'égalité des résultats ET de l'ordre contre l'ancienne grille : réussi (bords, coordonnées
négatives, grandes AABB, insertions inversées, vidage).

Trois passes alternées avant/après, profil headless, macOS ARM, médianes. Le premier essai,
contendu par la fin de la suite, est écarté ; tableau du deuxième essai et données dans
[`bench.json`](dettes-d15-d63-d68/bench.json). Machine partagée, pas un bench strict au calme.

| Scénario | Avant (fps) | Après (fps) | Variation du temps par frame |
|---|---:|---:|---:|
| `bench_bullets` | 474,6 | 524,2 | −9,5 % |
| `bench_horde` | 226,5 | 246,7 | −8,2 % |
| `bench_bullets_room` | 226,6 | 280,5 | −19,2 % |

Aucun ralentissement de plus de 10 % : retrait conservé. Mesures sur D15 avant le changement de
règle de salles D68 ci-dessous.

### D63 — contenu de test et armes des ennemis atteints

`zombie_scaled` (zombies) marqué `test_only: true`. `content::reach` lit `Shoot` dans les
comportements et phases des ennemis atteints, et `SpawnPattern` dans leurs effets, entrées de phase
et frises. Les ennemis inatteints ou de test ne donnent pas de source via leurs règles.
`arsenal` et `arsenal_lourd` ne sont plus signalées. Aucun effet sur la simulation.

Fixture `reach_enemy_weapons` : armes atteintes via comportement, phase, entrée et frise ; les
armes citées seulement par un ennemi inatteint ou `test_only` restent signalées. 124 tests de lint
réussis. `make lint` (les quatre jeux) : aucune erreur ; zombies 0 avertissement, throne 0,
testbed 53, gungeon 18. **Liste exacte restante** : [`lint.txt`](dettes-d15-d63-d68/lint.txt).

### D68 — réanimation, porte bloquée et campagne

**Cause de la réanimation interrompue** : le jeu autorise le déplacement à terre, avec
`downed_speed_mult` à 0,3. Le bot excluait ses buts spéciaux à terre, mais appelait toujours la
décision de `prudent` : sans ennemi, le portail restait un but. Sa surface se déplaçait donc hors de
portée du réanimateur. Une frame sans Interaction à portée supprime `Reviving`, ce qui remet à zéro
les 180 frames de progression. L'explorateur à terre envoie maintenant un input neutre ; la règle
de déplacement des joueurs humains reste inchangée.

Le contenu a évolué depuis le rapport bot-explorateur-v2. Avec les assets de `c2028d95`, rejoués
par le moteur actuel avant le correctif, la graine 90 reproduit **exactement le soft-lock à f5216**
(joueur 0 à terre depuis f3976, portail ouvert, position allant de (43,65 ; 7,46) à (10,36 ; −7,92)).
Avec le correctif, elle atteint l'étage 1 à **f4101**, sans desync. Assets extraits par `git archive`
et CLI identique avec un nom de jeu temporaire ; ce n'est pas un build complet du moteur historique.

Sur `origin/main` actuel, la commande demandée bloque plus tôt, à **f1591** : le `lanceur_kin` #205
(408 ; −40) chevauche la porte #71 (384 ; −56). La fermeture attend un passage libre, alors que
l'ennemi endormi attend cette fermeture pour se réveiller. Maintenant, **si un ennemi bloque une
porte ouverte**, l'entrée d'un joueur debout réveille la salle avant l'IA. La porte reste ouverte
tant qu'elle est occupée ; la dormance est recalculée à chaque frame. Après correction, la graine
90 se termine par une défaite à **f1780**, sans soft-lock ni desync. D64 (équilibrage) reste ouverte.
Instantanés et résultats : [`diagnostic-90.json`](dettes-d15-d63-d68/diagnostic-90.json).

La première tentative de lot 1–10 a aussi révélé des desyncs de fin d'arène (2, 3, 8, 9, 10).
Diagnostic détaillé sur la graine 2 : à f958, cinq `ShellPickup` et cinq incréments de
`GgrsNetIdFactory` disparaissaient en resimulation. `Death`, posée en fin de frame par l'arène,
est restaurée sans ses ticks `Added` : le drop n'était plus rejoué. Les drops de douilles,
power-ups et armes de progression interrogent maintenant `With<Death>` avant
`rollback_apply_death`, qui désactive le mort cette même frame : un seul drop, y compris après
rollback. Ce lot invalide est écarté, pas compté dans la campagne finale.

Tests de régression : `downed_explorer_waits_for_revive` échoue avant le correctif (pas réanimé à
f350), passe après ; `arena_death_loot_survives_rollback` joue la graine 2 au-delà de la fin de
l'arène. Les quatre tests d'intégration bots passent. Campagne finale : à terminer, lots de 10
joués au premier plan et poussés chacun via [`campagne.sh`](dettes-d15-d63-d68/campagne.sh).

## Preuve §5 et traces D68

Référence : binaire conservé avant les changements de collision/bots (base `77c145b7`, suite
sans bless verte). D15 a déjà prouvé l'identité de toutes ses traces. Dumps détaillés, un scénario
par appel, comparés par `scripts/trace-diff.py`, sans exclusion de type.

- `idle`, `two_players_shooting`, `points_on_kill`, `downed_all_lose` : identiques, respectivement
  1499, 299, 599 et 1699 frames comparées.
- `bench_floor` : première divergence **f83**, seuls les ennemis 309, 311, 321, 323 et 324 de la
  salle occupée changent : retrait de `RoomDormant`, puis ciblage, comportement, vitesse/position,
  orientation et émetteur de leur IA réveillée.
- `bot_explorer_two` : première divergence **f551**, même cause sur les ennemis de la salle du
  second étage. Les attentes existantes passent **sans modification**.

Toutes les autres lignes sont identiques jusqu'à cette première divergence. Les seules traces
blessées pour D68 sont `bench_floor.trace` et `bot_explorer_two.trace`, après ces preuves.
La suite sans bless avant cette mise à jour ne signalait que ces deux fichiers.
Détail : [`preuves.txt`](dettes-d15-d63-d68/preuves.txt),
[`résumé par composant`](dettes-d15-d63-d68/preuves-resume.txt). Dumps volumineux supprimés après comparaison.

## Vérification finale

À compléter après la campagne et la fusion avec `origin/main` : suite complète sans bless,
`make test_crates`, `make lint` ×4, fmt, scripts et `make gen` ×4.

## Non vérifié, dettes et atteignabilité

- **p2p à rejouer par l'orchestrateur**, requis pour ces changements de simulation D15/D68 ; pas de
  Docker utilisé conformément à la fiche. Pas de restart demandé (session/lobby inchangés).
- Bench strict sur machine au calme non vérifié ici.
- D64 reste ouverte : les salles de combat du contenu courant sont très mortelles ; ce travail
  corrige les blocages et la répétabilité, pas l'équilibrage.
- Atteignabilité : D15, les trois bancs et toute collision en partie ; D63, lint des quatre jeux ;
  D68, commande demandée graine 90 sur `etages_deux`, et campagne e2 ordinaire à deux explorateurs.
