# Rapport m3-t11b-effets (b0)

Conventions : `docs/conventions.md` §76.2. Branche `m3-t11b-effets` depuis `origin/main` `bfc71a02` (déjà à jour à la fin). Données seulement : aucun code moteur.

## 1. Fait
Dix objets, trois personnages (`frere`, `orbe` familiers ; `cible_mordante` de laboratoire), une synergie, deux scénarios écrits ; tableau objet → effet → pool → graine → scénario au §76.2.
- `tresor` (+8, poids 4/2/1) : `frere`, `orbe`, `nuee` (familiers), `crocs` (`OnHit` + `Scaled PerCopy` + `ApplyStatus`), `pouls` (`Tick`), `rancune_vive` (`OnDamageTaken` + `ApplyStatus`), `reliquaire` (`OnPickup` filtré, `GainSoul`), `calice_ame` (actif `Damage(30)`).
- `boss` (+2) : `sang_chaud` (`OnKill`), `calice_ardent` (actif `Rooms(2)`) ; les trois passifs de T8 passent de poids 1 à 2 (peu communs).
- Synergie `garde_rapprochee` (`frere` + `orbe`, +20 % de dégâts), dossier `synergies` déclaré dans `game.ron`.
- Cœurs d'âme : déjà dans `loot/salle.ron` (T3, poids 1) ; `salle.ron` inchangé.
- Preuves : `test:` avec témoin mesuré (`tears_crocs`, `tears_rancune_vive`, `tears_sang_chaud` + `_sans_objet` ; `familiar_frere` ≥ 6 coups contre 5 sans objet ; `FamiliarDistance` pour `orbe` et `nuee`) ; `item_pouls` (`MoveSpeed` 120 → 156,00037), `item_calice_*` (usage, `Damage` 2, `ExtraHealth` 2) ; scénarios écrits `isaac_reliquaire`, `isaac_synergy_garde`.

## 2. Avertissements `make lint GAME=isaac` : 30 → 26
Fermés (4) : « aucun objet à effets dans le pool trésor », « … dans le pool boss », « aucune synergie atteignable », « familiers : 0 sur 2 dans le pool trésor ». Restent : boutique (effets, familiers, larmes, clés, `exclude_held`/`exhaust` d'empilement des objets de larmes), bombes/rochers/piques, salle secrète, champions, curses, accomplissements : T10, T11c, T11d.

## 3. Traces bougées (preuve §5)
Cause unique : le tirage des pools à la génération de l'étage (`items::loot::PoolState`, objet tiré dépend de la taille du pool). Dumps `ALACOD_DUMP_TRACE` `main` (`bfc71a02`, `games/isaac` d'origine) contre branche, `scripts/trace-diff.py`, trois scénarios : `isaac_boots` (trésor `oeil` → `trefle_noir`, boss `griffe_acier` → `sang_chaud`), `grid_treasure_key` (trésor `givre` → `crocs`, boss `griffe_acier` → `sang_chaud`), `generated/isaac/boss_gardien` (boss `reservoir_vital` → `oeil_vif`) : première frame 0, seule ligne différente la ressource `PoolState`.
Bénies une à une : 19 traces écrites à la main (`curse_*` ×4, `devil_*` ×3, `grid_*` ×9, `hit_iframes_isaac`, `isaac_boots`, `isaac_floor_seed_1`) + 2 générées (`boss_gardien`, `room_combat`) + 20 neuves générées + 2 neuves écrites. Attentes recalées (tirage changé) : `grid_floor_seed_23` (`griffe_acier` → `reservoir_vital`), `grid_post_boss` (→ `calice_ardent`, actif), `devil_room_chance` (`PoolSeen` : `longue_vue`, `calice_ardent`). Aucune trace d'un autre jeu n'a bougé.

## 4. Résultats réels
- `make test_scenarios_par SHARDS=3` : 3 tranches ok, 610 scénarios, 249 s (après bénédiction ; avant : 21 traces isaac écrites à la main + attentes ci-dessus seules en échec, aucun autre jeu).
- `make test_crates` : 1 167 passés, 0 échec. fmt : propre.
- `make lint` ×5 : aucune erreur (isaac 26 avertissements, gungeon 1, testbed 107 comme avant). `make gen` ×5 : aucun échec ni trace différente, `git status` propre ensuite.
- p2p `scripts/p2p-stress.sh isaac tests/scenarios/p2p_smoke.ron 3000` (MATCHBOX_PORT=3536, projet b0) : **2 001 frames resimulées** (client 0), 0 desync, 2 999 lignes identiques. Exécuté avant le correctif de relecture (données de `reliquaire` seulement, hors partie p2p du script).
- Graines : balayage 1..125 (`alacod-sim`), les 10 objets sortent ; `sang_chaud` (graine 1) et `calice_ardent` (4) : butin du boss ramassé en jeu, `familiar_spawn`/`familiar_shot`/`familiar_block` journalisés pour `frere` (12) et `orbe` (37).

## 5. Relecture indépendante (reviewer)
Un défaut P2 : `reliquaire` sans filtre `item` rendait 4 points d'âme à **chaque** ramassage suivant. Corrigé (`item: Some("reliquaire")`), `isaac_reliquaire` ramasse ensuite `pouls` et vérifie que `ExtraHealth` reste à 4 ; trace rebénie. Reste RAS.

## 6. Dettes
- Pas de boutique (T11d) ; pas de transformation (Q10 non retenue) ; la synergie n'est jouée qu'en scénario (le bot ne rouvre pas le trésor du 2e étage : deux objets du même pool ne sortent pas dans une partie).
- `OnPickup` ne se déclenche pas pour un objet de départ (`PlayerScript::items`) : `reliquaire` n'a que `isaac_reliquaire` pour preuve.
- `MoveSpeed` en unités/s : seuil de `pouls` mesuré, fragile à un changement de la vitesse de base.
- Pas de sprite d'objet ni de familier propre. Bench non lancé (machine partagée).
