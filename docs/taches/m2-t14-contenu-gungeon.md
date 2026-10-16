# m2-t14-contenu-gungeon — contenu du clone `gungeon` (M2, V2, données)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §4 (checklist d'un
vocabulaire, `test:` des définitions), §16 et §41 (projectiles, patterns v2), §22 et §49
(behaviors, formations), §36/§44/§48/§50 (objets, synergies, butin, boutique), §37/§45 (boss), §40
(gabarits et grammaire d'étage), §46 (surfaces v2), et `games/gungeon/assets/`. Estimation : 6 j,
**découpée en quatre parties livrables séparément** (une branche chacune) : le contenu touche les
mêmes fichiers que les tâches V1 en cours, chaque partie ne démarre que quand ses prérequis sont
sur `main`. Section de conventions réservée : **§52** (une sous-section par partie).

## Objectif (plan M2)

Vingt armes, vingt objets, dix ennemis, un boss complet, quinze gabarits de salles ; un `test:` sur
chaque définition qui en accepte un ; `make lint` vert, scénarios générés (`make gen GAME=gungeon`)
verts. Rien de nouveau côté moteur : une mécanique qui manque est notée au rapport (dette), pas
codée ici.

## Parties

1. **T14a armes** (prérequis : rien de plus que `main`) — branche `m2-t14a-armes` : quatorze armes de
   plus (vingt au total), chacune avec un caractère lisible (pistolets, fusil à pompe, mitraillette,
   arc, lance-roquettes, laser, armes à rebond/perçantes/à tête chercheuse) en composant les
   comportements existants (`firing_mode`, `bullet_type`, modificateurs T1.1, patterns) ; munitions
   par type ; `test:` par arme ; scénarios générés `weapon_*`.
2. **T14b objets** (prérequis : M2-T5, M2-T6, D61 tranchée) — `m2-t14b-objets` : dix objets de plus
   (vingt), quatre synergies, répartis dans les tables de butin et de boutique par rareté ; **aucun
   objet d'armure tant que D61 est ouverte** (la stat `Armor` n'est lue par rien).
3. **T14c ennemis et boss** (prérequis : M2-T7, M2-T8) — `m2-t14c-ennemis` : six ennemis de plus
   (dix), variantes (§25) et escouades, patterns nommés v2 ; le boss de fin d'étage en trois phases
   complet ; `test:` par ennemi.
4. **T14d gabarits** (prérequis : M2-T11, M2-T5, M2-T6) — `m2-t14d-gabarits` : quinze gabarits de
   salles au total (combat, coffre, boutique, boss, secret), surfaces et props, la grammaire `etage`
   valide sur 1 000 graines (test de M2-T10) et la campagne du bot explorateur (M2-T16) sans
   soft-lock sur 200 graines. **Atteignabilité** (`docs/taches.md` §7) : y brancher ce que les
   tâches V1 ont laissé hors de la grammaire `etage` — la salle `escouade` de M2-T8 (grammaire
   `court_escouade` seule aujourd'hui), les coffres et la boutique de M2-T5/M2-T6, les arènes de
   M2-T7, les props de M2-T11 — et nommer pour chacun la graine `alacod-sim` qui la rencontre.

## Règles

Données seulement (`games/gungeon/assets/`, scénarios, traces gungeon). Les traces `gungeon` qui
bougent : preuve §5 contre le `main` du moment, cause dans le commit. Les autres jeux ne bougent pas.
`make lint GAME=gungeon` vert, `make gen GAME=gungeon` sans diff après génération.

## Vérification (par partie)

`make test_scenarios_par` (ou `make test_scenarios`), `make test_crates`, `make lint` ×4, fmt,
`make gen` ×4 ; p2p gungeon à deux clients ; rapport `docs/taches/rapports/m2-t14<x>-*.md` ;
livrer par `LIVRÉ m2-t14<x>-… <sha>`.

## T14c en session cloud (ajout du 2026-10-14)

T14c peut démarrer : M2-T7 (boss `gatling`, arène) et M2-T8 (escouades) sont sur `main`, M2-T15 aussi.
- **Atteignabilité vérifiée par le lint** (M2-T15, §53) : chaque nouvel ennemi doit apparaître dans
  une salle d'un type tiré par la grammaire `etage`, une arène ou un renfort de boss.
  `alacod lint gungeon` ne doit avoir **aucun nouvel avertissement d'ennemi** (aujourd'hui : `cible`
  seulement, dette D63). Placer les ennemis dans les gabarits existants de `salles.ldtk` (ou dans les
  vagues d'arène), sans ajouter de gabarit : c'est T14d.
- Le boss « complet en trois phases » part du `gatling` de M2-T7 : compléter ses patterns nommés v2 et
  ses renforts, pas un second boss. Les traces `gungeon_boss*`, `bot_explorer_*`, `boss_gatling` qui
  bougent : preuve §5, cause dans le commit.
- Une mécanique qui manque : dette au rapport, pas de code moteur.
- Session cloud : ignorer les consignes `orca` du README (§1) ; pas de Docker, le p2p est rejoué par
  l'orchestrateur, le dire au rapport. Pousser la branche régulièrement (commits WIP acceptés ; le
  conteneur redémarre environ toutes les deux heures : lancer la suite au premier plan, de préférence
  `make test_scenarios`, `make test_scenarios_par` exige un `CARGO_TARGET_DIR` absolu). Finir par un
  commit `LIVRÉ m2-t14c-ennemis` qui contient le rapport `docs/taches/rapports/m2-t14c-ennemis.md`.

## T14d en session cloud, avec D64 (ajout du 2026-10-15)

Prérequis tous sur `main` : M2-T11 (surfaces), M2-T5 (coffres), M2-T6 (boutique), M2-T14c (ennemis),
bot-explorateur-v2 (l'explorateur joue l'arène, la récompense et la boutique), D59 (gungeon démarre sur
`etages_deux`). Branche `m2-t14d-gabarits` depuis `origin/main`.
- **Gabarits** : la partie 4 ci-dessus (quinze gabarits au total, surfaces et props, salle `escouade`
  branchée dans `etage`, grammaire valide sur 1 000 graines), sans nouvelle mécanique.
- **D64, équilibrage des salles `combat`** (`docs/taches/dettes.md`) : sur `etages_deux`, le bot
  `explorateur` meurt dans 200 parties sur 200 à 1 bot et 67 sur 200 à 2 bots (après bot-explorateur-v2,
  avant les ennemis de T14c). Objectif : **moins de 15 % de morts à 2 bots et moins de 50 % à 1 bot** sur
  200 graines (`docs/taches/rapports/bot-explorateur-v2/` : `campagne.sh`, `analyse.py`), 0 soft-lock,
  0 desync. Leviers en données seulement : nombre et composition des ennemis par gabarit, dégâts des
  armes ennemies, salles de récompense ou de soin sur le chemin. Ne pas toucher au bot ni au boss.
  Tableau avant/après au rapport, avec les tueurs principaux.
- Les traces gungeon qui bougent : preuve §5 contre le `main` du moment, cause dans le commit. Les autres
  jeux ne bougent pas.
- Session cloud : ignorer les consignes `orca` du README (§1) ; pas de Docker, le p2p gungeon est rejoué
  par l'orchestrateur. Campagnes au premier plan, par lots de 10 graines poussés un par un (le conteneur
  redémarre environ toutes les deux heures). Voie rapide du README §6 : fusionner `origin/main` et
  rejouer la suite juste avant le commit `LIVRÉ m2-t14d-gabarits`, rapport
  `docs/taches/rapports/m2-t14d-gabarits.md`.

**Fait dans `m2-t14d-gabarits` (2026-10-16)** : quinze gabarits, grammaire `etage` à sept types exigés (escouade comprise) sur 1 000 graines, D64 tenue (1 bot 27 % de morts, 2 bots 3,5 %, 0 desync) ; un soft-lock résiduel à 1 bot (4 sur 200) est le cycle limite du bot, D80. Conventions §52.d, rapport `docs/taches/rapports/m2-t14d-gabarits.md`.
