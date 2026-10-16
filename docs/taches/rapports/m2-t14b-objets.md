# Rapport m2-t14b-objets (M2-T14, partie 2)

Données seulement (`games/gungeon/assets/{objets,synergies}/`) ; aucun code moteur. Conventions : §52.b.

## Livré
- **Dix objets de plus (vingt)** : `gilet`, `egide` (armure et bouclier, D61), `lunettes`, `chargeur_rapide`,
  `roulade`, `trophee`, `parchemin`, `sablier`, `bastion`, `rancune`. Répartition : 4 Commun, 4 Peu commun, 2 Rare.
  Un `test:` (scénario généré `item_<id>`) sur chaque passif et actif ; `egide`, consommable, n'en a pas
  (comme `ration`, `blank`, `key`).
- **Quatre synergies de plus (six)** : `rempart`, `fuite_armee`, `sang_froid`, `tireur_d_elite`.
  Lecture de la fiche : « quatre synergies » = quatre en plus des deux de M2-T4 (à confirmer).
- **Pools** : `commun`, `boutique` et `boss` tirent par rareté, les nouveaux objets y entrent sans modifier les pools.
  Prix de boutique par rareté (15/30/50, §50).

## Traces qui bougent (preuve §5, contre main 7277ef9d / d826679d)
Seulement `shop_seeded` et `bench_floor`, dès la frame 0. Cause : les présentoirs de boutique tirent leurs objets
dans un pool élargi (`Shelf.item_id` change, ex. `shop_seeded` présentoir 1 `trefle` → `trophee`, présentoir 2
`blank` → `gilet`) ; seules les lignes `Shelf` diffèrent (`scripts/trace-diff.py`, dump avec/sans les 14 fichiers).
`shop_seeded` : deux attentes `Shelf` recalées (objet seulement, prix inchangés 30 et 15, aucune retirée).
Les 9 scénarios `item_*` sont nouveaux (traces bénies par `GEN_BLESS=1`) ; les traces générées existantes sont
identiques.

## Dette / pièges
- `Armor` sans valeur de base : le générateur (`expected_item_stats`) refuse un passif dont `modifiers` touche `Armor`
  (`MissingStat`). `gilet` pose l'armure par un effet `OnPickup` ; à corriger côté générateur (lire 0 pour une stat
  sans base) si on veut des `modifiers` d'armure testés.
- `alacod lint gungeon` : aucun nouvel avertissement (restent les 20 armes de T14a non branchées).
- Non joué ici : synergies sans scénario dédié (les `test:` portent sur les objets seuls).

## Vérifications
`make test_scenarios_par` (4 tranches) vert, `make test_crates` vert, `make lint` ×4 vert, `cargo fmt --check`
propre, `make gen` ×4 sans diff, p2p gungeon à deux clients (900 frames, entrées neutres) : traces identiques
(899 lignes). Limite : en p2p sans entrée, aucun objet n'est ramassé ni acheté ; les objets sont couverts par
les scénarios `item_*` et `shop_seeded`.
