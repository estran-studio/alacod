# m1-d53-bench-horde — `bench_horde` a perdu la moitié de sa vitesse pendant M1 (D53)

## État en cours

- Bornes mesurées (même machine, même target, sources touchées à chaque commit, deux passages,
  charge 5 à 7 sur 12 cœurs à cause d'une sim de b1) : **`9e85e79` (fin de M0, m1-v1e) 63,7 /
  63,6 fps**, **`a8b813e` 33,0 / 32,7 fps** ; scénario `bench_horde.ron` identique entre les deux
  (seule sa trace a été rebénie au snap de m0-v7 p2). L'écart (÷ 1,9) se reproduit.
- Bissection automatique en cours sur les 40 commits first-parent qui touchent `crates/`,
  `games/` ou `Cargo.*` (`../d40/d53_bisect.sh`, seuil 48 fps).
