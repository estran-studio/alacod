# plan-m4-isaac — découper M4 (`isaac`) en fiches, comme M2 l'a été

Lire d'abord `docs/taches/README.md`, `docs/plan-engine.md` en entier (surtout §1 « minimal », §2
écarts par jeu, §6 chantiers, le tableau des jalons et §9.8), `docs/taches.md` §7 (M2 en tâches, voies,
prérequis, atteignabilité), deux fiches M2 comme modèles (`docs/taches/m2-t5-butin-coffres.md`,
`docs/taches/m2-t14-contenu-gungeon.md`), `docs/etat-engine.md`, et le code des chantiers que M4
étend (`crates/effects`, `crates/items`, `combat::projectile`, `world`). Branche `plan-m4-isaac` depuis
`origin/main`. Estimation : 1 j. **Documentation seulement, aucun code.**

## Contexte

M3 est la tranche verticale de 1837 : elle attend la bible de William et sera planifiée avec lui. M4
(`isaac`, Binding of Isaac) n'en dépend pas. Le plan prévoit ses chantiers :
- effets à l'échelle (100 objets en RON) ;
- empilement de modificateurs de projectiles et par nombre d'exemplaires ;
- familiers et orbitaux ;
- rochers et bombes ;
- cœurs, clés, sous ;
- marchés du diable, curses, champions et élites.

Le préparer maintenant permet de lancer ses tâches de moteur en parallèle de la fin de M2.

## À faire

1. **Écart** : ce que M2 a déjà livré (objets, synergies, butin, salles, boss, grammaire, bot
   explorateur, générateur) et ce que M4 ajoute, chantier par chantier, en un tableau.
2. **Tâches** `M4-T0…` dans `docs/taches.md` (nouvelle section §8.x « M4 en tâches ») :
   - voies parallèles et ordre de merge, comme §7 pour M2 ;
   - contrats d'abord (T0) : types partagés, kinds de contenu, sections de conventions réservées ;
   - puis mécaniques, contenu, outillage (générateur, bot), présentation.
   - Estimation par tâche.
   - **Prérequis explicites** : ce qui peut démarrer avant la fin de M2, et ce qui attend.
3. **Une fiche par tâche** dans `docs/taches/m4-t<n>-<sujet>.md`, au format des fiches M2 :
   - lire d'abord, contexte, à faire, règles, hors périmètre, vérification ;
   - section « Session cloud » quand la tâche s'y prête.
   - Les fiches des tâches T0 (contrats) sont détaillées ; les autres peuvent être plus courtes.
4. **Atteignabilité** dès la conception : chaque mécanique nomme où une partie `isaac` ordinaire la
   rencontre (règle §7 de `docs/taches.md`).
5. **Critères de sortie de M4** (complément de `docs/plan-engine.md` §9.8), mesurables (scénarios,
   campagnes du bot, p2p).

## Règles

Documentation seulement. Ne pas renommer ni déplacer les fiches existantes. Ne rien décider à la place
de William sur le design : lister les questions ouvertes au rapport.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Pousser la branche régulièrement et finir par un
commit `LIVRÉ plan-m4-isaac` qui contient le rapport `docs/taches/rapports/plan-m4-isaac.md` :
- liste des fiches ;
- tâches lançables tout de suite ;
- questions pour William.
