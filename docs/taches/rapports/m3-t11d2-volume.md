# Rapport m3-t11d2-volume (b1)

Conventions : `docs/conventions.md` §76.5. Branche `m3-t11d2-volume` depuis `origin/main` `8e25731d` (T11d1 mergée), fusionnée avec `origin/main` `5fa55fcb` (D69 étape 2, E1 et E2) en fin de tâche. **Données seulement** : aucun fichier Rust ni `crates/` modifié (`git diff $(git merge-base HEAD origin/main) --name-only` ne sort que `games/isaac/`, `tests/scenarios/`, `docs/`).
Décision de William du 2026-10-18 : le volume de Q1 reste le critère de sortie de M3 ; T11d est coupé en deux, ceci est **T11d2** ; **pas d'équilibrage** (victoires et morts en information, rien n'est réglé pour une fourchette).

## 1. Volume atteint par catégorie, contre Q1

| Catégorie | Avant | Après | Cible Q1 |
|---|---|---|---|
| Objets (fichiers `objets/*.ron`) | 49 | **100** (80 passifs, 11 actifs, 9 consommables) | 100 |
| dont objets qui modifient les larmes (tag `larmes`) | 19 | **37** (dont 23 à `projectile`) | ≥ 30 |
| dont familiers ou orbitaux | 3 | **9** | 8 |
| dont actifs | 2 | **11** | 10 |
| Ennemis (`characters/`, hors joueur, familiers et laboratoire) | 7 | **10** (`araignee`, `follet`, `arroseur`) | 10 |
| dont ennemis à champions | 3 | 3 | 3 |
| Boss à phases | 1 | **2** (`colosse`) | 2 |
| Gabarits | 20 | 20 | 20 |
| `alacod lint games/isaac` | 0 / 0 | **0 erreur, 0 avertissement** (hors `test_only`) | 0 |

Aucune mécanique du vocabulaire n'a été codée : ce qui manque est en dettes D112 à D115 (§ 8).

## 2. Les 51 nouveaux objets

Rareté C / U / R = Common / Uncommon / Rare ; poids de pool 4 / 2 / 1 par rareté. `Scénarios` : générés (`item_*` toujours, `tears_*` / `familiar_*` avec témoin `_sans_objet`) et écrits à la main (`tests/scenarios/isaac_*.ron`). Tous sont atteignables dans la séquence par défaut (lint : « aucune source » : 0).
Les seuils mesurés sont dans le commentaire de chaque `.ron`. Les trois consommables n'ont pas de `test:` (`ItemHeld` refuse un consommable, comme pour `coeur` et `ration`) : `isaac_consommables_lot`. Les quatorze passifs de stats de T8 et T10 (`bague`, `cornes_noires`, `couronne`, `griffe_acier`, `lunette_longue`, `medaille`, `oeil_vif`, `pacte_rouge`, `pile_rapide`, `queue_fourchue`, `regard_maudit`, `reservoir_vital`, `semelles_lestees`, `trefle_chat`) n'avaient pas de `test:` : ils ont maintenant `item_<id>`.

### Larmes (17)

| Objet | Rareté | Pools | Effet | Scénarios |
|---|---|---|---|---|
| `aiguille` | R | boutique, tresor | les larmes percent trois personnages (`Pierce(3)`) par exemplaire, deux de plus que `epine`. | item_aiguille, tears_aiguille |
| `billard` | U | boutique, tresor | les larmes rebondissent quatre fois sur les murs (`Bounce(4)`), deux de plus que `ricochet`. | item_billard, tears_billard |
| `lynx` | R | boutique, tresor | les larmes se guident fortement vers l'ennemi le plus proche (`Homing("0.3")`, trois fois `oeil`). | item_lynx, tears_lynx |
| `bifurque` | U | boutique, tresor | à l'impact, la larme se divise en deux en éventail serré (`Split(count: 2, spread: "0.35")`). | item_bifurque, tears_bifurque |
| `grappe` | R | diable | à l'impact, la larme éclate en cinq sur un large éventail (`Split(count: 5, spread: "1.4")`) au prix de 25 % de dégâts. | item_grappe, tears_grappe |
| `halo` | R | boutique, tresor | les larmes tournent autour du porteur (rayon 72, un tour en 90 frames) pendant 360 frames (`Orbit`). | item_halo, tears_halo |
| `acide` | R | diable | chaque larme pose deux piles de `poison` sur sa cible (`ApplyStatus`, `stacks: 2`). | item_acide, tears_acide |
| `cendre_froide` | R | boss | chaque larme enflamme (`brulure`) et ralentit (`lenteur`) sa cible, deux `ApplyStatus` dans un même `on_hit`. | item_cendre_froide, tears_cendre_froide |
| `massue` | U | boutique, tresor | une larme sur cinq étourdit sa cible (20 % de chance, +5 % par point de `Luck`, statut `etourdi` : 60 frames sans bouger ni tirer). | item_massue, tears_massue |
| `philtre` | R | diable | une larme sur sept charme sa cible (15 % de chance, +5 % par point de `Luck`, statut `charme` : elle attaque l'ennemi le plus proche pendant 200 frames). | item_philtre, tears_philtre |
| `saignee` | C | boutique, tresor | chaque larme fait saigner sa cible (statut `saignement` : 1 point toutes les 15 frames pendant 180 frames). | item_saignee, tears_saignee |
| `pois_chiche` | C | boutique, tresor | larmes minuscules mais rapides à tirer (`ShotSize` Mul 0,6 : collision et sprite réduits, `FireRate` Pct 0,4 : 3 -> 4,2 larmes par seconde). | item_pois_chiche, tears_pois_chiche |
| `plume` | C | boutique, tresor | larmes plus rapides (`ShotSpeed` Pct 0,4 : 260 -> 364) mais de portée réduite (`Range` Pct -0,3 : 300 -> 210). | item_plume |
| `lest` | U | boutique, tresor | larmes lourdes : dégâts +60 % (`Damage` Pct 0,6) mais vitesse des larmes -30 % (`ShotSpeed` Pct -0,3 : 260 -> 182). | item_lest |
| `fer_a_cheval` | R | secrete, tresor | (revue : plus de tag `larmes`, il ne touche pas aux larmes) chance +3 (`Luck` Add 3 : +15 % de chance d'effet pour `massue`, `philtre`, `venin`…). | item_fer_a_cheval |
| `loupe` | U | boutique, tresor | larmes deux fois plus grosses (`ShotSize` Mul 2 : collision et sprite), +10 % de dégâts, cadence -20 % (`FireRate` Pct -0,2). | item_loupe, tears_loupe |
| `ardoise` | U | boutique, tresor | larmes à courte portée et gros dégâts : elles disparaissent après 45 frames de vol (`Lifetime(45)`), dégâts +80 % (`Damage` Pct 0,8). | item_ardoise, tears_ardoise |

### Familiers (6)

| Objet | Rareté | Pools | Effet | Scénarios |
|---|---|---|---|---|
| `cousin` | C | boutique, tresor | un cousin (`cousin`) qui suit le porteur à 44 unités et tire une larme avec lui, au plus tous les 20 frames (`Follow` + `MirrorShot`). | familiar_cousin, item_cousin |
| `garde` | U | boutique, tresor | un garde (`garde`) collé au porteur (`Follow` à 20 unités) qui arrête les tirs ennemis (`Follow` + `BlockBullets`). | familiar_garde, item_garde, isaac_synergy_garde |
| `sentinelle` | R | tresor | une sentinelle (`sentinelle`) qui tourne autour du porteur (rayon 52, un tour en 90 frames) et tire une larme avec lui (`Orbit` + `MirrorShot`). | familiar_sentinelle, item_sentinelle |
| `anneau` | U | boutique, tresor | deux anneaux (`anneau`, `per_copy: 2`) qui tournent à 180° l'un de l'autre autour du porteur (rayon 60, un tour en 150 frames) et arrêtent les tirs ennemis (`Orbit` + `Bl | familiar_anneau, item_anneau |
| `escorte` | R | boss | deux escortes (`escorte`, `per_copy: 2`) en file derrière le porteur (`Follow` à 30 unités, la seconde suit la première) qui tirent une larme avec lui, au plus tous les 1 | familiar_escorte, item_escorte |
| `ombre` | C | boutique, tresor | une ombre (`ombre`) qui tourne serré autour du porteur (rayon 24, un tour en 40 frames) et arrête les tirs ennemis (`Orbit` + `BlockBullets`). | familiar_ombre, item_ombre, isaac_ombre_bloque, isaac_ombre_bloque_sans_objet |

### Actifs (9)

| Objet | Rareté | Pools | Effet | Scénarios |
|---|---|---|---|---|
| `fiole_vive` Active(Rooms(3)) | C | boutique, tresor | actif chargé par trois salles nettoyées ; à l'usage, +2 PV (un cœur, `Heal`, borné aux PV maximaux). Le gabarit `ItemHeld` part à pleine santé : la preuve du soin est le | item_fiole_vive, isaac_fiole_vive |
| `egide` Active(Rooms(2)) | U | boss | actif chargé par deux salles nettoyées ; à l'usage, un point de bouclier (`GainShield(1)`, `combat::actors::Shield`) : le prochain coup reçu est annulé en entier. Le bouc | item_egide, isaac_egide, isaac_egide_temoin |
| `bombe_fiole` Active(Frames(900)) | U | boss | actif rechargé par le temps (900 frames, `Frames`) ; à l'usage, une explosion centrée sur le porteur (`Explode`, rayon 60, 8 dégâts, `destroys: false` : ne casse ni roche | item_bombe_fiole, isaac_bombe_fiole |
| `nova` Active(Rooms(2)) | R | tresor | actif chargé par deux salles nettoyées (revue : la charge `Damage` de 60 se rechargeait par les dégâts de sa propre onde) ; à l'usage, `GainShield(1)` puis une onde (`Explode`, rayon 100, 10 dégâts, `destroys: false`) ; **coop : un coéquipier à portée prend 10 (D113)**. | item_nova, isaac_nova |
| `elixir_vitesse` Active(Rooms(2)) | U | boutique, tresor | actif chargé par deux salles nettoyées ; à l'usage (`UseActive`), +60 % de vitesse de déplacement pendant 300 frames (`MoveSpeed` en unités/s : 120 au repos, 192,00073 à | item_elixir_vitesse |
| `frenesie` Active(Rooms(3)) | U | tresor | actif chargé par trois salles nettoyées ; à l'usage, +100 % de cadence de tir pendant 240 frames (`FireRate` Pct 1,0 : multiplicateur 1 -> 2). | item_frenesie |
| `grimoire` Active(Rooms(3)) | R | diable | actif chargé par trois salles nettoyées (revue : doublon de `calice_ame`, qui se charge aux dégâts) ; à l'usage, 1 point de vie d'âme (`GainSoul`) et +2 de chance pendant 600 frames (`TimedModifier` `Luck`). | item_grimoire |
| `lanterne_vue` Active(Rooms(1)) | C | boutique, tresor | actif chargé par une salle nettoyée ; à l'usage, +80 % de portée et +50 % de vitesse des larmes pendant 600 frames (`Range` 1 -> 1,8, `ShotSpeed` 1 -> 1,5). | item_lanterne_vue |
| `hydre` Active(Damage(35)) | R | tresor | actif chargé par 35 points de dégâts infligés ; à l'usage, un point de bouclier (`GainShield`) et +50 % de dégâts pendant 240 frames. Substitution : la forme `Emit("hydre | item_hydre |

### Effets (12)

| Objet | Rareté | Pools | Effet | Scénarios |
|---|---|---|---|---|
| `sablier` | C | boutique, tresor | une salle nettoyée (`OnRoomClear`) soigne 1 PV (`Heal`, un demi-cœur). | item_sablier, isaac_sablier, isaac_sablier_temoin |
| `cuirasse` | U | tresor | une salle nettoyée (`OnRoomClear`) donne un point de bouclier (`GainShield`) : il absorbe un coup entier (§18). | item_cuirasse, isaac_cuirasse |
| `rage` | C | boutique, tresor | un coup reçu (`OnDamageTaken`) donne +50 % de dégâts pendant 180 frames (`TimedModifier`). Seuils mesurés (`cible_mordante`, premier coup à f~8, un coup toutes les 120 fr | item_rage, tears_rage |
| `avidite` | C | tresor | un ennemi tué (`OnKill`) donne +40 % de vitesse de déplacement pendant 120 frames (`TimedModifier`, `MoveSpeed` en unités/s : 120 au repos). Seuils mesurés : 167,99927 à | item_avidite, tears_avidite |
| `contagion` | U | boutique, tresor | un coup porté (`OnHit`) enflamme la cible (`ApplyStatus` `brulure`), au plus une fois toutes les 240 frames (`cooldown`). Seuils mesurés : `brulure` présente à f140 avec | item_contagion, tears_contagion |
| `metronome` | C | boutique, tresor | toutes les 120 frames (`Tick`), +60 % de cadence de tir pendant 40 frames (`TimedModifier`, `FireRate` : multiplicateur de base 1). Seuils mesurés : 1,0 à f60 (hors fenêt | item_metronome |
| `second_souffle` | U | tresor | le premier coup reçu de la salle (`OnDamageTaken`, une charge rendue à chaque salle : `recharge: PerRoom`) donne un point de bouclier (`GainShield`), qui absorbera le cou | item_second_souffle, tears_second_souffle |
| `bol_de_soupe` | C | tresor | au début de chaque étage (`OnFloorStart`), soigne 2 PV (`Heal`, un cœur). | item_bol_de_soupe, isaac_bol_de_soupe, isaac_bol_de_soupe_temoin |
| `rancune_froide` | R | tresor | un coup reçu (`OnDamageTaken`) gèle son auteur (`ApplyStatus` pose `gel` sur la source du coup). Seuils mesurés : `gel` présent sur la cible à f190 avec l'objet, absent s | item_rancune_froide, tears_rancune_froide |
| `epines_de_chair` | R | diable | un coup reçu (`OnDamageTaken`) fait exploser le porteur (`Explode`, rayon 40, 3 dégâts, sans détruire le décor). Tir ami (§69) : l'explosion touche aussi le porteur, mais | item_epines_de_chair, tears_epines_de_chair |
| `bouclier_de_fortune` | C | tresor | au ramassage **de cet objet seul** (`OnPickup` filtré par `item`, §67), deux points de bouclier (`GainShield`, §18). | item_bouclier_de_fortune, isaac_bouclier_de_fortune |
| `vengeance` | U | tresor | un ennemi tué (`OnKill`) soigne 1 PV par exemplaire (`Scaled(PerCopy)`), au plus une fois toutes les 600 frames (`cooldown`). | item_vengeance, isaac_vengeance |

### Stats (4)

| Objet | Rareté | Pools | Effet | Scénarios |
|---|---|---|---|---|
| `coeur_de_pierre` | R | boss | passif de stats, un cœur de plus (`MaxHealth` +2 PV) payé par un pas plus lent (`MoveSpeed` −10 %). Le joueur | item_coeur_de_pierre |
| `pot_de_miel` | C | boutique, secrete, tresor | passif de stats, un demi-cœur de plus (`MaxHealth` +1 PV), sans soin. Preuve : `item_pot_de_miel` (`Stat` résolue à f1 par `ItemHeld`). | item_pot_de_miel |
| `gants_de_lin` | C | boutique, tresor | passif de stats, +10 % de vitesse de déplacement (`MoveSpeed`) et +10 % de cadence (`FireRate`). | item_gants_de_lin |
| `talisman` | U | boutique, tresor | passif de stats, +1 de chance (`Luck`) et +10 % de dégâts (`Damage`). Preuve : `item_talisman` (`Stat` résolue à f1 par `ItemHeld`). | item_talisman |

### Consommables (3)

| Objet | Rareté | Pools | Effet | Scénarios |
|---|---|---|---|---|
| `pain` | C | boutique, salle, secrete | consommable de soin, 1 PV (un demi-cœur) au ramassage, ramassé même à pleine vie (pas de `refuse_when_full`). | isaac_consommables_lot |
| `eclat_ame` | C | boutique, secrete | consommable, 1 point de vie d'âme (`ExtraHealth`, un demi-cœur d'âme) au ramassage, jamais refusé. | isaac_consommables_lot |
| `festin` | U | boutique, secrete | consommable, 2 PV et 1 point de vie d'âme au ramassage ; reste au sol à pleine vie (`refuse_when_full`). | isaac_consommables_lot |

Pools après T11d2 (entrées avant → après) : `tresor` 33 → 71, `boutique` 26 → 55, `boss` 6 → 11, `diable` 4 → 9, `secrete` 5 → 10, `salle` 7 → 8. Statuts : `etourdi`, `charme`, `saignement`.

## 3. Ennemis, boss, gabarits

- `araignee` (sol, 4 PV, 66 unités/s, contact), `follet` (`Flying`, 5 PV, `Charge` 25 + contact), `arroseur` (tireur au sol, 8 PV, `KeepDistance`, pattern `gerbe`, `sight_range` 300) : un `test:` chacun (`enemy_<id>_still` / `_moving`, verts). Placés sans changer le nombre d'ennemis par salle, en remplaçant une mouche ou un suiveur dans huit gabarits de combat (`CombatRochers`, `Butin`, `Piques`, `Mixte`, `Croix`, `Long`, `Haut`, `Grand`) : script `m3-t11d2-volume.make_salles.py`, mêmes vérifications que T11d1, `salles.ldtk` régénéré.
- **`colosse`** (44 PV, immobile, trois phases ; renforts `araignee`, `follet`, `arroseur` ; patterns `gerbe`, `croix`, `couronne`, `tourbillon`) : scénario généré `boss_colosse` vert (un bot `fonceur` le bat, victoire f449 sur l'étage court `sous_sol_profond_court`). Premier jet (50 PV, plus de renforts, spirale à trois bras) : le bot mourait en phase 2 ; allégé jusqu'à ce que le scénario passe (aucune fourchette visée).
- **Rotation, justification** : second étage de la séquence par défaut (`floors/sous_sol.ron` : `sous_sol` puis `sous_sol_profond`), le `gardien` reste au premier. Choisi plutôt que le hasard : un boss par étage (difficulté croissante), toutes les séquences de test et leurs traces gardent le `gardien`, pas de loterie de graine dans les scénarios de grille. **Mécanisme (revue)** : `FloorGrammar::boss` désigne le type de boss de la grammaire et le planificateur choisit les gabarits par ce type : trois gabarits `BossProfond*` de type `boss_profond` dans `salles.ldtk` (23 gabarits, les vingt premiers inchangés, vérifié par comparaison JSON), `rooms/boss_profond.ron`, grammaires `sous_sol_profond*.ron` à `boss: "boss_profond"`. `salles_profond.ldtk` est supprimé. Deux petits changements de code hors simulation : `RoomKindDef::boss` (RON `boss: true`) lu par la minicarte à la place de `room.kind == "boss"` (`HudRoomBounds::boss`, `RoomKindTable::is_boss`, test unitaire), `BossFightTest::room` (type de la salle du boss dans l'attente générée).

## 4. Scénarios recalés et écrits

Recalés (graines, chronologies, objets de butin et de présentoir, le but de chacun conservé, aucune attente retirée) : `grid_floor_seed_123`, `grid_floor_seed_23`, `grid_post_boss` (butin `bombe_fiole`), `isaac_floor_seed_1`, `bot_isaac_floor`, `bot_isaac_two`, `isaac_shop_buy`, `devil_room_chance`, `isaac_spikes_temoin` (marche raccourcie), commentaire de `isaac_spikes`.
`bot_isaac_floor` : sans armure le bot mourait au premier combat du second étage (f3671, les trois ennemis neufs dans les gabarits) ; **`Armor +50` (D93, aucun équilibrage)** ; il bat le `colosse` (f5442) et le scénario prouve maintenant ses deux boss. D93 compte donc un scénario d'armure de plus.
Écrits : `isaac_fiole_vive`, `isaac_egide` (+ témoin), `isaac_bombe_fiole`, `isaac_nova`, `isaac_sablier` (+ `isaac_sablier_temoin`, partagé avec `isaac_cuirasse` et `isaac_vengeance`), `isaac_bol_de_soupe` (+ témoin), `isaac_bouclier_de_fortune`, `isaac_familiers_file`, `isaac_consommables_lot`, `isaac_ombre_bloque` (+ témoin).

## 5. Traces rebénies, preuve §5

Contre le point de départ : 40 `.trace` modifiées (toutes `isaac`), 134 neuves (118 générées, 16 écrites à la main) ; **zombies, throne, gungeon, testbed : aucune trace ne bouge** (suite complète, `make gen` ×5 sans diff).
Preuve §5 : dumps `ALACOD_DUMP_TRACE` du même binaire, `ALACOD_GAME_NAME=isaac ALACOD_GAME_DIR=<état>`, états de `games/isaac` (S0 `main` `8e25731d` ; S1 = S0 + objets, pools, statuts, familiers ; S2 = S1 + ennemis, patterns, gabarits ; S3 = la branche : `colosse`, type `boss_profond`, séquence), `scripts/trace-diff.py … --ignore Inventory,BombCarrier`, six scénarios (`hit_iframes_isaac`, `isaac_boots`, `grid_secret_no_bomb`, `devil_room_absent`, `generated/isaac/boss_gardien`, `generated/isaac/weapon_larmes`) puis `isaac_floor_seed_1` (revue) :

| Étape | Résultat |
|---|---|
| S0 → S1 (**pools enrichis**) | première frame qui diffère = 0 sur les cinq scénarios d'étage : `PoolState` (objet tiré par pool) et le contenu des présentoirs seulement ; `weapon_larmes` (carte de test, aucun pool) **identique** |
| S1 → S2 (**nouveaux ennemis dans les gabarits**) | première frame = 0 : les ennemis de salle (`suiveur`, `mouche` → `araignee`, `follet`, `arroseur`), rien d'autre qu'un plan d'étage différent (les ennemis posés changent) ; `weapon_larmes` identique |
| S2 → S3 (**boss, séquence**) | six scénarios : **identiques** (199 à 1 499 frames, aucun n'atteint le second étage). **`isaac_floor_seed_1`** (4 000 frames, deux étages, revue) : **première frame qui diffère = 3752**, la génération du second étage ; **une seule entité change, le boss net id 584 : `gardien` → `colosse`** (le plan, les ids, les salles, les ennemis et le butin sont identiques) |

Les scénarios de grille `grid_floor_seed_*` jouent `sous_sol_un` (un étage, `gardien`) : ils ne bougent que par les pools et les ennemis (S1, S2).

## 6. Campagnes de contrôle T14

`docs/taches/rapports/m3-t11d2-volume.campagne-t14.sh` (copie de `m3-t11d1-gabarits.campagne-t14.sh` : `alacod-sim --game isaac --bots N --profiles explorateur… --floors sous_sol --until-floor 5 --max-frames 20000 --pickup-progress`, séquence par défaut, sans `--bombs`), lots de 20 graines, binaire d'avant la fusion avec `main` (D69 refactore sans bouger les traces, voir § 7), JSON et journaux par lot dans `/tmp/t11d2/camp` (non versionnés). **Les victoires sont des informations.**

| Campagne | Graines | Victoires | Morts | Soft-locks | Desyncs | Premier étage franchi | Graines avec champion | Frames moyennes |
|---|---|---|---|---|---|---|---|---|
| **s1** : 1 bot | 1 à 200 | 24 (12 %) | 175 | **1** (graine 63, faux : voir ci-dessous) | **0** | 79 | 150 | 3 455 |
| **i2** : 2 bots | 1 à 200 | 95 (47,5 %) | 105 | **0** | **0** | 160 | 179 | 5 467 |

Pour mémoire, T11d1 (même script, contenu à 49 objets, un seul boss) : s1 29 victoires, i2 115. Les chiffres ne sont pas comparés ni réglés (décision de William). Le détail graine par graine est dans `m3-t11d2-volume.campagne-t14.tsv` (issue, frames, étage, champions, bombes, murs secrets, portes à clé, achats, coffres, phases de boss).
Atteignabilité observée par les bots : mur secret ouvert à la bombe 202 fois dans 182 graines (s1), 334 dans 195 (i2) ; porte à clé du trésor 43 graines (s1), 128 (i2) ; achats en boutique 492 dans 130 graines (s1), 860 dans 180 (i2) ; coffres 353 dans 169 graines, 596 dans 184 ; **les victoires sont des parties où le `colosse` meurt** (24 et 95).

**Graine 63, 1 bot : le « soft-lock » n'en est pas un.** Le détecteur d'`alacod-sim` s'arrête à f2118 (plafond « phase NotStarted », 0 kill, joueur à 1 288 sous, ennemis de la salle hors de vue à 408 à 632 unités : un long trajet, famille D100). Protocole de T11d1 (scénario `shopping: true`, graine 63, séquence par défaut, bot `explorateur`, 4 800 frames, **sans détecteur**, `ALACOD_EVENTS=1`) : salle de combat verrouillée f2034, nettoyée f2421, boss `gardien` verrouillé f2451, phases f2553 et f2694, boss mort f2738, portail f2739, étage suivant f3294 (champions `explosif` et `essaim`), mort f3609. La partie reprend et se résout : aucun gel.

## 7. Vérifié (résultats réels, sur la fusion avec `origin/main` `5fa55fcb`)

- `make test_scenarios_par SHARDS=3` **sans BLESS** : 3 tranches ok, **802 scénarios**, 0 « trace différente » (306 s) ; la bénédiction des 174 traces avait été faite avant la fusion (`BLESS=1`, 621 s).
- `make test_crates` : **1 271 passés, 0 échec**. `cargo fmt --all -- --check` propre. `make lint` ×5 : aucune erreur (isaac **0 avertissement**, gungeon 1 comme avant). `make check_sim_boundary` OK (13 crates, 2 en dette D69), `check-forbidden.sh` OK (40 occurrences), `check-rollback-registration.sh` OK.
- `make gen GAME=…` ×5 : rc 0 chacun, `git status` propre ensuite (isaac : 91 objets avec `test:`, 20 gabarits de personnage, 2 boss, 3 champions, 43 témoins, tous verts).
- Fuzz d'inputs (avant et après la fusion, mêmes résultats) : `ALACOD_FUZZ=0:8` (9 graines), `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` (9 graines, 8 explosions chacune), `ALACOD_FUZZ_SCENARIO=isaac_floor_seed_1 ALACOD_FUZZ=0:4` : aucune divergence.
- p2p `scripts/p2p-stress.sh isaac tests/scenarios/p2p_smoke.ron 3000` (Docker OrbStack, `MATCHBOX_PORT=3571`, projet `b1`) : avant la fusion **1 707 frames resimulées** (client 0 : 16 rollbacks, profondeur maximale 11), après **1 699**, 0 `DesyncDetected`, 2 999 lignes identiques. Le script ne franchit pas d'étage : le `colosse` n'est pas rejoué en p2p (comme la transition d'étage, T11d1).
- Disque : la machine est passée à 100 % pendant la tâche (b0 : 136 Go de `target`) ; un lot de campagne a été rejoué.

## 8. Dettes (D112 à D115, `docs/taches/dettes.md`)

- **D112** `SpawnPattern` et `Emit`/`OnEvent` refusés à `OnUse` d'un objet : `nova` = `Explode`, `hydre` = actions directes.
- **D113** `Explode` d'objet : tir ami `Always`, blesse le porteur **et ses coéquipiers** : `GainShield(1)` devant `bombe_fiole` et `nova` (il ne protège que le porteur : un coéquipier à portée meurt à 6 PV), `epines_de_chair` protégé par les `hit_iframes` ; la charge `Damage` compte tous les dégâts du porteur, ses explosions comprises (`nova` passé en `Rooms(2)`).
- **D114** lacunes de preuve : pas d'attente `Shield`, pas de départ blessé (le `cooldown` de `vengeance` n'est pas prouvé), `familiar:` sans input ni balle hostile, pas de `test:` pour un consommable, `hits` ne prouve pas une absence.
- **D115** (réduite) le lint n'exige pas `boss: true` sur le type nommé par `FloorGrammar::boss` ; le `colosse` passe par un type de salle propre `boss_profond` (plus de second fichier de gabarits, voir § 3).
- Rappel : D93 (un scénario d'armure de plus : `bot_isaac_floor`), pas de sprite d'objet (T17), `spectre` toujours sans gabarit propre.

## 9. Relecture indépendante (agent `reviewer`, lecture seule)

Aucun P0. P1 : (1) `boss_colosse` généré désynchronisé de son `test:` (3 000 frames contre 1 200) : régénéré et trace bénie ; (2) §76.5 inachevée et rapport absent : écrits ; (3) comptes de traces faux (40 / 134) : corrigés ; (4) `grid_floor_seed_*` cités à tort comme voyant le second boss : corrigé.
P2 corrigés : `TODO` laissés dans trois objets (`acide`, `cendre_froide`, `aiguille`), `rage` (empilement 2,0 et non 2,5), `fer_a_cheval` (+37,5 % pour `venin`), « 50 PV » du `colosse`, recharge de `bombe_fiole`, script de `sous_sol.ron`, « sept gabarits » (huit), script de campagne copié sans changement, poids des consommables, justification (1) de la rotation (même pool de butin). Reconnu et laissé : le `cooldown` de `vengeance` n'est pas prouvé (D114) ; `fiole_vive` et `epines_de_chair` ont une attente de `test:` qui passe aussi sans l'objet (la preuve est ailleurs, dit en commentaire).

## 10. Revue de livraison (indépendante) : points traités

1. **`nova` et `bombe_fiole` en coop.** Écrit dans D113 : l'`Explode` d'un objet est une bombe à tir ami `Always` de l'équipe du porteur, un coéquipier à moins de 100 unités (`nova`, 10 dégâts) ou 60 (`bombe_fiole`, 8) meurt à 6 PV, le `GainShield(1)` ne protège que le porteur ; la charge `Damage` additionne tous les `DamageEvent` du porteur, ses propres explosions comprises (avec plusieurs cibles l'actif se rechargeait entièrement : passer de 25 à 60 le masquait). **`nova` passe en `Active(Rooms(2))`** (`isaac_nova` : charge 2 à f20, 0 à f36 et jusqu'à f119) ; `bombe_fiole` (charge `Frames(900)`) n'est pas concernée par la recharge.
2. **Noms (Q9).** Comparés aux 1 018 noms de `tboi.com/all-items` : `metronome` (Metronome), `contagion` (Contagion), `halo` (The Halo), `sablier` (The Hourglass), `ombre` (Shade) existent dans Isaac. Champ `name:` seulement : Pendule, Miasme, Cerceau, Clepsydre, Silhouette. Deux traductions de T11b renommées aussi : `nuee` (The Swarm) → Volée d'orbes, `reliquaire` (The Relic) → Châsse d'âme (attente du bandeau de `isaac_reliquaire` et test `hud_model` mis à jour). Autres noms : aucun équivalent dans la liste (recherche par mot-clé sur les cent noms). Ids inchangés, **aucune trace ne bouge** pour les noms.
3. **D115 corrigée (constat inexact).** `FloorGrammar::boss` existait. `salles_profond.ldtk` supprimé ; trois gabarits `BossProfond`, `BossProfondGrand`, `BossProfondAbris` (type `boss_profond`) à la fin de `salles.ldtk` (23 gabarits ; les vingt premiers identiques octet pour octet au JSON précédent) ; `rooms/boss_profond.ron` ; `sous_sol_profond.ron` et `_court.ron` : `boss: "boss_profond"`, `required: {"boss_profond": 1, …}`. **Minicarte** : propriété du type, pas un nom : `RoomKindDef::boss` (RON `boss: true`, faux par défaut), `RoomKindTable::is_boss`, `HudRoomBounds::boss` rempli au chargement par `map_ldtk` ; `boss.ron` de gungeon, testbed, `boss_butin` et isaac portent `boss: true` ; test unitaire. Autre petit code : l'attente générée du gabarit `BossFight` disait « salle boss nettoyée » (`BossFightTest::room`, défaut `boss`, `colosse` : `boss_profond`). D115 réécrite (reste : un lint du type de boss).
4. **D93** : `bot_isaac_floor` (`Armor +50`) ajouté, avec la tendance vers le critère n°3 (`isaac_full_run`).
5. **Preuve §5 S2 → S3 sur `isaac_floor_seed_1`** : première frame qui diffère **3752** (génération du second étage), **une seule entité change** (boss net id 584 : `gardien` → `colosse`). Dumps, en plus : `isaac_floor_seed_1` et `bot_isaac_floor` contre le commit de la première livraison : seule la chaîne `kind` de `RoomChanged` (`boss` → `boss_profond`) bouge, f5273 pour le bot, f245 pour `boss_colosse` ; `isaac_floor_seed_1` identique (3 999 frames).
6. **Facultatifs faits** : `grimoire` différencié (`Active(Rooms(3))`, `GainSoul(1)` + `Luck` +2 pendant 600 frames ; `calice_ame` reste l'actif à dégâts) ; `fer_a_cheval` perd le tag `larmes` : **37 objets de larmes** (cible 30, 23 à `projectile`).

**Traces rebénies (5, cause)** : `item_nova`, `isaac_nova` (charge par salles), `item_grimoire` (nouvel effet), `generated/isaac/boss_colosse` et `bot_isaac_floor` (chaîne `kind` de `RoomChanged`). Aucune autre trace ne bouge, autres jeux identiques.

**Résultats réels (fusion avec `origin/main` `a3aa8456`, D69 E3a)** : `make test_scenarios_par SHARDS=3` sans BLESS : 802 scénarios, 3 tranches ok, 0 trace différente ; `make test_crates` : 1 276 passés et 1 échec (le test `hud_model` qui lit le vrai nom « Reliquaire », renommé) ; corrigé, **`make test_crates` non rejoué en entier** : `cargo test -p game --lib hud_model` 13 verts, le reste inchangé ; `cargo fmt --all -- --check` propre ; `make lint` ×5 : aucune erreur (isaac **0 avertissement**, gungeon 1 et testbed 115 comme avant) ; `make gen` ×5 : rc 0, arbre propre ; `check_sim_boundary`, `check-forbidden`, `check-rollback-registration` OK ; fuzz `0:8` et `ALACOD_FUZZ_BOMBS=1` verts ; p2p isaac 3 000 frames : **1 859 frames resimulées**, 0 desync, 2 999 lignes identiques ; **campagne s1, graines 1 à 20 : 3 victoires (1, 15, 20), 17 morts, 0 desync, 0 soft-lock** (mêmes issues que les 20 premières graines de la campagne complète).
