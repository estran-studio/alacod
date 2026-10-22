# polissage-throne-complet — throne visuel, sons, map (S2-10, S2-11, S2-12)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1, §3), inventaire §3-§5, §9-§12, `docs/taches/polissage-pack-assets.md`, `docs/taches/polissage-sons-evenements.md`, `docs/conventions.md` §83, `CLAUDE.md` (déterminisme, traces).
`docs/taches/polissage-throne-jouable.md` (S2-7), `games/throne/assets/game.ron:1-4`, `caves/*.ron`, `floors/run.ron`.
Branche `polissage-throne-complet` depuis `origin/main`. Estimation : 3 + 1 + 2 = 6 j. Voies : v1 (visuel), b1 (sons), b6 (map).
Clone : throne.

## Contexte

13 armes, 14 personnages, 1 boss ; sprites copiés du testbed (placeholders) ; pas de `wall_art` ; cavernes sans inventaire de
salles (nombre par étage **non trouvé**) ; aucune campagne chiffrée (S2-7 la produit).

## À faire

1. **Audit (J1)** : classes × état, événements × sons, cavernes : nombre de salles par étage, variété, spawners, rythme.
2. **Visuel (S2-10)** : pack S-0 ; `wall_art` de throne ; personnages animés, tourelle (avec S2-7), boss.
3. **Sons (S2-11)** : 13 armes par familles, boss, mutation, portail, étage.
4. **Map (S2-12)** : corriger les défauts relevés par l'audit (variété, couloirs morts, spawners) sans mécanique nouvelle.

## Acceptation

- Tableaux avant/après ; vidéo ; traces rebénies avec preuve §5 pour S2-12 seulement.
- Rapport `docs/taches/rapports/polissage-throne-complet.md`. **À demander à William** : le pack (S-0), essai 30 min.
