# C-8 — Clé au sol visible

**Prêt pour la partie de William** (pas livré avant son verdict) — **code écrit, jamais compilé ni joué** : aucun cargo cette nuit (suite de nuit sur le VPS) ; preuves, traces neuves et capture à produire avec le feu (voir « Reste à prouver »).
- Jouer (après le feu) : `make play_scenario SCENARIO=cle_visible_deux_joueurs` (image ×2) puis `cle_visible_un_joueur` (marqueur de repli).
- Capture : à produire (`play_scenario --capture`, rendu hors écran sous Xvfb sur le VPS si possible).

- **Existait** : `spawn_key_pickup` (`crates/game/src/keys.rs`) ne posait aucun visuel ; aucun champ de contenu ne donnait une image à une clé au sol.
- **Manquait** : une référence visuelle en données par id de clé d'équipe, un repli lisible, un contrôle qui échoue si une clé vivante n'a pas de visuel, le lint.
- **Ajouté** : entrée `key/<id>` puis `key` de `sprites.ron` + `display_scale: Some(f)` (§ 134) ; `game::key_visuals` (décision hors rendu `KeyVisual`) ; `world_sprites::attach_key_visuals` / `hide_despawned_keys` (présentation) ; marqueur par défaut ; invariant `cle_visible` ; avertissement `KeyWithoutVisual` et erreur `display_scale` ; testbed `cle_image` + 2 scénarios ; 4 fixtures de lint.

## Conception (écarts à noter)

- **Fichier de données** : la table `SpriteSheet` existante (`sprites/sprites.ron`, déjà « comme les autres objets au sol », avec sa surcharge locale) plutôt qu'un nouveau kind : pas de nouveau chargeur ni de nouveau fichier de manifeste ; `alacod describe SpriteSheet` montre `display_scale` par le schéma (`SpriteSheetEntrySchema`). Le facteur multiplie l'échelle de la feuille (`SpriteSheetConfig::scale`), il ne la remplace pas.
- **Contrôle de présentation** : l'invariant du runner (`cle_visible`) lit `KeyVisual`, la décision de présentation prise hors rendu (même régime que `ModelPlayback` et `animation_vivante`, § 122) : il est donc vrai en headless et dans toute suite, à un et à deux joueurs. Il ne voit pas le `Sprite` lui-même (absent en headless) : le dessin est couvert par les tests unitaires de `world_sprites` (marqueur, échelle) et par la capture.
- **Disparition** : `despawn_rollback` laisse la clé désactivée jusqu'à la confirmation de sa frame ; `hide_despawned_keys` cache ses sprites enfants immédiatement (même motif que les bombes et rochers).

## Reste à prouver (avec le feu)

1. Rouge puis vert : le commit `65b38aaa` (invariant + scénarios, sans décision) doit faire échouer `cle_visible_*`, `key_single_pickup`, `key_carrier_coop` (« invariant cle_visible ») ; le commit suivant les rend verts.
2. Traces neuves seulement : `cle_visible_un_joueur`, `cle_visible_deux_joueurs` (`BLESS`) ; `git diff --stat origin/main` : aucune `.trace` existante modifiée. Les expectations de `cle_visible_un_joueur` (joueur 0 seul, script de `key_single_pickup`) sont à confirmer au premier passage.
3. `make check_sim_boundary`, `cargo test -p content -p game` (tests `key_visuals`, `key_tests`, `c_8_cle_visible_fixtures`), contrôle rapide découpé, capture.
