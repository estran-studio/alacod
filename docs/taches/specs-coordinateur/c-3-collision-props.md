# Brief C-3 — un décor solide bloque les corps (lot de correction du verdict de William du 23 nov)

Objectif studio : O2 au service d'O3 ; jalon **D2 dim 29 nov** (l'extérieur : caisses, rochers, caisse de la clé). Lot de correction : passe avant X-6e (REGLES § 6).
Simulation : `anthropic/claude-sonnet-5-5`, `--agent omp`. Français partout.

## Le défaut
William, 23 nov, 1837 `3a29299` sur engine `5095dfa9` : « Il n'y a pas de collision des personnages avec les objets comme les crates sur la map. » Les balles s'y arrêtent.
Relevé du coordinateur (`~/orca/runs/alacod/releve-verdict-2-4-2026-11-23.md`) :
- un prop naît sur `enemy_layer` (`crates/game/src/character/enemy/create.rs` ~97) ;
- joueur : `check_hard_collision` (`crates/game/src/character/player/input.rs:698-699`) saute `enemy_layer` : ralentissement seul ;
- ennemis : `check_wall_collision` (`crates/game/src/character/enemy/ai/pathing.rs` ~764-803) ne bloque que murs + `Obstacle { blocks_movement }` ; prop dans la séparation seulement (`pathing.rs:307-320`) ;
- champ de flux : `rebuild_blocked_cells` (`navigation.rs:1009-1040`) ignore les props ;
- aucun champ de `Character` ne rend un décor solide.

## La correction attendue
1. **Test d'abord** : scénarios testbed rouges sur `main`, puis verts. À un joueur (pousse contre un prop solide : `PlayerPosition` bornée au bord du prop à plusieurs frames, aucun dépassement), à **deux** joueurs (un de chaque côté), un ennemi (`EnemyPosition`/`EnemyDistance` : ne traverse pas, contourne par le champ de flux), un prop `Destructible` qui **cesse de bloquer** à sa mort (`despawn_rollback`, cases du champ de flux libérées), un **témoin** (prop non solide : rien ne change, `prop_repels_enemy` reste tel quel).
2. **Opt-in** : un champ de donnée du personnage (nom au choix du worker, ex. `solid: true`, valide seulement avec `prop: true` : lint sinon), absent par défaut ; s'il touche un composant sous checksum, contribution **neutre** quand absent (Hash/Debug manuels, règles du `CLAUDE.md`). **Aucune trace existante ne bouge** (aucun BLESS) : c'est la condition.
3. Joueur : collision **dure avec glissement**, comme un mur (même chemin que `Wall`), pas le ralentissement d'`enemy_layer`. Ennemis : `check_wall_collision` + cases bloquées du champ de flux (mise à jour quand un prop solide naît ou meurt, déclencheur des cases bloquées déjà existant). Balles : inchangées.
4. Déterminisme : `Fixed`, ordre `GgrsNetId`, `BTreeSet`, pas de `f32` ; enregistrement rollback éventuel dans `CoreSimPlugin` (variante neutre pour un type nouveau).
5. Docs : § 132 de `docs/conventions.md`, ligne `CLAUDE.md` (attentes, une ligne), `alacod describe Character` à jour ; dettes à partir de **D240** (X-6e garde D234–D239).

## Preuve (avant le merge)
- Tableau rouge → vert des scénarios neufs (1 et 2 joueurs, ennemi, destructible, témoin) et des scénarios des familles touchées (`prop_*`, `couvert_*`, `wisp_respawn_prop`, `repli_couvert_*`, `zigzag_*`) : traces **inchangées**.
- `check_sim_boundary`, `make schedule_order_diff BASE=a0eb6b62` (si un système ou un type rollback est ajouté).
- Capture de 20 à 60 s du vrai jeu natif faite par l'agent : le testbed (bac à sable, profil à props solides) où le joueur bute et glisse contre une caisse, un ennemi la contourne. La preuve dans 1837 (`collision_decor` à 1 et 2 joueurs, `maps/exterieur.ldtk`, critère d'acceptation du coordinateur 1837, lot V-A) se fait au réépinglage côté 1837.
- Contrôle rapide **découpé** (`specs-coordinateur/scripts/p-1d-check-rapide-decoupe.sh`, `LOGS`, `CARGO_TARGET_DIR=~/orca/targets/alacod-engine`), journal dans `~/orca/runs/alacod/logs/`.
- Rapport `docs/taches/rapports/c-3-collision-props.md`, trois lignes en tête (existait / manquait / ajouté).

## Machine (règles valables)
- Tout cargo : `nice -n 19 env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 CARGO_TARGET_DIR=$HOME/orca/targets/alacod-engine`, jamais de profil debug, un seul cargo à la fois.
- **Feu du coordinateur avant chaque cargo lourd** (build à froid, scénarios, contrôle rapide, release) : un seul cargo lourd sur la machine, coordinateurs alacod et 1837 réunis ; demande-le par `ask`.
- Disque : plancher 30 Go (`df -h /System/Volumes/Data`), arrêt sous 28 et prévenir ; `~/orca/targets/1837` intouchable ; `scripts/target-sweep.sh` après le contrôle.
- Tu ne merges pas ; tu pousses ta branche et tu rends `worker_done` avec SHA, journal, rapport.
