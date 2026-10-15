# m2-armes-sources — donner une source aux 20 armes de gungeon (D62, armes)

Lire d'abord `docs/taches/README.md`, `docs/taches/rapports/revue-sortie-m2.md`, dette D62,
`docs/conventions.md` §48 (butin), §50 (boutique), §52.a (armes), `content::reach`
(`loot_pool_weapons`, aujourd'hui vide). Branche `m2-armes-sources` depuis `origin/main`.
Estimation : 2 j. **Bloque la sortie de M2** (un joueur n'a que le `pistolet` : vingt armes
jamais rencontrées, 20 avertissements de lint).

## Contexte

`alacod lint games/gungeon` : 20 avertissements, tous « arme X : aucun personnage, mur ni pool ne la
donne » (dont `arsenal` et `arsenal_lourd`, contenu de test D63). Aucune entrée de pool de butin ne
désigne une arme (`LootPick` n'a pas de variante `Weapon`, ou elle n'est pas utilisée).

## À faire

1. Entrées de pool qui donnent une arme (coffres, récompense de salle, boutique : prix par rareté
   dans `economy.ron`) ; échange d'arme avec les emplacements du joueur (règle déjà posée par
   `weapon_slots` / lâcher d'arme).
2. Pondération par rareté et par étage ; au moins une arme à chaque étage dans la boutique.
3. `loot_pool_weapons` rempli depuis les pools ; lint : 0 avertissement d'arme (hors `arsenal*`,
   marqués `test_only`).
4. Atteignabilité (règle du 2026-10-14) : un scénario ou une graine `alacod-sim` qui montre une arme
   ramassée ou achetée dans une partie ordinaire (`etages_deux`, grammaire `etage`).
5. Les traces gungeon qui bougent : preuve §5 ; conventions §48 mise à jour.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Pas de Docker : le p2p n'est pas rejoué ici, le dire
au rapport. Pousser la branche régulièrement. Finir par `LIVRÉ m2-armes-sources` et le rapport
`docs/taches/rapports/m2-armes-sources.md`.
