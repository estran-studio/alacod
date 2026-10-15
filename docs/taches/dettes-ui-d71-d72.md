# dettes-ui-d71-d72 — écran de mutation en `bevy_hui`, mémoire de visite de la minicarte

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« HUD », « Simulation et présentation »), les lignes
D71 et D72 de `docs/taches/dettes.md`, le rapport `docs/taches/rapports/m2-t18-hud-gungeon.md`,
`docs/conventions.md` §32 et §56, `crates/game/src/ui/` (`screens/`, `mutation_screen.rs`,
`hud_model.rs`). Branche `dettes-ui-d71-d72` depuis `origin/main`. Estimation : 1,5 j. Un commit par
dette. Compléter §56.

## À faire

1. **D71, écran de mutation de throne en `bevy_hui`** (décision de William du 2026-10-15 : `bevy_hui`
   pour les écrans).
   - Garder `MutationScreenView` et le protocole `ChoiceA/B/C` : les inputs de choix restent ceux de la
     simulation.
   - Écrire un gabarit de cartes `.html` et un adaptateur sur le modèle de la pause et de l'inventaire.
   - Retirer ensuite `mutation_screen.ron` et son schéma de lint.
   - Aucune trace ne bouge (présentation seule). Les attentes `HudText` et les scénarios `throne_mutation_*`
     restent verts.
2. **D72, mémoire de visite de la minicarte** : une salle non combattante (boutique, récompense)
   visitée puis quittée doit être marquée visitée.
   - Préférer une **mémoire dérivée en présentation** : l'ensemble des salles où un joueur local est
     entré, mis à jour dans `Update` depuis la position des joueurs et `RoomStates`. Aucune trace ne
     bouge.
   - Un état de salle rollback (`Visited`, variante neutre) seulement si la présentation ne suffit pas
     (par exemple, un restart doit le garder) : alors preuve §5 et cause dans le commit.
   - Attente `HudText` de la minicarte dans un scénario gungeon qui passe par la boutique
     (`shop_seeded`).

## Règles

Rien dans `GgrsSchedule` sauf le cas justifié de D72. Les crates de simulation n'importent rien de
rendu (`make check_sim_boundary`). `cargo fmt --all -- --check`.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1).

Pas de GPU ni de Docker :
- captures en rendu logiciel, comme M2-T18 ;
- dire au rapport les frames à regarder.

Pousser la branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et rejouer la
suite juste avant le commit `LIVRÉ dettes-ui-d71-d72`, avec le rapport
`docs/taches/rapports/dettes-ui-d71-d72.md`.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
scripts, `make check_sim_boundary`).
