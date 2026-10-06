# Rapport — m1-v3-bots-armes (choix d'arme par la config, tir à portée, ramassage)

Branche partie de `origin/main` 8379d5d (D51 mergée).

## État en cours

Vérifié, livré (décision d'orch : (a), livrer tel quel ; l'anticipation devient
m1-v3-bots-lead). Suite complète : attentes vertes après remesure des six scénarios à bots qui
bougent ; lint ×3, fmt, interdits, enregistrement rollback, `make gen` ×3 (aucun fichier
modifié, aucune trace générée ne bouge), `cargo check -p throne`, exemples : verts ; tests `bots`
58/58. Target purgé.

## Ce qui est fait (`crates/bots` seulement, mode `Floors`, `prudent` et `fonceur`)

Conventions §24 « Armes des bots ».

- `arms.rs` (pur, testé) : `WeaponView::from_config` (mode de tir courant : dégâts par
  projectile, projectiles par tir — plombs, rafale —, cadence, portée, pleine largeur de la gerbe
  `spread` ou `spread_angle`), `expected_dps` = dégâts × projectiles × cadence × part qui touche,
  nul hors portée ou sans munitions ; part qui touche = `min(1, 2 × rayon / (distance × spread))`
  (dispersion réelle depuis D51 ; la formule `r / (d × s)` de ma proposition était fausse d'un
  facteur 2, corrigée avant le code). `better_weapon` : gain ≥ 1,5 × ou arme en main inutile.
  `WeaponChoices::step` : hystérésis de 120 frames, index visé poursuivi (`switch_weapon` pressé,
  le jeu cycle d'un emplacement par appui). `worth_picking` : arme au sol meilleure que l'arme en
  main à 250 px (emplacements pleins : c'est elle qui tombe) ou emplacement libre.
- `WeaponChoices` (hors rollback, comme la navigation) : frame de simulation, aucun tirage RNG,
  vidée à chaque `OnEnter(AppState::InGame)` (première partie, restart D14) ; test
  `choix_deterministes_et_tenus` (deux mémoires sur la même suite : mêmes appuis, même état ;
  mémoire vidée : même suite).
- `input.rs` : vue des armes portées, rayon de la cible (collider de l'ennemi le plus proche, 12
  par défaut), `BotView::fire_range` (portée de l'arme en main) ; ramassage sans ennemi visible à
  moins de 150 px : power-up à moins de 96 px (partout à sec, règle de m1-v3-bots-softlocks),
  arme au sol (`WeaponPickup` sans prix) à moins de 96 px si `worth_picking` et pas déjà portée,
  Interaction tenue par `surface_step` (factorisé avec la réanimation, même logique).
- `decide.rs` : pas de tir au-delà de `fire_range` (`in_range`) ; `fonceur` change d'arme en
  `Floors` (`manage_weapon`) ; Interaction sur `loot_interact`. Hors `Floors` : inchangé (aucun
  scénario zombies ni en vagues ne bouge).
- Tests : `arms::tests` (dégâts attendus : à 300 px mitraillette ≈ 43/s, lance-lames 36, laser
  48 ; à 500 px mitraillette ≈ 26 ; hors portée / vide ; hystérésis de gain ; ramassage ; choix
  déterministes), `decide::tests::tir_seulement_a_portee_de_l_arme`,
  `ramasser_une_arme_tient_interaction`.

## Mesure avant/après

« Avant » = `main` 8379d5d, identique en code à la branche D51 (`git diff 1a14874 origin/main --
crates` vide : même binaire que l'« après » de D51) ; « après » = cette branche ; mêmes commandes,
une sim à la fois, balles perdues par la méthode du digest (trace détaillée, un dump à la fois).

| | avant | après |
|---|---|---|
| throne 2 bots, témoins 1..20 : étage 3 / défaites / soft-locks / desync | 20/20 / 0 / 0 / 0 | 20/20 / 0 / 0 / 0 |
| throne 2 bots : frames moyennes jusqu'à l'étage 3 | 3554 | 3501 |
| throne 4 bots : étage 3 / défaites / desync | 20/20 / 0 / 0 | 20/20 / 0 / 0 |
| throne 4 bots : frames moyennes | 2391 | 2358 |
| balles perdues au 3e étage, graine 25 | 62 % | 56 % |
| graine 103 | 56 % | 57 % |
| graine 131 | 54 % | 61 % |

**Résultat neutre, et pourquoi** : la mitraillette tire seule dans les trois rejeux, avant comme
après. `prudent` combat entre 180 et 320 px ; à ces distances, avec la vraie dispersion (D51), la
mitraillette (0,15) fait ≈ 40 à 64 dégâts/s sur une cible de rayon 12, contre laser 48,
lance-lames 36, revolver 30 : aucune arme portée n'atteint 1,5 ×, la règle ne change pas d'arme,
**et c'est juste** : depuis D51, la mitraillette est réellement la meilleure arme à la portée où
`prudent` se tient. Le constat « mitraillette seule dans 12/12 » du digest de b0 venait du bug de
dispersion (toutes les armes à ±0,5 rad), pas d'un défaut de choix. Le ramassage et le tir à
portée n'ont pas d'effet mesurable sur ces graines.

**Critère « balles perdues en baisse » : non atteint** (médiane 56 → 57 %). À 250 px, la gerbe de
la mitraillette (±19 px) couvre ≈ 64 % d'une cible de ±12 px ; le reste se perd sur des cibles
mobiles (balles à 300 px/s, aucune anticipation, hors périmètre v1) : c'est l'objet de
m1-v3-bots-lead, mesuré seul. Les autres critères de merge tiennent : aucun soft-lock ajouté,
défaites non augmentées sur les témoins (0 → 0), étage 3 égal.

**Régression notée** : `throne_three_floors` (graine 4, 2 bots, joué jusqu'à f5000, au-delà de
l'objectif) : l'étage 3 est atteint (f3239), mais le joueur 1 tombe à f2317 (étage 2) et n'est
pas relevé, puis les deux meurent (f4117, f4223) dans la quatrième caverne ; avant, personne à
terre et les deux vivants. Les témoins (`--until-floor 3`) s'arrêtent à l'étage 3 et ne la voient
pas. À l'inverse, `throne_solo` : le bot seul survit maintenant jusqu'à la fin (avant D51 + armes :
mort à f2908).

## Scénarios et traces

Bougent (**six** scénarios à bots ; `throne_softlock_recul`, `clone_quad`, `bots_four_mixed`,
zombies, vagues, testbed hors `bot_floors_three` : inchangés) ; preuve par inputs (`alacod-sim`
`--save-scenario`, `main` contre la branche, même graine, même nombre de bots) :

| scénario | premier input différent | ce qui change | trace diverge à |
|---|---|---|---|
| `throne_floor_1`, `throne_progression`, `throne_solo` (1 bot, 123456) | f470, joueur 0 | `Up` → `Up, Right` sans ennemi (pan 0) : détour vers un butin | ligne 476 (f475) |
| `throne_quad` (4 bots, 123456) | f335, joueur 0 | `Right` → `Up, Right` : détour vers un butin | ligne 341 (f340) |
| `throne_three_floors` (2 bots, graine 4) | f965, joueur 0 | `Up, Left` → `Down, Left` : détour vers un butin | ligne 971 (f970) |
| `bot_floors_three` (testbed, 2 bots) | f0, joueur 0 | `Fire` → `Fire, SwitchWeapon` : choix d'arme | ligne 22 (f21) |

Les traces throne divergent 5 frames après le premier input différent ; `bot_floors_three` à f21 :
le changement d'arme demandé à f0 n'a lieu qu'après la garde de 20 frames du jeu
(`frame_switched + 20 < frame`).

Attentes remesurées (`ALACOD_EVENTS=1`) : `bot_floors_three` (portails à f41, f259, f548),
`throne_floor_1` (franchi à f558), `throne_progression` (niveau 2 à f959, `tireur` à f1563, étage 2
à f1643), `throne_solo` (vivant à f3699), `throne_quad` (étages à f433, f854, f1902),
`throne_three_floors` (voir la régression : `PlayerDowned` du joueur 1 à f2320, morts à f4120 et
f4225).
