# m1-v3-generateur-v1 — générateur v1 : gabarits par ennemi, placement scripté (T1.13, voie V3)

Lire d'abord `docs/taches/README.md` (agent **local**, branche `m1-v3-generateur-v1` créée depuis
`origin/main` ou la tête livrée de la tâche précédente de l'agent, même target). **Prérequis** : T1.4
behaviors (mergée : `EnemyState`, `EnemyDistance`, `EnemyContactBefore`), T1.5 variantes (mergée :
champ `variant`), T2.10 générateur d'armes (`crates/scenario/src/generate.rs`, `make gen`). Les
statuts (T1.3, c3) ne sont **pas** dans main : les gabarits par statut sont hors périmètre. Petite
tâche (3 j).

## Contexte

Plan §6 : « T1.13 Générateur v1 — gabarits par ennemi (seul contre joueur immobile, puis mobile) et
par statut ; `test:` sur les ennemis. » Ce qui existe : `generate::Template::WeaponOnTarget`
(une arme, le joueur vise `target` dans `arena.ldtk`), `WeaponTest { frames, min_hits, max_hits,
expect }` dans le RON de l'arme, `generate_game(game_dir)` → `tests/scenarios/generated/<jeu>/
weapon_<id>.ron`, `make gen GAME=<jeu>` (régénère et exige « sans modification »), lint
`lint_weapon_test`. Un scénario **ne peut pas faire apparaître de personnage** : T1.4 a dû créer
une arène LDtk par behavior, T1.5 une arène à quatre spawns. Les power-ups ont déjà un placement
scripté (`Scenario.powerups: [(id, x, y, at_frame)]`, `RollbackSystemSet::Effects`, §14).

## Décisions (proposées par l'orchestrateur ; l'agent confirme ou amende en dix lignes avant de coder)

1. **Placement scripté de personnage** : `Scenario.characters: Vec<CharacterPlacement { character,
   x, y, at_frame, variant: Option<String>, team: Option<Team> }>` (même famille que `powerups`),
   appliqué dans `GgrsSchedule` à `frame == at_frame` (rollback-safe comme un spawn de vague),
   dans l'ordre de déclaration, par `spawn_enemy` (ou le chemin des `CharacterSpawn` LDtk pour les
   non-ennemis) : net id alloué à cette frame, variante imposée possible. Aucune trace existante ne
   change (champ vide = rien). Le runner le préserve au réenregistrement. **C'est l'outil qui
   remplace « une arène par ennemi »** ; les arènes de T1.4/T1.5 restent telles quelles.
2. **`test:` sur un personnage** (`CharacterConfig.test: Option<CharacterTest>`, miroir léger dans
   `content::registry` comme `WeaponTestRange`) : `( frames: 600, still: true, moving: true,
   expect_still: [...], expect_moving: [...] )`. Attentes par défaut quand la liste est vide :
   gabarit **immobile** → `Health { max: < santé de départ, at_frame: frames }` (l'ennemi a infligé
   des dégâts) et `EnemyNeverInWall` ; gabarit **mobile** → `PlayerAlive(frames)` et
   `EnemyNeverInWall`. Les attentes typées sont celles de `combat::weapons::expectations`
   (réexportées), ajoutées telles quelles.
3. **Gabarits** : `Template::EnemyVsStillPlayer` (joueur en (0, 0) de `arena.ldtk` avec son arme de
   départ, **aucun input** ; ennemi placé à 200 unités à droite à `at_frame: 1`) et
   `Template::EnemyVsMovingPlayer` (même placement ; le joueur marche en carré : 90 frames par côté,
   répété, sans tirer). Fichiers `tests/scenarios/generated/<jeu>/enemy_<id>_still.ron` et
   `enemy_<id>_moving.ron`, en-tête commenté comme `header_comment`. **Opt-in** : seuls les
   personnages qui déclarent `test:` sont générés (les traces restent en nombre maîtrisé).
4. **Contenu** : testbed → `test:` sur `grunt`, `archer`, `charger`, `kiter`, `turret`, `breacher`
   (12 scénarios générés) ; zombies → `test:` sur chaque ennemi de vague (lister dans le rapport ;
   2 scénarios par ennemi). Attentes par défaut partout sauf besoin avéré (ex. `turret` immobile :
   `EnemyDistance` constante). Toutes les traces générées : **bless orchestrateur**.
5. **Lint** (`lint_character_test`) : `frames > 0`, au moins un gabarit actif, `expect_*`
   seulement si le gabarit est actif ; `CharacterPlacement` : personnage connu, `variant` connu,
   `at_frame < frames` (runner). Fixtures.
6. **`make gen`** : couvre armes + personnages ; `make gen GAME=testbed` et `GAME=zombies` sans
   modification après génération ; `scripts/scenario-metrics.py` inchangé.
7. **Hors périmètre** : gabarits par statut (T1.3 absente), vidéos des gabarits, génération de
   cartes, bots dans les gabarits.

## Traces attendues

Aucune trace existante ne change (`characters` vide partout ; `test:` n'a aucun effet en jeu).
Vérifiable par `make test_scenarios` vert sans bless hors des scénarios générés nouveaux.

## Règles

Deux compilations au plus ; purge du target + point d'état après chaque suite (README §1) ; aucune
trace bénie par l'agent ; `docs/conventions.md` : **uniquement** §28 « Générateur v1 et placement
scripté » (+ une ligne au §3 pour `test:` des personnages) ; `CLAUDE.md` : réglage `characters` dans
la liste des scénarios ; `docs/taches.md` : ne pas toucher. Merger `origin/main` juste avant de
livrer ; conflits : garder les deux.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires : placement à la frame exacte et ordre des net ids, variante imposée, deux
   gabarits construits depuis un `CharacterTest` (attentes par défaut et explicites), lint.
2. Scénarios générés verts pour les six ennemis du testbed et les ennemis zombies ; **tous les
   scénarios existants verts sans trace modifiée** ; `make gen` des deux jeux sans modification.
3. Suite des crates verte, `make lint` (deux jeux), `make fmt`, scripts ; exemples racine compilés.
4. §28 écrit ; rapport honnête avec point d'état et la liste des scénarios générés.

## Livrer

Rapport `docs/taches/rapports/m1-v3-generateur-v1.md` sur la branche (README §7 ; sha de tête, base
`origin/main`). `git push -u origin m1-v3-generateur-v1`, puis `SendMessage` à `orch` :
`LIVRÉ m1-v3-generateur-v1 <sha> : <une ligne>` (ou `BLOQUÉ …`). Ne merge pas, ne bénis pas.
