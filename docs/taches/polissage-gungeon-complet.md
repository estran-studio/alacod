# polissage-gungeon-complet — gungeon visuel, sons, map et équilibrage (S2-1, S2-2, S2-3)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1, §3), inventaire §3-§5, §9-§12, `docs/taches/polissage-pack-assets.md`, `docs/taches/polissage-sons-evenements.md`, `docs/conventions.md` §83, `CLAUDE.md` (déterminisme, traces).
Rapports `docs/taches/rapports/m2-salles-murs-gungeon.md`, `m2-t14d-gabarits.md`, `bot-explorateur-v2.md`, dettes D64, D118.
Branche `polissage-gungeon-complet` depuis `origin/main`. Estimation : 3 + 1 + 2,5 = 6,5 j. Voie b4 (Mac). Clone : gungeon.

## Contexte (inventaire)

21 armes, 20 objets, 11 personnages, 1 boss, 15 gabarits + hub ; silhouettes générées CC0 sur placeholders ; `wall_art`,
`player_colors`, `legend` présents ; sons : trois familles de tir, pas de boss/achat/coffre ; campagne 200 graines :
27 % de morts à 1 bot, 3,5 % à 2 bots (D64 accepté pour sortir de M2).

## À faire

1. **Audit chiffré d'abord (J1)** : tableau classe × état × animations (visuel), événement × son, gabarits (spawners, couvertures,
   salles vides après agrandissement, D64/D118) ; réestimer.
2. **Visuel (S2-1)** : intégrer le pack de S-0 (dès son choix) : personnages animés idle/marche/attaque/mort, boss, sol,
   projectiles, objets ; garder `wall_art`/légende fonctionnels.
3. **Sons (S2-2)** : familles pour les 21 armes, boss, hub, achat, coffre, blank, salles (clés de S1-7).
4. **Map et équilibrage (S2-3)** : réduire la létalité des salles `combat` à 1 joueur, redonner une menace aux salles vides,
   régler armes et boss ; campagne 200 graines avant/après ; aucune mécanique nouvelle.
5. Vidéo (Mac) et page de revue.

## Acceptation

- Tableaux d'audit avant/après sans case « emprunté » sur les classes principales ; aucune case sonore vide sans repli justifié.
- Campagne avant/après publiée ; traces rebénies **avec preuve §5** pour S2-3 seulement ; visuel et sons : traces inchangées.
- Rapport `docs/taches/rapports/polissage-gungeon-complet.md`. **À demander à William** : le pack (S-0), son essai de 30 min.
