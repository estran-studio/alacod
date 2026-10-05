# m1-relecture-conventions — relecture d'ensemble de `docs/conventions.md` et `CLAUDE.md` (vague 2 de M1)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b0, branche
`m1-relecture-conventions` créée depuis `origin/main`). Tâche de **documentation seule** : aucun
changement de code ni de contenu, aucune trace, aucune compilation nécessaire (grep et lecture).
Purge du target au début comme d'habitude (README §1). Petite tâche (1 à 2 j).

## Contexte

Le digest `docs/digests/m1-fin-de-vague-2.md` note, au critère « doc des conventions à jour » :
« à jour pour chaque tâche livrée, **sans relecture d'ensemble** ». `docs/conventions.md` a 2 078
lignes et 33 sections écrites par une douzaine d'agents sur deux jalons ; `CLAUDE.md` en a 833.
Défauts déjà visibles : il n'y a **pas de §7** ; le **§9 existe deux fois** (l. 446 « Stats et
modificateurs », l. 492 « Feedback (présentation, T2.13) ») avec le §10 inséré entre les deux ; le
§8 porte une ancre `{#section7}`. Les sections de M1 (§19 à §33) ont été remises en ordre
numérique le 2026-10-05, pas relues.

## Décisions (fixées ici)

1. **Numérotation** : une section = un numéro, dans l'ordre du fichier. Avant de renuméroter,
   **inventorier toutes les références** `§N` / `section N` / `{#sectionN}` dans `docs/`,
   `CLAUDE.md`, `README.md`, `crates/**` (commentaires `//!` et `///`), `games/**` (commentaires
   RON), `docs/taches/**` (fiches et rapports) et choisir la renumérotation qui **change le moins
   de références** (par exemple « Feedback (présentation) » devient le §7 manquant si rien ne le
   contredit). Chaque référence touchée est corrigée dans le même commit ; les rapports et fiches
   historiques (`docs/taches/rapports/`, fiches déjà livrées) ne sont **pas** réécrits : une note
   « (numérotation du 2026-10-0x : ancien §N = §M) » en tête de `conventions.md` suffit pour eux.
   Les commentaires `//!`/`///` qui citent un numéro faux sont corrigés (seule exception à
   « aucun changement de code » : commentaires uniquement, `cargo fmt` non requis).
2. **Vérité du code** : pour chaque section, chaque type, champ, fichier, commande `make`, variable
   d'environnement, nom de scénario ou de kind cité doit exister sur `main` (grep). Ce qui n'existe
   plus ou a changé de nom est corrigé dans la doc. Ce que la doc annonce « v2 / à venir / hors
   périmètre » et qui a été livré depuis est mis à jour avec la tâche qui l'a fait (journal
   `docs/taches.md` §10). Quand **le code et la doc divergent sur une décision** (la doc dit « on
   fait X », le code fait Y), **ne pas changer le code** : lister l'écart dans le rapport, section
   « écarts à décider », une ligne chacun (section, phrase de la doc, ce que fait le code, fichier).
3. **`CLAUDE.md`** : la liste des attentes (`game::replay`) doit égaler les variantes de l'enum
   (compter les deux, lister les manques dans les deux sens) ; la liste des kinds doit égaler le
   `KindRegistry` ; les cibles `make` citées doivent exister dans le `Makefile` ; la checklist
   « ajouter un vocabulaire » (conventions §4) et celle de `CLAUDE.md` ne doivent pas se
   contredire (une seule source, l'autre renvoie). Les pièges de la checklist (parité du checksum,
   `derive(Hash)`, scale et collider) restent où ils sont.
4. **Forme** : pas de réécriture de style, pas de fusion de sections, pas de déplacement de
   contenu entre `conventions.md` et `CLAUDE.md` ; « Notes essentielles » en fin de fichier reste
   si son contenu n'est pas déjà ailleurs (sinon la remplacer par un renvoi). Une table des
   matières (liste des §, une ligne chacun) en tête de `conventions.md` si elle n'existe pas.
5. **Hors périmètre** : tout changement de comportement, de contenu RON, de trace, de fiche
   historique ; la doc de `docs/plan-engine.md` et `docs/taches.md` (ne pas toucher) ; le
   `docs/taches/README.md` (une ligne de rapport si quelque chose y est faux).

## Critères d'acceptation (vérifiés par l'orchestrateur)

1. `grep -n '^## ' docs/conventions.md` : numéros uniques, croissants, continus de 1 à N ; aucune
   référence `§N` cassée (l'inventaire du rapport dit comment tu l'as vérifié : commande et
   compte avant/après).
2. Rapport `docs/taches/rapports/m1-relecture-conventions.md` : un tableau **une ligne par
   section** (numéro, titre, verdict `OK` / `corrigé (n)` / `écart`), la section « écarts à
   décider », le bilan attentes/kinds/make de `CLAUDE.md` avec les comptes, et la liste des
   commentaires de code touchés.
3. `make lint` des trois jeux et `cargo test -p content -p game --profile headless` ne sont **pas**
   requis puisque le code ne change pas ; si tu touches un commentaire dans un `#[doc]` ou un
   test de doc, dis-le et `cargo check -p <crate>` suffit (CARGO_BUILD_JOBS=2, demander le feu
   vert d'orch avant toute compilation).
4. Diff lisible : un commit par nature (renumérotation + références ; corrections de fond par
   section ; `CLAUDE.md`), messages en français.

## Livrer

Merger `origin/main` juste avant de livrer (conflits : garder les deux). `git push -u origin
m1-relecture-conventions`, puis `SendMessage` à `orch` : `LIVRÉ m1-relecture-conventions <sha> :
<n sections corrigées, n écarts à décider, numérotation : ancien §N → §M>` (ou `BLOQUÉ …`).
Ne merge pas, ne bénis pas.
