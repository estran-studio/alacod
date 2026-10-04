# m1-throne-gen-et-d40 — `make gen GAME=throne` vert et butin par munition (D40)

Lire d'abord `docs/taches/README.md` (agent **local**, branche depuis la tête livrée précédente,
`origin/main` mergé d'abord). Petite tâche (1 j). Deux sujets indépendants, un commit chacun.

## Décisions (fixées ; l'agent amende en dix lignes s'il voit un problème)

1. **`generate_template` de throne** : `games/throne/assets/game.ron` déclare
   `generate_template: (map: "throne/gabarit_armes.ldtk", target: "cible")` (carte et cible déjà
   livrées par b1 en phase 2 ; vérifier que la cible est bien à +128 / −48 du spawn du joueur en
   coordonnées LDtk et qu'elle a `counts_hits`). `make gen GAME=throne` **vert** : 15 scénarios d'armes
   (12 à distance + 3 de mêlée) + les gabarits ennemis des personnages throne qui déclarent `test:`
   (en ajouter sur les 10 ennemis, opt-in T1.13 : 20 scénarios). Les générés sont **versionnés**
   (`tests/scenarios/generated/throne/`) ; les `test:` sont calibrés par la mesure (`min_hits`
   atteignable, `max_hits` si utile). Traces : bless orchestrateur. Zombies et testbed : rien.
2. **D40, butin par munition** : nouvelle variante `Action::RefillAmmoOf(ammo: AmmoType)` (dernière
   variante, piège du derive à une variante sans objet ici : l'enum a déjà plusieurs variantes) : même
   règle que `RefillAmmo` (§14) restreinte à une munition ; power-ups throne `munitions_<type>` par
   munition (drop pondéré) ; lint : munition connue du jeu (au moins une arme la déclare). `RefillAmmo`
   inchangé (traces zombies/testbed intactes : aucun contenu n'utilise la nouvelle variante hors throne).
   Scénario `throne_ammo_pickup` (placement scripté T1.13 `powerups` + `Ammo`/`AmmoReserve`).
3. **Hors périmètre** : `grunt` sans attaque (deuxième moitié de D40, décision de William), écran de
   mutation, HUD.

## Règles

Deux compilations au plus ; purge + point d'état ; aucune trace bénie par l'agent ; `docs/conventions.md` :
une ligne au §14 (`RefillAmmoOf`) et une au §29 (gabarits throne) ; `dettes.md` : ne pas toucher.

## Livrer

Rapport `docs/taches/rapports/m1-throne-gen-et-d40.md` ; `git push -u origin m1-throne-gen-et-d40` ;
`SendMessage` à `orch` : `LIVRÉ m1-throne-gen-et-d40 <sha> : <une ligne>`. Ne merge pas, ne bénis pas.
