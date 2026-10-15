# m2-t13-hub — hub de gungeon : la Brèche, marchand de déblocages, entrée en run (M2-T13, chantier G2)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/plan-engine.md` §6 G (G1 profil, G2 hub),
`docs/conventions.md` §38 (profil), §51 (déblocages de M2-T12), §50 (boutique), §33 (restart en
ligne), §40 (grammaire d'étage), `docs/taches/rapports/m2-t12-profil-deblocages.md`. Branche
`m2-t13-hub` depuis `origin/main`, **une fois M2-T12 mergée** (`git log origin/main` contient
`merge m2-t12-profil-deblocages` ; sinon, attendre). Estimation : 3 j. Section de conventions
réservée : **§57**.

## Objectif (plan M2)

Une carte de hub (LDtk) avec services activés par drapeaux de profil, un PNJ marchand de déblocages
et l'entrée en run. Le joueur arrive au hub, achète des déblocages avec la monnaie méta (M2-T12),
entre dans la run par une porte ; à la fin de la run, il revient au hub.

## À faire

1. **Carte** `games/gungeon/assets/hub.ldtk` (ou un niveau de `salles.ldtk`) : une salle, la porte
   d'entrée en run, l'emplacement du marchand. Gabarit simple, sans asset nouveau obligatoire.
2. **Services en données** (`hub/*.ron`, plan-engine §6, arborescence) : un service = une condition
   (drapeau de profil, déblocage) et une action (ouvrir l'écran du marchand, entrer en run). Lint :
   références, conditions connues, fixtures.
3. **Marchand** : écran en `bevy_hui` (décision du 2026-10-15) qui liste les `Unlock` de M2-T12
   achetables, avec leur coût. Achat = écriture du profil **hors simulation**, comme M2-T12.
4. **Entrée et sortie** :
   - la porte lance la run avec les `RunUnlocks` figés (règle de M2-T12 : en ligne, l'hôte) ;
   - la fin de run (victoire ou défaite) ramène au hub au lieu de l'écran de fin, si le jeu déclare
     un hub ;
   - le restart en ligne (§33) reste possible.
5. **Preuves** :
   - scénario `hub_enter_run` (le joueur traverse le hub et entre en run, `FloorIndex(0)`) ;
   - scénario `hub_buy_unlock` (achat, puis `ProfileHas` ; le déblocage est présent dans les pools de
     la run suivante).
   - Atteignabilité : une partie gungeon ordinaire démarre au hub. Dire au rapport l'effet sur les
     scénarios qui jouent « la partie du manifeste », qu'il faut peut-être fixer sur `floors`.

## Règles

Le hub est dans la simulation (déplacement, interaction) : `order_iter!`, `FrameEvents`, pas de `f32`.
Les écritures de profil restent hors `GgrsSchedule`. Les traces qui bougent : preuve §5, cause dans le
commit. `cargo fmt --all -- --check`, `make check_sim_boundary`.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de
Docker : le p2p (entrée en run à deux clients depuis le hub) est rejoué par l'orchestrateur ; le dire
au rapport.

Pousser la branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et rejouer la
suite juste avant le commit `LIVRÉ m2-t13-hub`, avec le rapport `docs/taches/rapports/m2-t13-hub.md`.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
scripts, `make gen` ×4, `make check_sim_boundary`).
