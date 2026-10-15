# Rapport m2-t16-bot-explorateur — bot explorateur d'étages (M2, V3, chantier K2 v1)

Branche `m2-t16-bot-explorateur` (WIP d'un premier agent repris, `main` fusionné). Conventions : §47.

## Livré

- **Profil `explorateur`** (`bots::explorer`, `BotProfile::Explorateur` en fin d'enum) : combat de `prudent`
  contre les seuls ennemis éveillés, blank sur balle ennemie à moins de 48 px, exploration par le but le plus
  proche **par le chemin** (`BotNavigation::costs_from`, ordre coût / objets avant salles / `GgrsNetId` ou id de
  salle), boss en dernier (`--boss-direct` : tout droit), portail ensuite. Aucune mémoire : « visitée » se lit
  dans `RoomState`.
- **`alacod-sim`** : `--boss-direct`, JSON enrichi (`outcome`, `rooms` par étage), résumé par issue en fin de
  sortie. `--until-floor` existait ; il compte l'index 0-based : **atteindre l'étage 2 = `--until-floor 1`**
  (la fiche dit `2` : avec la séquence `etages_court_deux` l'index 2 n'existe pas, la victoire termine avant).
- **Scénarios** `bot_explorer_floor` (1 bot) et `bot_explorer_two` (2 bots), `etages_court_deux`, graine 1,
  sans input scripté ; traces créées ; budgets 25 fps (mesuré 38). `gungeon_floor_seed_*` inchangés.
- Corrections du WIP : collision de nom `outcome` (fonction / variable) dans `alacod-sim`, fixture de test du
  champ de coûts trop petite (cellules de 8 px).
- Doc : conventions §47, D59 mise à jour (`dettes.md`).

## Campagne (`gungeon`, `etages_court_deux`, graines 1..200, `--until-floor 1`, lots de 10 poussés un à un)

JSON par lot : `docs/taches/rapports/m2-t16-campagne/b{1,2}_*.json`.

| Bots | Étage 2 atteint | Morts | Soft-lock | Desync | Frames (objectif) min / méd. / max |
|---|---|---|---|---|---|
| 1 | 177 / 200 | 23 | 0 | 0 | 653 / 764 / 825 |
| 2 | 200 / 200 | 0 | 0 | 0 | 469 / 614 / 679 |

Échecs à 1 bot, classés : **23 morts, toutes à l'étage 0, toutes à la frame 591**, face au boss (`gatling`,
3 dégâts par balle en rafale) après avoir nettoyé la salle de combat. Graines : 5 8 10 15 30 32 49 52 54 60 62 82
84 104 106 126 128 148 150 170 178 180 200. Cause : puissance de feu et esquive d'un seul `prudent` contre le
boss (équilibrage, hors périmètre), pas la navigation : 200/200 graines ont visité et traversé les 3 salles de
l'étage 0 (`rooms.visited == total`), et les salles verrouillantes sont toutes `Cleared` pour les 177 réussites.
À 2 bots, tout passe. Objectif « 200 graines sans soft-lock ni desync » : **atteint** (0 et 0 dans les deux
campagnes) ; les morts solo sont classées ci-dessus. Seule la séquence `etages_court_deux` est couverte
(les grammaires `etage` et `grand` ne le sont pas).

## Vérifications

- `make test_scenarios` : suite complète rejouée sans `BLESS` scénario par scénario (232 noms distincts sur
  239 fichiers, conteneur cloud) : **0 échec**, aucune trace existante modifiée (`git status` propre hors
  les 2 nouvelles traces).
- `make test_crates` : exit 0 (42 binaires de test verts, dont `bots` 68).
- `make lint` ×4 : aucune erreur. `cargo fmt --all -- --check` propre. `check-rollback-registration` OK,
  `check-forbidden` : 4 avertissements préexistants (HashSet ×3, rand ×1, Instant ×2 : inchangés, aucun ajouté).
- `make gen` ×4 : exit 0, aucun fichier modifié.
- **Non vérifié ici** : p2p à deux clients (pas de Docker ni d'affichage), bench strict au calme, vidéos.
