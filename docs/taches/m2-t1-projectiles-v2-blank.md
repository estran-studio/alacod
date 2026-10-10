# m2-t1-projectiles-v2-blank — projectiles v2 et blank (M2, V1a, chantier B5 v2)

Lire d'abord `docs/taches/README.md` (machine `orca`, §1 : `make sweep`, `make test_crates`,
signaling par session), `CLAUDE.md`, `docs/conventions.md` §16 (projectiles composables), §22 (tir
ennemi, émetteurs, `Pattern`), §36 (objets, consommable `blank`), §39 (`gungeon`), et les rapports
`m1-v1a-patterns-tir-ennemi` et `m2-t0b-contrats-objets`. Branche `m2-t1-projectiles-v2-blank`
depuis `main`. Estimation : 3 j. Section de conventions réservée : **§41**
(§40 = grammaire d'étage, T10 en cours).

## Contexte

`combat::projectile::Pattern` sait `Aimed`, `Spread`, `Ring`, `Sequence`, `Telegraph`, `Wait`,
`Scatter`, `Named` ; les modificateurs `Bounce`/`Pierce`/`Size`/`Lifetime`/`Homing`/`Gravity`
existent (T1.1). T0b a posé le **contrat** du blank : bit d'input 17 (`INPUT_BLANK`, touche Q,
bouton `Blank`), consommable `blank` dans l'inventaire ; **aucun effet** encore.

## À faire

1. **Patterns** `Spiral` (angle qui tourne de `step` par tir, `count` bras), `Fan` (éventail de
   `count` balles réparties exactement sur `spread`, sans hasard, à la différence de `Scatter`),
   `Burst` (`count` salves de `per_burst` espacées de `interval` frames). Ajoutés **en fin d'enum**
   (hash des variantes existantes inchangé, voir le commentaire T1.2) ; état d'avancement dans
   l'émetteur (rollback) ; séquences seedées via le flux RNG `"patterns"` quand il y a du hasard.
   Lint : bornes (`count` > 0, `interval` > 0), refus en `on_expire` si étape temporelle.
2. **Rebond des balles ennemies** : `Bounce` sur les murs fonctionne aussi pour un projectile tiré
   par un ennemi (vérifier, corriger sinon) ; un pattern de `gungeon` l'utilise.
3. **Blank** : appui sur Blank avec au moins un `blank` en inventaire → consomme un `blank`, efface
   toutes les **balles ennemies** dans un rayon (donnée, RON du jeu), bref i-frames ou repousse
   (au choix, en donnée) ; jamais les balles des joueurs. `despawn_rollback()`,
   `FrameEvents<BlankUsed>` (lu par la présentation et par M2-T3 `OnBlank` plus tard), recharge
   de `n` blanks au début de chaque étage (donnée). Front montant de l'input (pas de blank à chaque
   frame tant que Q est tenu ; mémoire de l'appui précédent **rollbackée**, piège « donnée dérivée
   non rollbackée » de `CLAUDE.md`).
4. **Contenu `gungeon`** : deux ou trois patterns nommés (spirale, éventail, salve) sur
   `bullet_kin` et le boss squelette ; `blank` de départ dans l'inventaire du personnage.
5. **Scénarios** : un par pattern (`pattern_spiral`, `pattern_fan`, `pattern_burst` : nombre de
   balles et angles attendus, `BulletCount`), `enemy_bullet_bounce`, `blank_clears_bullets`
   (balles ennemies effacées, celles du joueur gardées, compteur décrémenté : `Consumable`),
   `blank_empty` (sans blank : rien). **`bench_bullets` à 500 balles** dans une salle verrouillée
   (§35), plancher dans `tests/budgets.ron`.
6. Présentation minimale : flash/onde du blank dérivé de `BlankUsed` dans `Update` ; HUD
   `gungeon` : compteur de blanks si la source existe déjà, sinon laisser à M2-T18.

## Règles

Aucune trace existante ne doit bouger (nouveaux types : variante **neutre** ; nouvelles variantes
d'enum en fin) ; si une bouge, preuve §5 (`ALACOD_DUMP_TRACE` main / branche + `trace-diff.py`)
avant tout bless. Pas de `f32` dans la simulation, itération `order_iter!`.

## Hors périmètre

Esquive v2 (M2-T2, même voie, tâche suivante), déclencheur `OnBlank` des effets (M2-T3), parade
(B5 parade), télégraphes visuels des patterns (M2-T19).

## Vérification

README §4 (`make test_scenarios`, `make test_crates`, lint ×4, fmt, scripts, `make gen` ×4) ;
p2p à deux clients sur `gungeon` avec un blank utilisé (port de la session, `MATCHBOX_PORT`) ;
rapport `docs/taches/rapports/m2-t1-projectiles-v2-blank.md`, conventions §41. Livrer par
`LIVRÉ m2-t1-projectiles-v2-blank <sha>`.
