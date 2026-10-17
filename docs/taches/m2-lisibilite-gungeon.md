# m2-lisibilite-gungeon — on doit comprendre ce qu'on voit (M2, présentation seule)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Simulation et présentation », « HUD »),
`docs/conventions.md` §15 (HUD), §31 et §55 (feedback), §82 (feedback v3), `games/gungeon/assets/assets.yaml`
(tout est placeholder copié de zombies), `games/gungeon/assets/sprites.ron`, `crates/game/src/character/visuals.rs`,
`world_sprites.rs`, `room_fog.rs`, `scripts/generer-sprites-feedback.py` (M3-T17 : sprites CC0 générés par
script). Branche `m2-lisibilite-gungeon` depuis `origin/main`. Estimation : 2 j. **Bloque la revue humaine
de M2** (constat de William sur les vidéos `gungeon_full_run` du 2026-10-17 : « je ne comprends absolument
rien, qu'est-ce qui est quoi »).

## Contexte

Chaque ennemi de gungeon a la silhouette du zombie de `zombies`, le joueur aussi ; objets, coffres,
présentoirs, portes, portail, sous, clés, bombes, barils, tables, pièges n'ont pas de sprite propre ou
partagent le même. Les projectiles ennemis et alliés se ressemblent. Rien n'indique ce qui est
ramassable, achetable, dangereux, cassable. La revue humaine est impossible tant que ça dure.

Ce n'est **pas** la direction artistique (voie V6, assets définitifs) : c'est un **langage visuel
provisoire lisible**, généré par script (CC0), remplaçable fichier par fichier par V6.

## À faire

1. **Langage visuel par classe**, cohérent dans tout gungeon (et réutilisable par isaac/testbed) :
   - joueur : silhouette unique, contour clair, couleur par handle (0 bleu, 1 vert, 2 jaune, 3 violet) ;
   - ennemis : une silhouette **par famille** (`bullet_kin`, `shotgun_kin`, `lanceur`, `tourelle`
     stationnaire, `veteran`, `bandana`, `fuyard`, boss) — forme ET couleur distinctes, taille
     proportionnelle au collider, orientation vers la cible ; variantes/champions : teinte de M3-T17 ;
   - projectiles : ennemis rouges/orangés, alliés blancs/bleutés, taille = dégât approximatif ;
     télégraphes déjà faits (M2-T19, D67) ;
   - ramassables : **tous distincts** et posés sur un halo pulsant — sous (pièce jaune), clé, bombe,
     cœur/ration, blank, armes au sol (silhouette d'arme), objets passifs (carte/icône avec initiale) ;
   - monde : porte fermée / ouverte / verrouillée (cadenas) / portail (spirale) / coffre fermé-ouvert /
     présentoir avec étiquette de prix / baril (cassable) / table (couverture) / rocher / piques /
     eau-sable-fosse (surfaces de T14d) ;
   - étiquettes **optionnelles** au-dessus des entités (nom de l'ennemi, nom et prix de l'objet),
     touche **F6**, activées par défaut dans `play_scenario --capture` et `make videos`
     (`ALACOD_LABELS=1`) pour que les vidéos de revue s'expliquent d'elles-mêmes.
2. **Légende** : un panneau HUD repliable (touche **F7**, `hud.ron` source `legend`) listant les
   silhouettes/couleurs ; incrusté dans la première seconde de chaque vidéo de revue (`scenario-video`
   : image de légende en tête, comme le commentaire « À regarder »).
3. **Génération** : étendre `scripts/generer-sprites-feedback.py` (ou un `scripts/generer-sprites-gungeon.py`)
   pour produire tous les PNG, enregistrés dans `assets.yaml` (CC0, « placeholder généré, à remplacer
   par V6 »), `sprites.ron` à jour ; `make gen` et le lint des assets verts.
4. **Preuve** : captures `play_scenario --capture` de `gungeon_full_run` à f42 (salle verrouillée), f157
   (clé), f1890 (arène), f2825 (boutique), f2928 (coffre), f3439 (boss), f3489 (portail) avec étiquettes
   : au rapport, et `make videos SCENARIO=gungeon_full_run,gungeon_full_run_four` régénérées ;
   une capture **avant/après** côte à côte pour chaque classe.

## Règles

Présentation seule : rien dans `GgrsSchedule`, aucun composant posé sur une entité rollback qui ne
le soit déjà (`WorldSprite`, `BulletSkinned`, `ChampionTint`, `TearStyled` sont le modèle), tout
dérivé de l'état en `Update`. **Aucune trace ne bouge**, `make check_sim_boundary` vert. Les
étiquettes et la légende sont absentes en headless. Rien de spécifique à gungeon dans le code moteur :
le langage visuel est **données** (`sprites.ron`, `assets.yaml`, `hud.ron`) + systèmes génériques
(étiquettes lisent `CharacterConfig.name`/`ItemDef.name`/prix).

## Vérification

README §4 : `make test_scenarios_par` sans BLESS (aucune trace), `make test_crates`, `make lint` ×5,
`make gen` ×5, fmt, `check_sim_boundary`, `make test_tools`. Rapport
`docs/taches/rapports/m2-lisibilite-gungeon.md` avec les captures avant/après ; ligne au journal ;
`LIVRÉ m2-lisibilite-gungeon`. Relecture indépendante avant merge, puis **William revoit les vidéos**.
