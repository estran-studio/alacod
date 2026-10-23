# k5-5-bac-a-sable — bac à sable jouable par profil, pour tester et régler à la voix (K5, demande de William du 2026-10-23)

Lire d'abord `docs/taches/README.md` (§1, §4, §5, §7), `CLAUDE.md` (« Headless », « Scénarios », « Jouer et enregistrer », « Simulation et
présentation »), `docs/editeur-vocal.md` en entier, `docs/conventions.md` §28 (placements) et §84 (éditeur par l'IA), les rapports
`docs/taches/rapports/k5-1-partie-pilotable.md`, `k5-2-commandes-spawn.md`, `k5-3-apps-de-test.md`. Branche `k5-5-bac-a-sable` depuis `origin/main`.
Estimation : 1,5 à 2 j.

## Besoin (William, mot pour mot)

« Je ne veux pas nécessairement jouer le jeu au complet. Pour zombies, je charge le jeu mais dans une map et un mode de jeu différents, genre
character_tester et map_tester. Je veux jouer et démarrer avec des profils : telle arme, etc., puis faire spawner des zombies et autres pour tester
et régler. Ce n'est pas vraiment possible pendant une partie quand j'ai besoin de moins d'action. »

Aujourd'hui : `make editor` joue la partie normale (vagues, étages) ; `make character_tester` est limité à `zombies`, avec vagues. Aucun des deux ne
part d'un **profil** (personnage, armes, objets, argent, modificateurs) ni d'une **arène calme**.

## Décision de conception (orchestrateur, à défaut d'avis contraire)

Un **profil de bac à sable est un scénario** (même format RON, même code de mise en place : `crates/scenario/src/runner.rs`,
`apply_player_overrides`, `characters`, `items`, `chests`, `shop`, `weapon_overrides`, `wave_overrides`, `mode`, `map`) **joué au clavier**,
sans fin, en mode éditeur. On ne crée pas un second format ni un second chemin de mise en place.

1. **`play_scenario --live`** (`crates/scenario/src/bin/play_scenario.rs`) : rendu fenêtré ; les joueurs **locaux** lisent le clavier et la
   manette (même source que le jeu : `DeviceInputs`, `collect_device_inputs`, E4) au lieu de leurs `inputs` ; `frames` ignoré (partie sans fin) ;
   attentes non évaluées ; **mode éditeur** (`CoreSetupConfig::editor`, `check_distance=0`, remote BRP sur 15702) : toutes les méthodes
   `alacod/*` de K5-1/K5-2 marchent (`spawn --ahead`, `modifier`, `restart`, `reload`). `restart` rejoue la mise en place du profil.
2. **Profils** : `games/<jeu>/assets/sandbox/<nom>.ron`, format `Scenario`, déclarés et lintés comme les scénarios (aucun nouveau kind si le
   format suffit ; sinon le dire). Par défaut un profil joue l'arène de test du jeu (`generate_template.map` : `gabarit_armes` pour gungeon, isaac,
   throne ; une carte calme pour zombies et testbed, à choisir et justifier), `mode: Sandbox`, vagues coupées (`wave_overrides` ou `waves_enabled`),
   aucun ennemi au départ sauf ceux du profil. Livrer **au moins deux profils par jeu** (zombies, gungeon, isaac, throne) : `arene` (personnage par
   défaut, arme de départ) et un profil chargé (plusieurs armes ou objets, argent, un ou deux ennemis posés), valeurs lues dans les RON du jeu.
3. **`make sandbox GAME=<jeu> [PROFIL=<nom>]`** (défaut `arene`) ; `make character_tester` devient un alias de `make sandbox GAME=zombies` si
   l'équivalence est complète (sinon le garder et dire pourquoi). Le « Map Tester » reste `make editor` + `restart --seed N`.
4. **Voix** : `docs/editeur-vocal.md` §1 (tableau des apps : la ligne « Bac à sable »), §2.1 (« donne-moi le fusil », « mets-moi 500 sous » : par
   quelle méthode — une méthode `alacod/*` nouvelle seulement si aucune existante ne suffit ; sinon édition du profil + `restart`), et la phrase
   « change de profil » (relance par `make sandbox PROFIL=…`, dite comme limite si elle exige une relance du binaire).

## Règles

- Aucune trace de scénario existante ne bouge (`BLESS` interdit) ; `--live` n'est jamais pris par la suite de tests.
- Rien de neuf dans `GgrsSchedule` sauf nécessité prouvée ; la garde `hot_edit_allowed` de K5-1 s'applique.
- Pas de second format de profil, pas de second chemin de mise en place.

## Vérification

`make test_crates`, `make test_scenarios_par SHARDS=3` sans BLESS, lint ×5, fmt, `check-forbidden` ; preuve headless de la mise en place de chaque
profil (un test qui charge chaque `sandbox/*.ron` comme scénario de quelques frames : zéro erreur de contenu, joueur vivant, aucune vague) ; preuve
`--live` sans fenêtre impossible sur le VPS : donner à William la commande exacte et ce qu'il doit voir (rapport §« Essai »).

## Livrables

Rapport `docs/taches/rapports/k5-5-bac-a-sable.md`, `docs/editeur-vocal.md`, `docs/jouer.md` (« Éditer en jouant »), `docs/conventions.md` §84,
`CLAUDE.md` (« Jouer et enregistrer »), journal `docs/taches.md`, commit `LIVRÉ k5-5-bac-a-sable`, push de la branche. Dettes : à partir de **D126**.
