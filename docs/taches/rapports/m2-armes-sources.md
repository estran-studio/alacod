# Rapport m2-armes-sources (D62) — des sources pour les armes de gungeon

Base : `origin/verif/train15`, puis fusion de `origin/main` (3bab4764) avant livraison.

## Livré
- `LootPick::Weapon(id, rareté)` : une arme est une entrée de pool (coffres, boutique) ; `weapon:<id>` désigne le tirage (`items::loot_weapon`). Tirage RNG inchangé (deux `next_u32`).
- Coffres : l'arme tombe au sol (`game::loot::spawn_loot_weapon`) ; boutique : prix par rareté, l'arme achetée tombe au pied de l'acheteur. `RoomShopDef::weapon_shelves` (au moins une arme par boutique) et `floor_tables` (pool par étage).
- Contenu gungeon : 18 armes (hors `pistolet`) réparties en 5 communes / 7 peu communes / 6 rares ; pools `coffre_etage1`/`coffre_etage2`, `boutique`/`boutique_profond` ; `floors/etages_deux.ron` (`loot`) ; `arsenal` et `arsenal_lourd` en `test_only`. `commun` reste sans arme (scénarios de déblocage), `coffre_riche` enrichit aussi les pools d'étage.
- Lint : arme inconnue, `weapon_shelves`/`floor_tables` incohérents ; `reach::loot_pool_weapons` rempli. `alacod lint games/gungeon` : 0 avertissement d'arme.
- Atteignabilité : option `ExplorerOptions::shopping` (`alacod-sim --shopping`, faux par défaut) et test `explorer_finds_weapons_on_ordinary_floors` (graine 3 d'`etages_deux`, arme achetée à f69). Scénario `shop_weapon` (achat puis échange d'emplacement). Test unitaire `weapon_entries`.
- Correctif trouvé par le bot : l'arme lâchée par un échange d'emplacement n'est ramassable qu'après `WEAPON_SWAP_COOLDOWN_FRAMES` (30) ; sans cela Interaction maintenu échangeait à chaque frame (emplacement unique). Conventions §48.

## Traces qui bougent (preuve §5)
`shop_seeded`, `minimap_shop_visit`, `bench_floor` (boutique : les présentoirs sont posés au chargement, donc les traces divergent dès la frame 0, comme prévu : seul l'identifiant d'objet des `Shelf` change ; les trois autres scénarios de boutique et tous les scénarios sans boutique sont identiques) et `shop_weapon` (nouveau). Attentes de `shop_seeded` recalibrées (même rôle, aucune retirée). Aucune autre trace ne bouge ; `make gen` des 5 jeux sans diff.

## Non vérifié / à rejouer
- p2p non rejoué (pas de Docker ici) : à l'orchestrateur.
- clippy absent du nightly : non vérifié.
- Le bot `explorateur` de gungeon meurt tôt sur la plupart des graines d'`etage` (avant et après ce changement) : la preuve d'atteignabilité repose sur la graine 3, dont la boutique est voisine du départ.
