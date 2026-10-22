# polissage-isaac-complet — isaac visuel, sons, map et équilibrage (S2-4, S2-5, S2-6)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1, §3), inventaire §3-§5, §9-§12, `docs/taches/polissage-pack-assets.md`, `docs/taches/polissage-sons-evenements.md`, `docs/conventions.md` §83, `CLAUDE.md` (déterminisme, traces).
Rapports `docs/taches/rapports/m3-salles-murs-isaac.md`, `m3-t18-integration-sortie.md`, dettes D93, D119-D125, Q14.
Branche `polissage-isaac-complet` depuis `origin/main`. Estimation : 3,5 + 1 + 3,5 = 8 j. Voies : v1 (visuel, Mac), b0 (sons),
la première voie libre (map et équilibrage, S2-6). Clone : isaac.

## Contexte

100 objets, 9 boss, 23 gabarits ; personnages, ennemis et boss = tuiles Kenney de gungeon ; sol uni `#7B6A5F`, pas de
pointeur de visée (D123) ; 138/60 victoires/pertes à 1 bot, 183/14 à 2 ; D93 (armure), D125 (caméra de salle, mur fissuré non
vus avec rendu), D121/D124 (soft-locks), D119 (piques).

## À faire

1. **Audit chiffré (J1)** : classes × état, événements × sons (bombe, objets, 9 boss, clés, sous), props par étage.
2. **Visuel (S2-4)** : pack de S-0 pour personnages, ennemis, boss, projectiles (larmes), sol, pointeur de visée ; D123.
3. **Sons (S2-5)** : bombe, ramassages, objets actifs, boss ×9, clés, sous, explosion.
4. **Map et équilibrage (S2-6)** : D93 (parties gagnées par l'armure), props, D125 **vu avec rendu** (Mac), décision Q14
   (William) ; campagne 1 et 2 bots ; les soft-locks de bot (D121/D124) restent en P2 sauf s'ils bloquent la campagne.

## Acceptation

- Tableaux avant/après ; captures de D125 ; campagne avant/après ; traces rebénies avec preuve §5 pour S2-6 seulement.
- Rapport `docs/taches/rapports/polissage-isaac-complet.md`. **À demander à William** : le pack (S-0), Q14, essai 30 min.
