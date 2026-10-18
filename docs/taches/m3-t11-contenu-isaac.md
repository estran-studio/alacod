# m3-t11-contenu-isaac — contenu du clone `isaac` (M3, V2, données)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1 (atteignabilité, critères de
sortie), `docs/conventions.md` §4 (`test:` des définitions), §60 à §75 (vocabulaires de M3), §52 (contenu
gungeon, modèle), et `games/isaac/assets/`. Estimation : 10 j, **en quatre parties livrables séparément**
(une branche chacune), chacune quand ses prérequis sont sur `main`. Section de conventions réservée :
**§76** (une sous-section par partie). Volume total : **Q1** (proposition : 100 objets, dont au moins 30
qui modifient les larmes, 8 familiers ou orbitaux, 10 actifs ; 10 ennemis dont 3 à champions ; 2 boss ;
20 gabarits). Noms et visuels originaux, rien du jeu commercial : **Q9**.

## Objectif

Le clone se joue avec du contenu seulement : chaque définition a son `test:` et son scénario généré
(`make gen GAME=isaac`) vert, `make lint GAME=isaac` sans erreur et sans avertissement d'atteignabilité hors
`test_only`. Une mécanique qui manque est une dette au rapport, pas du code ici.

## Parties

1. **T11a larmes et stats** (prérequis : M3-T1) — `m3-t11a-larmes` : le personnage passe en `Infinite` ; la
   moitié des objets (stats `Damage`, `FireRate`, `Range`, `ShotSpeed`, `ShotSize`, `MoveSpeed`, `Luck`,
   modificateurs de larmes, statuts à l'impact), répartis dans les pools par rareté.
2. **T11b effets, familiers, actifs** (prérequis : M3-T2, M3-T6 ; T3 pour les cœurs) — `m3-t11b-effets` :
   l'autre moitié (déclencheurs v3, `PerCopy`, actifs à charge par salles, familiers, orbitaux,
   transformations si Q10) ; graines `alacod-sim` nommées pour une transformation et un familier.
3. **T11c ennemis, champions, boss** (prérequis : M3-T7, M3-T4 pour les explosions) — `m3-t11c-ennemis` :
   dix ennemis (au sol, volants `Flying`, tireurs en patterns), trois à champions, deux boss à phases ;
   placement dans les gabarits existants (sans en ajouter : c'est T11d).
4. **T11d gabarits, pools, curses, équilibrage** (prérequis : M3-T5, M3-T8, M3-T9, M3-T3) — **coupé en deux par William le 2026-10-18** :
   - **T11d1 structure de la partie ordinaire** — `m3-t11d1-gabarits` (**livrée**, rapport `docs/taches/rapports/m3-t11d1-gabarits.md`, §76.4) :
     vingt gabarits en grille (neuf de combat avec rochers, rochers à butin et piques, trois de boss, trois de trésor, secrète, diable, boutique),
     boutique et pool `boutique`, curses dans la séquence par défaut, bombe de départ (D81), tableau d'atteignabilité du §8.1 rempli, campagne
     de contrôle (200 graines à 1 et 2 bots). **L'équilibrage n'est pas un objectif** (William est sceptique sur l'équilibrage mesuré par bot dans la
     boucle de développement) : victoires et morts sont rapportés, rien n'est réglé pour une fourchette (Q14 reste ouverte).
   - **T11d2 volume de Q1** — `m3-t11d2-volume` (**livrée**, rapport `docs/taches/rapports/m3-t11d2-volume.md`, §76.5) : objets 49 → **100** (dont 37 de larmes, 9 familiers ou orbitaux, 11 actifs),
     ennemis 7 → **10** (dont 3 à champions), boss 1 → **2** (`colosse`, type de salle `boss_profond`, second étage de la séquence par défaut), placement dans les gabarits de T11d1 ; dettes D112 à D115 (mécaniques qui manquent).

## Règles

Données seulement (`games/isaac/assets/`, scénarios et traces `isaac`). Les traces `isaac` qui bougent :
preuve §5 contre le `main` du moment, cause dans le commit. Les autres jeux ne bougent pas. `make gen
GAME=isaac` sans diff après génération.

## Vérification (par partie)

README §4 (`make test_scenarios`, `make test_crates`, `make lint` ×5, fmt, `make gen` ×5) ; rapport
`docs/taches/rapports/m3-t11<x>-*.md` ; livraison par commit `LIVRÉ m3-t11<x>-…`.

## Atteignabilité

C'est le rôle de la partie d1 : le tableau du §8.1 de `docs/taches.md` est rempli ligne par ligne (scénario ou
graine) au rapport de T11d1 (`docs/taches/rapports/m3-t11d1-gabarits.md`).

## Session cloud

Chaque partie est prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas
de Docker : p2p `isaac` rejoué par l'orchestrateur, le dire au rapport. Campagnes au premier plan, par lots
de 10 graines poussés un par un (le conteneur redémarre environ toutes les deux heures). Voie rapide du README
§6 : fusionner `origin/main` et rejouer la suite juste avant le commit `LIVRÉ`.
