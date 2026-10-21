# polissage-throne-jouable — D66/D67 et campagne chiffrée de throne (S2-7)

> **Révisé le 2026-10-21** : tâche S2-7 (semaine 2) dans `docs/taches/plan-polissage.md` §3.2 ; le jeu se joue dans le navigateur (captures et vidéos sur Mac), et les estimations et clones de ce document sont ceux du plan.


Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axes 3 et 4),
`docs/taches/lisibilite-throne-d66-d67.md` (fiche existante D66/D67), `docs/taches/rapports/m1-revue-humaine.md`,
`docs/taches/rapports/m1-analyse-200-throne.md`, `docs/conventions.md` §31 et §55, `games/throne/assets/characters/tourelle.ron`,
`roi_rat.ron`, `crates/game/src/feedback.rs` (`pattern_shape`). Branche `polissage-throne-jouable` depuis `origin/main`.
Estimation : 2 j. Clone : throne. Machine : VPS pour contenu et campagne ; **la vidéo de recette** (tourelle, boss) est
produite sur Mac (b3).

## Contexte

D66 et D67 sont ouvertes (`dettes.md:74-75`) : une tourelle avec silhouette de zombie et orientation figée est prise pour
un ennemi coincé ; le télégraphe de salve est pris pour une zone de dégâts. Throne : 13 armes, 14 personnages, 1 boss
(inventaire §4), placeholders copiés du testbed (`game.ron:1-4`) et **aucune campagne chiffrée**.

## À faire

1. Exécuter la fiche `lisibilite-throne-d66-d67.md` (silhouette et orientation de la tourelle, forme du télégraphe) sans
   la recopier ici ; ne retenir que ce qui reste ouvert au vu de `dettes.md`.
2. Campagne `alacod-sim` sur throne (un et deux bots, ≥ 100 graines) : étages franchis, morts par cause, soft-locks ; publier.
3. Si la campagne montre un mur (un étage franchi par moins de 20 % ou plus de 95 % des parties [INFERENCE : seuils de
   travail, William tranche]), régler **une** valeur de contenu (PV, dégâts, fréquence d'une salle) avec preuve §5 ; sinon
   ne rien changer.
4. Aucune mécanique ni ennemi nouveau.

## Acceptation

- D66 et D67 fermées dans `dettes.md` ou renvoyées avec ce qui manque ; vidéo où une tourelle se distingue d'un zombie.
- Tableau de campagne publié dans `docs/taches/rapports/polissage-throne-jouable.md`.
- Traces : présentation seule intacte ; tout réglage de contenu : rebénies avec preuve §5.
