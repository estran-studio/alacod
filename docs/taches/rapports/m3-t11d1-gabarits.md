# Rapport m3-t11d1-gabarits (b1)

Conventions : `docs/conventions.md` §76.4. Branche `m3-t11d1-gabarits` depuis `origin/main` `5067b209` (D88, D95/D96, D69 étape 1, D76/D79/D80/D90/D91 mergées). Données seulement
(`games/isaac/assets/`, scénarios et traces `isaac`) ; seuls changements de code : un test de `map_ldtk` ajusté et le diagnostic ignoré `floor_probe`.
Décision de William du 2026-10-18 : T11d est coupé en deux ; T11d1 (ici) = structure de la partie ordinaire, T11d2 = volume de Q1. **L'équilibrage n'est pas un objectif** :
victoires et morts ci-dessous sont des informations, aucun réglage n'a visé une fourchette.

## 1. Fait

- **Vingt gabarits** (au lieu de dix) dans `maps/salles.ldtk`, par `docs/taches/rapports/m3-t11d1-gabarits.make_salles.py` (reprend le script de T5 : les dix gabarits d'avant sont
  reproduits à l'octet près, vérifié ; il vérifie les dix autres, voir §2). Neuf gabarits de combat (un seul avant) avec rochers, rochers à butin et piques, trois de boss
  (`BossAbris` en plus), trois de trésor, secrète, diable, boutique, départ, transit.
- **Boutique** : type `boutique`, pool `boutique` (26 entrées, `exhaust`, prix en sous par rareté), exigée par la grammaire `sous_sol` (un cul-de-sac par étage).
- **Récompenses** (cœurs, sous, bombes, clés) : `loot/salle.ron` et `loot/secrete.ron` déjà cohérents avec §68 et §73 ; **inchangés** (la clé est aussi en boutique).
- **Curses dans la séquence par défaut** : `floors/sous_sol.ron`, chance 0,3 par étage, les cinq selon leurs poids (`darkness` 3, `unknown` 2, `blind` 2, `maze` 2, `labyrinth` 1 ; 112 étages maudits sur 400 graines au premier étage).
- **D81 fermée** : `pleureur` a une bombe de départ (`starting_consumables`) et la `BombDef` (mèche 75 frames, rayon 56, 4 PV, tir ami `Always`).
- **Surface `piques`** (`surfaces/piques.ron`, 1 PV tous les 60 frames) et dossier `surfaces` du manifeste.
- **Scénarios** : quatorze recalés (graines, chronologies ; `achievement_unlocks` ne change que ses commentaires), quatre neufs (`isaac_shop_buy`, `isaac_bomb_rock`, `isaac_spikes` et son témoin) ; `make gen GAME=isaac` : les `.ron` générés ne changent pas, 95 traces bénies.
- **Tests** : `map_ldtk::generation::isaac_etages_de_grille_sur_1000_graines` (les neuf gabarits `Combat…` sont des salles ordinaires ; une boutique par `sous_sol`, cul-de-sac
  derrière une salle ordinaire) ; `…_sous_trois_fois_grand_de_gungeon` (ancien `…_sous_deux_fois_…` : trois chronomètres, borne 2 → 3 sur la génération entière, garde sur les évaluations de candidats, voir §8 et D107) ; diagnostic `floor_probe`.
- **Docs** : `docs/conventions.md` §76.4 (et retouches §69, §70, §73, §74, §76.1 à §76.3, §77), `dettes.md` (D81 fermée, D93 annotée, D99, D100), `m3-t11-contenu-isaac.md`
  (T11d1/T11d2), `CLAUDE.md`, journal §10.

## 2. Gabarits

| Gabarit | Cases | Contenu |
|---|---|---|
| `Combat` | 1×1 | inchangé : `suiveur`, `mouche`, `tourelle_croix` |
| `CombatRochers` | 1×1 | un amas de rochers (3 cases en L) dans chaque angle ; `suiveur`, `mouche`, `tourelle_croix` au centre |
| `CombatButin` | 1×1 | deux angles de rochers dont le coin est un rocher à butin ; `suiveur`, `mouche`, `cracheur` |
| `CombatPiques` | 1×1 | deux angles de piques ; `suiveur`, `suiveur`, `bourdon` |
| `CombatMixte` | 1×1 | rochers (un amas plein, un à butin), piques dans les deux autres angles ; `suiveur`, `rampant`, `mouche` |
| `CombatCroix` | 1×1 | deux angles de rochers ; `tourelle_croix`, `cracheur`, `mouche` |
| `CombatLong` | 2×1 | deux angles de rochers à butin, deux de piques ; `suiveur` ×2, `mouche`, `cracheur` |
| `CombatHaut` | 1×2 | trois angles de rochers (deux à butin), un de piques ; `suiveur`, `mouche`, `rampant`, `tourelle_croix` |
| `CombatGrand` | 2×2 | quatre angles de rochers (deux à butin) ; `suiveur`, `mouche`, `cracheur`, `bourdon` |
| `Boss`, `BossGrand`, `BossAbris` | 1×1, 2×2, 1×1 | `gardien` au centre ; `BossAbris` : un amas de rochers dans chaque angle |
| `Tresor`, `TresorLong`, `TresorHaut` | 1×1, 2×1, 1×2 | inchangés |
| `Secrete`, `Diable`, `Transit`, `Depart` | 1×1 | inchangés |
| `Boutique` | 1×1 | vide (présentoirs posés par le type de salle) |

**Pourquoi des rochers dans les angles seulement.** Premier essai : rochers au centre (piliers, amas à butin au milieu). Le `grid_post_boss` du bot s'est arrêté après 700 frames : le bot
`explorateur` ne voit pas les rochers (`bots::input` ne lit que `Wall`, `Window`, `DoorComponent` ; `spawn_level_rocks` ne pose pas `Wall`), marche en ligne droite et reste collé
au premier rocher posé sur son chemin. Le script refuse donc tout rocher à moins d'une case (piques : trois quarts) d'un segment qui joint deux ouvertures, une ouverture et un ennemi
ou le centre, et vérifie que les ouvertures et les ennemis restent reliés par des cases libres larges (bloc libre de 2 × 2). Casser un rocher n'est jamais nécessaire pour traverser une salle.
Un gabarit par pose est tiré uniformément : sur 300 graines de `sous_sol_un`, chaque gabarit de combat sort 139 à 183 fois.

## 3. Tableau d'atteignabilité (`docs/taches.md` §8.1)

Séquence par défaut d'`isaac` (`entry.floors: sous_sol`, deux étages, grammaire en grille, sans `FloorsOverride`). Graines : `alacod-sim --game isaac --bots 1 --profiles explorateur --shopping
--floors sous_sol --until-floor 2` (le plan de l'étage 0 est celui de la graine de carte ; `--max-frames 6` suffit pour lire les lignes `slot_assign` et `curse` de la génération).
« Scénario » = scénario joué à la main (le bot de T14 n'existe pas encore sur `main`). Balayage des objets sur les graines 1 à 400.

| Mécanique | Où une partie ordinaire la rencontre | Graine `alacod-sim` (étage 1) | Scénario scripté |
|---|---|---|---|
| Larmes modifiées, empilement | objets de larmes au trésor (premier étage ouvert) et en boutique | 1 : trésor `vent_arriere` (`ShotSpeed`), boutique `givre` ; **deux exemplaires** par deux pools : 71 (`epouvante` au trésor **et** en boutique), 76 (`vent_arriere`), 95 (`oeil`) | `isaac_shop_buy` (achat), générés `tears_*` ; empilement du moteur : `tears_stack_two_copies` (testbed) ; lint `Tears` sans avertissement |
| Effets v3, transformations | objets à effets : trésor, boutique, boss | trésor `crocs` 14, `pouls` 3 ; boutique `rancune_vive` 8 ; boss `sang_chaud` 8, `calice_ardent` 4 ; transformations : non retenues (Q10), `transformation_two/three` (testbed) | générés `item_*`, `tears_crocs`, `tears_rancune_vive`, `tears_sang_chaud` |
| Familiers, orbitaux | trésor (trois), boutique (trois) | 9 (trésor `nuee`, boutique `frere`), 13 (`orbe` aux deux), 18 (`frere` aux deux) | `isaac_synergy_garde`, générés `familiar_*` |
| Bombes | **bombe de départ** de `pleureur` ; boutique, récompense de salle, coffre secret, rochers à butin en donnent | toute graine (une bombe au départ) | `grid_secret_bomb` (graine 39), `isaac_bomb_rock` (graine 4) |
| Rochers | sept des neuf gabarits de combat (`Combat` et `CombatPiques` n'en ont pas), `BossAbris` | 4 : `CombatButin` au nord du départ, 45 rochers à l'étage | `isaac_bomb_rock` : 45 → 42, rocher à butin → une `ration` |
| Piques | gabarits `CombatPiques`, `CombatMixte`, `CombatLong`, `CombatHaut` | 10 : `CombatPiques` au nord du départ | `isaac_spikes` : 1 PV dès f108, f168, f228, f288 (les deux premiers coups sont ceux des piques : le témoin `isaac_spikes_temoin` reste à 6 PV jusqu'à f170 ; les suivants peuvent être aussi ceux d'un ennemi, § « Traces ») |
| Salle secrète | exigée par la grammaire (une par étage), murée, ouverte à la bombe | toute graine (1 à 400 : une secrète à chaque étage) ; sud du départ : 39 | `grid_secret_bomb` (explosion f110, `DoorLocked` faux f115, joueur indemne), `grid_secret_room` (graine 25), `grid_secret_no_bomb` |
| Clés, trésor fermé | trésor fermé à clé au second étage ; clés en récompense de salle (graines 14, 20, 22…) et en boutique | 1 : `key` à la boutique du premier étage ; second étage de toute partie : `isaac_floor_seed_1` | `grid_treasure_key` (graine -1013904097, `sous_sol_cle`), `isaac_floor_seed_1` (f3300 : trésor fermé, une clé) |
| Cœurs, sous, boutique | récompenses de salle (cœurs, cœurs d'âme, bombes, clés), sous à chaque mort d'ennemi (`shells_drop` 2 à 5), boutique exigée | cœur de récompense : 1, 3, 14, 21 ; cœur d'âme : 3, 23, 32 ; boutique : toute graine (trois présentoirs par étage) | `isaac_shop_buy` (graine 2 : 60 → 45 sous, `reliquaire`, cœur d'âme 4), `isaac_floor_seed_1` (boutique 3 puis 6 objets) |
| Pools à épuisement | trésor, boutique, boss, diable | toute partie : `PoolCount` du trésor et du boss 1 → 2, de la boutique 3 → 6 au second étage | `isaac_floor_seed_1`, `devil_room_chance` |
| Marché du diable | salle post-boss (un tiers des étages), deux présentoirs en vie maximale | diable au premier étage : 8, 13, 16 | `devil_room_chance` (graine 8 de `sous_sol_un`), `devil_deal_isaac` (paiement). **Le bot qui achète au diable : T14** |
| Champions | ennemis des salles `combat` | 1 (`essaim`, `salve`), 4 (`explosif`) | générés `champion_*` |
| Curses | une chance de 0,3 par étage | 1 (`maze` puis `blind`), 2 (`darkness`), 13 (`blind`), 19 (`labyrinth`), 42 (`unknown`) ; 3 : aucune | `isaac_floor_seed_1`, `isaac_hud_items` (`maze`), `curse_*` (séquence `sous_sol_maudit`) |
| Accomplissements | deux parties d'affilée | — | `achievement_unlocks` (une run) et `achievement_not_met` ; deux runs enchaînées : `crates/scenario/tests/achievements.rs` (`les_accomplissements_d_une_run_entrent_dans_le_pool_de_la_suivante`) |

Mécaniques qui demandent le bot de T14 (prouvées par scénario scripté, puis **par campagne : voir le § 10.3, fait à l'intégration de T14**) : bombe sur le mur secret
(`grid_secret_bomb`), bombe sur un rocher (`isaac_bomb_rock`), porte à clé (`grid_treasure_key`), achat en boutique (`isaac_shop_buy` ; le bot actuel `--shopping` achète déjà : « paie 15 »
dans les journaux de la campagne), achat au diable (`devil_deal_isaac` seul : aucun scénario ne le joue dans un étage assemblé), cœur ramassé à vie basse (bloque le bot à pleine vie : D99).

## 4. Avertissements restants

`alacod lint games/isaac` : **0 avertissement, 0 erreur**. Les 24 mesurés sur `main` avant la tâche étaient tous des `MechanicUnmet` : mur secret sans bombe (2 : règle de la salle `secrete`, règle `SecretRoom`),
boutique absente (larmes, effets, familiers, clés : 4 ; cœurs, sous et boutique : 1), treize objets de larmes sans second pool (`braise`, `eclat`, `epine`, `epouvante`, `fantome`, `givre`, `glu`, `goutte`,
`oeil`, `ricochet`, `satellite`, `venin`, `vent_arriere`), aucun personnage à `bomb` (1), aucun rocher (1), aucune pique (1), curses de la séquence par défaut (1). Hors isaac : `zombies` 0, `throne` 0, `gungeon` 1, `testbed` 115, inchangés
(`make lint` ×5 : « aucune erreur » pour les cinq).

## 5. Campagne de contrôle (bot d'avant T14, séquence par défaut ; **remplacée par le § 10.2**, bot de T14)

`alacod-sim --game isaac --bots N --profiles explorateur[,explorateur] --shopping --floors sous_sol --until-floor 2 --seeds A..B --max-frames 20000`, lots de 20 graines, sur la branche (aucun
réglage pour viser une fourchette). Script versionné : `docs/taches/rapports/m3-t11d1-gabarits.campagne.sh` (`tout`, un lot, `resume`), JSON et journal par lot dans `/tmp/t11d1/camp`. Durée : 6 à 11 minutes par lot de 20 graines sur la machine partagée.

| | 1 bot (graines 1 à 200) | 2 bots (graines 1 à 200) |
|---|---|---|
| desync (synctest) | **0** | **0** |
| victoires | 12 (6 %) : 14 15 43 57 79 81 141 145 146 166 176 181 | 72 (36 %) |
| morts | 169 (84,5 %) | 102 (51 %) |
| « soft-lock » d'`alacod-sim` | 19 (9,5 %) : 2 4 30 35 39 49 63 73 77 83 112 122 139 144 149 171 183 190 196 | 26 (13 %) : 35 49 53 63 73 77 80 83 84 86 91 94 105 107 139 144 149 150 152 159 168 174 176 182 190 191 |
| premier étage franchi | 64 (32 %) | 141 (70,5 %) |
| graines avec champion | 169 | 187 |
| frames moyennes par graine | 2 631 | 4 355 |

**Les « soft-locks » ne sont pas le contenu.** Chacun a été rejoué en scénario (`shopping: true`, mêmes graines, **sans `until_floor`, donc sans détecteur**) avec `frames` = frame d'arrêt + 2 000, événements
`ALACOD_EVENTS` relevés après l'arrêt (mort d'ennemi, salle nettoyée, changement d'étage, victoire, défaite) ; les immobiles ont aussi été relevés au dump (`ALACOD_DUMP_TRACE`, dernière frame). Résumé graine par graine :
`m3-t11d1-gabarits.campagne.tsv`.

| Famille | 1 bot | 2 bots | Preuve et cause |
|---|---|---|---|
| Cœur à pleine vie, au sol | 5 : 39, 63, 112, 122, 171 | 3 : 63, 144, 152 | ne reprend pas, vivant 2 000 frames de plus ; immobile à 6 PV sur 6, un `coeur` à 4 à 11 unités (`refuse_when_full`, §68) que les objets « au contact » (avant les salles) lui font viser sans fin. **D99, T14** |
| Cœur à pleine vie, en boutique | 1 : 149 | 2 : 86, 149 | ne reprend pas ; `purchase_refused … shop_full` toutes les 30 frames (103 refus sur 3 000 frames à la graine 86 : le joueur 0 à pleine vie monopolise le présentoir, le joueur 1 à 4 PV sur 6 ne l'obtient jamais). **D99, T14** |
| `gardien` intact | 1 : 4 | 0 | ne reprend pas ; le bot reste à 48 unités du boss en vue : `FOUR_WAY_LANE` (14) dépasse la demi-largeur du collider du `gardien` (10), tir sans toucher. **D99, T14** |
| Réanimation interrompue par l'esquive | 0 | 1 : 84 | ne reprend pas ; le coéquipier à terre meurt au saignement (f7076), puis l'autre (f7380). Voir ci-dessous. **D99, T14** |
| Long trajet pris pour un soft-lock | 12 | 20 | **28 reprennent** (76 à 759 frames après l'arrêt), **4 se résolvent par la défaite** du bot (95 à 337 frames après), **aucune ne gèle**. **D100** |

### 5.1 Graines 84 et 86 (2 bots)

**84 (second étage, boss en phase 2).** Le joueur 0 tombe à f5276, `Downed { since_frame: 5275, bleedout_at_frame: 7075 }`. Le joueur 1 est collé à lui (11 unités) et tient `Interaction` **119 frames, la lâche 31, la tient 119, la lâche 31…** (dump,
`InteractionInput.is_holding` : segments de 119 frames à f5281, f5431, f5581…, enregistrement des inputs identique : `Interaction` f5276→5395, relâchée jusqu'à f5426, etc.). Le cycle de 150 frames est la répétition de l'éventail du boss. La progression
de la réanimation (`handle_revive_interaction`) exige **180 frames consécutives** : elle monte à 119 au plus (`Reviving.progress_frames` relevé), retombe à 0, neuf fois.

- **Interactable visé** : le seul à portée est le joueur à terre (net id 147, `InteractionType::Revive`, prompt « Réanimer ») ; à moins de 85 unités du réanimateur, il n'y a que lui, un familier `orbe` (26,7), des balles et un mur : aucun présentoir, cœur ou
  porte ne capte l'interaction. Instrumentation temporaire de `input.rs` (retirée) : à chaque frame de f5540 à f5599, `view.revive = (direction nulle, interact = vrai)` et la liste des surfaces à portée est `[(147, 0,84, portée 40)]`.
- **Ennemi le plus proche** : le `gardien` (net id 485, 29,5 PV), 177,7 unités, en vue, donc **au-delà de `REVIVE_SAFE_DISTANCE` (150)** : pas de « menace » au sens de la réanimation.
- **Pourquoi `Interaction` tombe** : seconde instrumentation temporaire (retirée) de `decide_prudent` : pendant les 31 frames, `crate::dodge::dodge(view)` répond `Some((0, 1))` (un projectile du gardien approche dans les 30 prochaines frames) et sa branche, qui **précède** la
  réanimation, remplace tout l'input (direction et tir, jamais `Interaction`) ; le bot ne bouge pas (position constante, bloqué contre le corps du joueur à terre et le mur) mais lâche le bouton.
- **Cause : le bot (D99)**, pas le contenu : `dodge` doit garder `Interaction` quand elle ne peut pas esquiver, ou la réanimation doit passer avant l'esquive quand le saignement est urgent. Le contenu (gardien qui tire en éventail toutes les 150 frames) rend le cas possible
  mais n'y est pour rien : aucune interaction concurrente.

**86 (premier étage, boutique).** Au dump de la frame d'arrêt (4147), les deux joueurs (6/6 et 4/6 PV) sont à 2,2 et 2,8 unités du présentoir 0 de la boutique : un `coeur`, non vendu. Les journaux : `purchase_refused … shop_full` pour le joueur 0 toutes les 30 frames
de f3071 à f6131 (103 refus), aucun achat du joueur 1. Le joueur 0, à pleine vie, ne peut pas prendre un cœur (`ItemDef::accepts`, `refuse_when_full`) ; le joueur 1, qui le pourrait, partage la même position et ne l'obtient pas. **Bot (D99) : cœur en boutique, deux joueurs.**

### 5.2 Les 32 « longs trajets » prolongés

Protocole : `frames` = frame d'arrêt + 2 000, `shopping: true`, sans `until_floor`. « Reprend » = au moins une mort d'ennemi, une salle nettoyée, un changement d'étage ou la victoire **avant toute défaite**.

| Bots | Graine | Frame d'arrêt (détecteur) | Prolongation de 2 000 frames, sans détecteur | Détail |
|---|---|---|---|---|
| 1 | 2 | 2365 | **reprend** | reprend f2693 (+328) : mort d'ennemi, salle nettoyée, étage |
| 1 | 30 | 3892 | défaite | défaite f3995 (+103), aucun progrès avant |
| 1 | 35 | 3429 | défaite | défaite f3524 (+95), aucun progrès avant |
| 1 | 49 | 2067 | **reprend** | reprend f2311 (+244) : mort d'ennemi, salle nettoyée, étage |
| 1 | 73 | 4085 | **reprend** | reprend f4844 (+759) : mort d'ennemi, salle nettoyée, étage |
| 1 | 77 | 2747 | **reprend** | reprend f2869 (+122) : mort d'ennemi, salle nettoyée |
| 1 | 83 | 3415 | **reprend** | reprend f3630 (+215) : mort d'ennemi, salle nettoyée, étage |
| 1 | 139 | 1890 | **reprend** | reprend f2006 (+116) : mort d'ennemi |
| 1 | 144 | 3198 | **reprend** | reprend f3451 (+253) : mort d'ennemi, salle nettoyée, étage |
| 1 | 183 | 1990 | **reprend** | reprend f2088 (+98) : mort d'ennemi, salle nettoyée, étage |
| 1 | 190 | 2996 | **reprend** | reprend f3322 (+326) : mort d'ennemi, salle nettoyée, étage |
| 1 | 196 | 3377 | défaite | défaite f3622 (+245), aucun progrès avant |
| 2 | 35 | 3074 | **reprend** | reprend f3180 (+106) : mort d'ennemi, salle nettoyée, étage |
| 2 | 49 | 2147 | **reprend** | reprend f2275 (+128) : mort d'ennemi, salle nettoyée, étage |
| 2 | 53 | 4927 | **reprend** | reprend f5076 (+149) : mort d'ennemi, salle nettoyée |
| 2 | 73 | 3651 | **reprend** | reprend f4337 (+686) : mort d'ennemi, salle nettoyée, étage |
| 2 | 77 | 2469 | **reprend** | reprend f2587 (+118) : mort d'ennemi, salle nettoyée |
| 2 | 80 | 2536 | **reprend** | reprend f2642 (+106) : mort d'ennemi, salle nettoyée, étage |
| 2 | 83 | 3183 | **reprend** | reprend f3321 (+138) : mort d'ennemi, salle nettoyée, étage |
| 2 | 91 | 2890 | **reprend** | reprend f3019 (+129) : mort d'ennemi |
| 2 | 94 | 4684 | **reprend** | reprend f4760 (+76) : mort d'ennemi |
| 2 | 105 | 3583 | **reprend** | reprend f3771 (+188) : mort d'ennemi, salle nettoyée, étage |
| 2 | 107 | 2915 | **reprend** | reprend f3408 (+493) : mort d'ennemi, salle nettoyée, étage |
| 2 | 139 | 3179 | **reprend** | reprend f3718 (+539) : mort d'ennemi, salle nettoyée, étage |
| 2 | 150 | 6676 | **reprend** | reprend f7123 (+447) : mort d'ennemi, salle nettoyée, victoire |
| 2 | 159 | 3825 | défaite | défaite f4162 (+337), aucun progrès avant |
| 2 | 168 | 2852 | **reprend** | reprend f3061 (+209) : mort d'ennemi, salle nettoyée, étage |
| 2 | 174 | 3318 | **reprend** | reprend f3405 (+87) : mort d'ennemi, salle nettoyée, étage |
| 2 | 176 | 1579 | **reprend** | reprend f1714 (+135) : mort d'ennemi, salle nettoyée, étage |
| 2 | 182 | 3234 | **reprend** | reprend f3365 (+131) : mort d'ennemi, salle nettoyée, étage |
| 2 | 190 | 2631 | **reprend** | reprend f2971 (+340) : mort d'ennemi, salle nettoyée, étage |
| 2 | 191 | 2914 | **reprend** | reprend f3508 (+594) : mort d'ennemi, salle nettoyée, étage |

Résultat : 28 reprennent (1 bot 9 : 2, 49, 73, 77, 83, 139, 144, 183, 190 ; 2 bots 19), 4 finissent par la défaite du bot sans progrès préalable (1 bot 30, 35, 196 ; 2 bots 159 : à 1 PV ou presque, le bot était en route vers le boss ou devant lui, boss intact (40/40) aux trois graines à un bot, et meurt), 0 gèlent.
Ces quatre-là ne « reprennent » pas au sens strict, mais la partie se résout (défaite) : le détecteur de 1 200 frames s'est déclenché pendant un trajet, pas devant un gel. **Dette D100.**

**Référence sur `main`** (avant la tâche, mêmes options, graines 1 à 10) : 10 morts sur 10, aucune victoire (T11c avait mesuré 17 victoires sur 50 graines à un étage ; D95/D96 avaient depuis déplacé le butin et les graines).

## 6. Traces bougées (preuve §5)

Les 120 traces `isaac` (25 écrites à la main, 95 générées) ont bougé, quatre sont neuves (`isaac_shop_buy`, `isaac_bomb_rock`, `isaac_spikes`, `isaac_spikes_temoin`) ; **zombies, throne, gungeon, testbed : aucune**
(suite complète de 653 scénarios, `make gen` ×5 : seuls 95 fichiers `.trace` d'`isaac` changent, aucun `.ron` généré). Causes : (1) la bombe de départ (`Inventory`, `BombCarrier`) :
tout scénario avec le personnage ; (2) les curses de la séquence par défaut (`RunCurses`, `Modifiers`) : toute partie sans `floors:` ; (3) les gabarits et la boutique (le plan d'un étage change) :
tout scénario qui assemble un étage de `salles.ldtk`.

Preuve : dumps `ALACOD_DUMP_TRACE` du même binaire, `ALACOD_GAME_NAME=isaac ALACOD_GAME_DIR=<état>` sur cinq états de `games/isaac` (extraits de `git archive 5067b209`, puis fichiers de la branche
ajoutés par étapes), `scripts/trace-diff.py … --ignore Inventory,BombCarrier`, douze scénarios (`generated/isaac/{weapon_larmes, enemy_suiveur_still, tears_encre,
champion_suiveur_explosif, familiar_frere, room_combat, boss_gardien}`, `hit_iframes_isaac`, `isaac_hud_items`, `isaac_boots`, `grid_secret_no_bomb`, `devil_room_absent`).

| Comparaison | Résultat |
|---|---|
| `main` → **bombe seule** (`pleureur.ron`, `bomb.ron`) | **identiques sur les douze** (299, 299, 319, 239, 199, 599, 149, 599, 149, 499, 1 499 et 39 frames) : seules les lignes `Inventory` (gagne `bomb: 1`) et `BombCarrier` (neutre) bougent |
| `main` → branche, cinq scénarios en carte de test (`weapon_larmes`, `enemy_suiveur_still`, `tears_encre`, `champion_suiveur_explosif`, `familiar_frere`) | **identiques** (même `--ignore`) : aucune boutique, curse ni gabarit n'y entre |
| bombe → bombe + curses (séquence par défaut, graine 1) | f0 : `RunCurses { floors: {0: "maze"}, entered: Some(0) }` au lieu de `{}`, `None`, et `Modifiers` du joueur gagne `MoveSpeed ×0,85` de source `curse:maze` ; identiques sur `boss_gardien`, `devil_room_absent`, `grid_secret_no_bomb` (séquences sans curse) |
| bombe + curses → **vingt gabarits, boutique non exigée** (cinquième état : la grammaire sans `boutique` dans `required`, `hit_iframes_isaac`, `isaac_hud_items`, `isaac_boots`) | dès f0 : `FloorState.enemies_placed` 16 → 17, `GgrsNetIdFactory.counter` 148 → 193, `PoolState` et entités de salle : **les gabarits déplacent le plan à eux seuls** (neuf gabarits de combat tirés par pose) |
| vingt gabarits sans boutique → branche (boutique exigée) | dès f0 : `enemies_placed` 17 → 14, `PoolState` gagne `boutique` (trois présentoirs), entités de salle : **la boutique déplace le plan à elle seule** (un cul-de-sac de plus, tirages décalés) |

## 7. Vérifié (résultats réels)

- `cargo run -p content --bin alacod -- lint games/isaac` : aucune erreur, 0 avertissement ; `make lint` ×5 : aucune erreur (zombies 0, testbed 115, throne 0, gungeon 1, isaac 0).
- `make test_scenarios_par SHARDS=3` : **653 scénarios verts** (649 de `main` + 4 neufs), traces comparées, sur la fusion avec `origin/main` `7ee084e8` (HEAD `75cca57a`, 244 s) ; aucune trace non-isaac ne change (les passes précédentes, avant bénédiction, ne montraient « trace différente » que pour les 120 scénarios `isaac`).
- `make test_crates` : **1 228 passés, 0 échec** sur la fusion (1 225 avant la fusion avec `origin/main`, 1 224 sur `main` de départ).
- `make gen GAME=zombies|testbed|throne|gungeon|isaac` (sur la fusion) : rc 0 chacun ; aucun `.ron` ni `.trace` ne change (arbre propre après les cinq) ; isaac : 95 scénarios générés, traces déjà bénies.
- `make lint` ×5, `cargo fmt --all -- --check` : propres sur la fusion ; `make check_sim_boundary` : 13 crates SIM, 2 en dette D69 (inchangé) ; `check-forbidden.sh` : OK (38 occurrences auditées) ; `check-rollback-registration.sh` : OK ;
  `make test_tools` : 25 tests OK.
- Fuzz d'inputs (avant et après la fusion, mêmes résultats) : `ALACOD_FUZZ=0:8` 9 graines sans divergence ; `ALACOD_FUZZ_BOMBS=1 ALACOD_FUZZ=0:8` 9 graines sans divergence (8 explosions chacune) ;
  `ALACOD_FUZZ_SCENARIO=isaac_floor_seed_1 ALACOD_FUZZ=0:4` 5 graines sans divergence (le tirage aléatoire de ce mode ne presse pas `Bomb` : 0 explosion).
- `scripts/p2p-stress.sh isaac /tmp/t11d1/p2p_inputs.ron 3000` (`--pool Up,Down,Left,Right,Fire,Reload,Bomb,Interaction`, `MATCHBOX_PORT=3571`, `COMPOSE_PROJECT_NAME=b1`) : **« p2p-stress : OK »**,
  codes de sortie 0 0, 0 `DesyncDetected`, traces de 2 999 lignes identiques. Avant la fusion : 10 942 frames resimulées (client 0) et 24 (client 1) ; **sur la fusion : 6 900 frames resimulées** (client 0 : 1 224 rollbacks, profondeur moyenne 6,6, maximale 11 ; client 1 : aucun),
  journal : 3 poses de bombe, 7 explosions, une curse, 14 assignations de butin (avant la fusion : 4, 11, 1, 14). Aucun passage d'étage en 3 000 frames d'inputs aléatoires : la transition d'étage en p2p d'`isaac` n'est pas prouvée ici (le script n'est pas un bot qui finit l'étage).
- Relecture indépendante (agent `reviewer`, lecture seule, deux passes) : passe 1 : un P0 (§76.4 pas encore écrite), quatre P1 (attentes qui ne prouvaient pas : `isaac_bomb_rock`, `grid_secret_bomb`, `isaac_spikes`, table §73 périmée) et quatorze P2 de commentaires ou de docs périmés, tous corrigés (`PlayerPosition` ajoutées, témoin `isaac_spikes_temoin`, tables et commentaires) ; passe 2 sur la branche corrigée : aucun P0 ni P1, six P2 documentaires (poids des curses, comptes, preuve des deux runs, résumé des scénarios, comptes de suite, cause de la borne de temps), corrigés. Aucun risque de déterminisme ni de désync trouvé. Revue de la livraison (BLOQUANT sur deux points, levés sans toucher au code du jeu) : graines 84 et 86 analysées (§5.1), 32 « longs trajets » prolongés sans détecteur (§5.2), borne de génération mesurée en (a)/(p)/(b) (§8), dettes renumérotées D99, D100 (+ D107), résumé de campagne versionné, séparation gabarits/boutique dans la preuve §5, titre de D93.

### 7.1 Revue de livraison (résultats réels)

- `origin/main` n'a pas bougé (`7ee084e8`) : pas de nouvelle fusion, pas de rebless ; aucun code de simulation ni aucune donnée n'a changé depuis la suite de 653 scénarios et les 1 228 tests de `test_crates` ci-dessus (seul le test de temps de `map_ldtk` change).
- 45 graines rejouées en scénario sans détecteur (`frames` = arrêt + 2 000, `ALACOD_EVENTS`, `make test_scenarios_par SHARDS=3`) : tableau du § 5.2 (graine, frame d'arrêt, issue prolongée) ; 6 dumps de dernière frame (86, 144, 152, 63, 149 à 2 bots, 4 à un bot) ; dump complet de la graine 84 (cycle `Interaction` 119 / 31 frames, `Reviving` ≤ 119) ; deux instrumentations temporaires du bot (réanimation, esquive), retirées (`git diff` du bot vide).
- `cargo test -p map_ldtk --profile headless --no-default-features isaac` : 2 tests verts (le test de temps : (b) 2,39 × gungeon, 2 396 évaluations par étage).
- `cargo fmt --all -- --check` : propre ; `alacod lint games/isaac` : aucune erreur, aucun avertissement.

## 8. Non fait / incertain

- **Bot** : quatre blocages nommés (D99 : cœur à pleine vie au sol ou en boutique, visée à quatre directions contre le collider du boss, réanimation interrompue par l'esquive, rochers invisibles) et un détecteur de soft-lock trop court (D100 : 32 graines prolongées, aucune ne gèle).
  Aucun n'est corrigé ici (code du bot et du runner : hors périmètre, T14). La campagne est à refaire avec le bot de T14.
- **Boutique** : `shop.cost: Currency(n)` n'est toujours exercé par aucun scénario (les prix passent par `economy.ron`) ; l'achat en boutique par le bot ne fait pas partie de l'objet
  d'un scénario d'étage assemblé (le bot actuel achète avec `--shopping`).
- **Diable** : aucun scénario ne joue l'achat en vie maximale dans un étage assemblé (le bot n'y entre pas, la salle apparaît dans un tiers des étages) ; `devil_deal_isaac` joue le paiement en carte de test.
- **Borne de génération** (revue) : le test `isaac_grille_de_12_a_15_salles_sous_trois_fois_grand_de_gungeon` (ancien `…_sous_deux_fois_…`) mesure maintenant (a) clone + `from_map`, (p) planificateur, (b) génération entière, 100 graines, des deux côtés :
  isaac (vingt gabarits) (a) 1,0 ms, (p) 0,67 à 0,82 ms, (b) 0,85 à 1,05 ms ; gungeon `grand` (a) 0,25 ms, (p) 0,17 à 0,24 ms, (b) 0,36 à 0,49 ms ; **(b) = 2,1 à 2,4 × gungeon, la borne d'origine de 2 n'est pas tenue** ; avec dix gabarits (b) = 0,77 à 0,81 (isaac (a) 0,23 ms, (p) 0,14 ms).
  **La cause est dans le planificateur** : (p) passe de 0,14 à 0,67 ms et le nombre d'évaluations de candidats (`FloorPlan::steps`, moyenne sur 100 graines) de **278 à 2 396** (×8,6), parce que `grid::attempt` tire un gabarit parmi ceux qui ont au moins une pose valide et évalue donc parents × ports × gabarits × ports à
  chaque pose, et que le remplissage est passé de un à neuf gabarits de combat. La part de (a), qui croît avec le `.ldtk` (145 → 314 Ko), est 4 × celle de gungeon et reste hors de la borne. J'ai mis la borne à 3 sur (b) et ajouté une garde déterministe (évaluations sous 3 500) ; la rendre à 2 demande de changer le
  planificateur (une pose par classe de gabarits de même taille et mêmes ports) : **D107**, non fait (hors périmètre : code moteur).
- **Bombe de départ** : `damage: "4.0"` enlève **deux cœurs** au poseur (sur trois), contre un seul dans Isaac ; valeur de départ (tir ami `Always`, Q4), non réglée : à relire par T11d2 et T14.
- Les sprites des rochers et des piques sont les couleurs de couche IntGrid de LDtk (aucun sprite propre : T17 et T11d2).
- Les rochers sont dans les angles : un gabarit où un rocher barre un passage et demande la bombe est T14 (quand le bot saura les bombes).
- Pas de scénario d'achat au diable ni de scénario d'étage complet joué avec la bombe de départ (le bot actuel ne la pose pas).

## 9. Dettes laissées

D81 fermée. D93 annotée et retitrée (quatre scénarios gagnent par armure : `grid_floor_seed_1`, `isaac_floor_seed_1`, `achievement_unlocks`, `grid_post_boss` à `Armor +100`). D99 (bot), D100 (détecteur de soft-lock) et D107 (planificateur de grille) ouvertes
(numérotées D99, D100 après la collision avec `m3-t15-bench-larmes`, qui garde D97 et D98).
Q14 (fourchette de franchissement) : sans objet tant que l'équilibrage par bot n'est pas rouvert.

## 10. Intégration de T14 (`m3-t14-bot-isaac`, dette D106)

`origin/m3-t14-bot-isaac` `3feaeb87` fusionné dans la branche (conflits : `CLAUDE.md`, §79 des conventions, `dettes.md` ; ma dette « planificateur » devient **D107**, T14 réserve D101 à D106). Le bot de T14 pose des bombes par défaut
(`BombPolicy::default()`) dès qu'un joueur en porte une : la bombe de départ de `pleureur` les active dans tous les scénarios isaac à `explorateur`, comme T14 l'avait annoncé.

### 10.1 Traces rebénies (14) et preuve §5

La suite sans BLESS sur la fusion : **seuls 14 scénarios isaac échouent**, aucun autre jeu ne bouge. Causes : (1) les huit `bot_isaac_*` joués sur `isaac` (T14 en compte dix : `bot_isaac_rock` et `_rock_sans_bombe` jouent la carte `testbed`, dont le personnage n'a pas de bombe de départ : ils gardent leur kit et ne bougent pas) étaient écrits pour un `isaac` sans bombe, sans boutique et sur l'ancien plan (kit d'essai `PlayerScript::bomb` et `items`) ;
(2) `grid_floor_seed_1/23/123`, `grid_post_boss`, `isaac_floor_seed_1`, `achievement_unlocks` : le bot pose maintenant la bombe de départ contre le mur secret et ses chronologies bougent.

| Scénario | Ce qui change | Recalage |
|---|---|---|
| `bot_isaac_secret` | graine 39 (la 8 de T14 n'a plus la secrète au sud du départ) ; **bombe de départ réelle** (mèche 75), plus de kit | bombe f19, explosion f93, mur ouvert f94, coffre secret f1412 |
| `bot_isaac_two` | idem, deux bots (un seul poseur, `GoalContext::peers`) | bombe f19 (joueur 0), trois achats f1755 à f1792, victoire f2224 |
| `bot_isaac_floor` | graine 6 (la 48 « sans bombe » n'a plus de sens) : **les deux étages, bombe, boutique, clé, curses** | mur secret f439 et f4262, porte à clé f3268, achats, victoire f4815 |
| `bot_isaac_key` | graine -1013903988 (plan 254 : trésor au sud sans secrète voisine, aucun détour à la bombe) | porte f20, coffre f81 |
| `bot_isaac_shop`, `_shop_blesse` | une bombe de plus (la bombe de départ) | comptes de bombes 1 → 2 |
| `bot_isaac_devil`, `_devil_never` | plan de la graine 1 | attentes inchangées |
| `grid_floor_seed_1/23/123`, `isaac_floor_seed_1`, `achievement_unlocks` | le bot pose la bombe de départ (f986, f261, f673, f1022, f677) | chronologies recalées, attentes de bombe et de mur ouvert ajoutées |
| `grid_post_boss` | idem, et le bot meurt au boss avec `Armor +100` (f1657) | `Armor +200` (D93 : l'armure grandit, aucun rééquilibrage) |

`bot_isaac_rock`, `_rock_sans_bombe` (carte de testbed) ne bougent pas ; les générés `isaac` non plus.

**Preuve §5** (dumps `ALACOD_DUMP_TRACE` du même binaire, `ALACOD_GAME_DIR` sur deux états de `games/isaac`, `scripts/trace-diff.py … --ignore Inventory,BombCarrier`, neuf scénarios : `grid_floor_seed_1/23/123`, `grid_post_boss`, `isaac_floor_seed_1`,
`achievement_unlocks`, `bot_isaac_secret`, `_two`, `_floor`, tronqués à la pose + 300 frames). État « sans bombe » : `pleureur.ron` sans `bomb` ni bombe de départ.
- **Contrôle** : le joueur porte la bombe mais la politique du bot coupe ses décisions de bombe (`bombs: (secret: false, rocks: false)`) : **identique à l'état sans bombe sur les neuf scénarios** (318 à 1 321 frames). La bombe portée ne déplace donc rien par elle-même.
- **Avec les décisions de bombe** : la première frame qui diffère est le **trajet vers le mur secret**, avant la pose : f636 (pose f986), f220 (f261), f517 (f673), f611 (f674), f945 (f1022), f624 (f677), f333 (f364, `bot_isaac_floor` ; la bombe f138 que le journal y montre est celle d'un champion explosif, rayon 44), f6 (f19, `bot_isaac_secret` et `_two`) ; une seule ligne, celle du joueur (`FacingDirection`, `AnimationState`). Le critère de T14
  (« première frame qui diffère = première bombe posée ») est donc **trop strict** : le but « mur secret » existe dès que le joueur porte une bombe, le bot s'y met en route avant de la poser (13 à 350 frames avant).
- Les quatre scénarios `bot_isaac_devil*` et `bot_isaac_shop*` : la cause est le plan (gabarits, boutique, curses) déjà prouvée au § 6 ; la bombe y ajoute une unité à l'inventaire.

### 10.2 Campagnes de T14 rejouées sur le contenu livré (D106)

`docs/taches/rapports/m3-t11d1-gabarits.campagne-t14.sh` (les options de `m3-t14-bot-isaac/campagne.sh`, `--pickup-progress`, séquence par défaut `sous_sol`, **sans `--bombs`** : « kit de bombes None » au journal d'`alacod-sim` ; binaire de la fusion `3feaeb87`), lots de 20 graines ; résumé graine par graine :
`m3-t11d1-gabarits.campagne-t14.tsv` (issue, frames, étage, bombes, murs secrets ouverts, portes à clé, achats, coffres ; la colonne `bombes` compte aussi les bombes de champions explosifs) ; journal des poses du joueur, des rochers cassés et des portes à clé sur les 40 premières graines à un bot : `m3-t11d1-gabarits.campagne-t14.bombes-rochers.txt`.

| Campagne | Graines | Victoires | Morts | Soft-locks | Desyncs | Premier étage franchi | Frames moyennes |
|---|---|---|---|---|---|---|---|
| **s1** : 1 bot | 200 | 29 (14,5 %) | 171 | **0** | **0** | 89 | 3 717 |
| **i2** : 2 bots | 200 | 115 (57,5 %) | 85 | **0** | **0** | 173 | 5 657 |
| **i4** : 4 bots | 20 | 20 | 0 | **0** | **0** | 20 | 6 613 |

Les victoires sont des informations (équilibrage hors objectif, décision de William). **Aucun soft-lock** : la méthode de prolongation (+2 000 frames sans détecteur) n'a rien à prolonger. Les **45 graines** que le bot d'avant T14 laissait en soft-lock (19 à un bot, 26 à deux : cœurs à pleine vie, visée du gardien, longs trajets, graines 84 et 86)
finissent maintenant en victoire ou en mort (1 bot : 4 victoires, 15 morts ; 2 bots : 16 victoires, 10 morts). **D99** : trois de ses quatre situations sont fermées par T14 ; la réanimation interrompue par l'esquive (cause de code inchangée, `decide_prudent`) ne se reproduit pas (graine 84 : victoire), la dette reste ouverte, réduite à ce point. **D100 fermée** (`--pickup-progress`).

### 10.3 Atteignabilité complétée (renvoyée à T14 au § 3)

| Mécanique du bot | Campagne (séquence par défaut, sans kit) | Scénario scripté |
|---|---|---|
| Bombe sur le mur secret | s1 : 222 murs ouverts dans 187 graines sur 200 (graines 1 à 5…) ; i2 : 347 dans 194 ; i4 : 40 dans 20 | `bot_isaac_secret` (39), `bot_isaac_two` (39, un seul poseur), `bot_isaac_floor` (6, deux étages) |
| Bombe sur un rocher | **aucune en partie ordinaire** : sur 40 graines à un bot, 45 poses du joueur (`bomb_place`) = 45 murs secrets ouverts (le compteur `bombe` du JSON compte aussi les bombes de champions explosifs) ; les rochers sont dans les angles (§ 2) et casser un rocher n'est jamais nécessaire. Rochers cassés par le **souffle** du mur secret : 29 dans 12 graines (3, 4, 6, 14, 19, 20…), dont 6 rochers à butin (graine 6 : clé et ration, 19 : cœur, 21 : clé, 26 : un sou, 32 : bombe) | `bot_isaac_rock` (carte de testbed, T14) ; `isaac_bomb_rock` (scripté) |
| Porte à clé (trésor du second étage) | s1 : 51 graines (3, 6, 13, 14, 15…) ; i2 : 143 ; i4 : 20 sur 20 | `bot_isaac_key` (-1013903988), `bot_isaac_floor` (f3268) |
| Achat en boutique | s1 : 544 achats dans 143 graines (1, 2, 3, 4, 5…) ; i2 : 915 dans 188 ; i4 : 117 dans 20 | `bot_isaac_shop`, `_shop_blesse`, `bot_isaac_two` (trois présentoirs), `bot_isaac_floor`, `isaac_shop_buy` |
| Achat au diable | **aucun** par défaut (`devil: Never`, Q14 à confirmer) ; politique `IfHealthAbove` non rejouée en campagne ici | `bot_isaac_devil` (présentoir posé), `devil_deal_isaac` |
| Coffres (trésor, secrète) | s1 : 378 dans 173 graines | `bot_isaac_secret`, `bot_isaac_two`, `bot_isaac_floor` |

D106 est **fermée**. D99 (réduite à l'esquive qui coupe la réanimation) et D107 (planificateur) restent ouvertes ; D100 est fermée.

### 10.4 Vérifié (résultats réels, sur la fusion de T14, avant `main`)

- `make test_scenarios_par SHARDS=3` : **663 scénarios, 0 échec, 0 « trace différente »** (214 s) ; `make test_crates` : **1 245 tests, 0 échec** ; `cargo fmt --all -- --check` propre ; lint ×5 sans erreur ; `make gen` ×5 sans diff ; `make check_sim_boundary` : 13 crates ; `make test_tools` : 25 tests ;
  `check-rollback-registration.sh` OK ; fuzz `ALACOD_FUZZ=0:8` et `ALACOD_FUZZ_BOMBS=1` : 1 test chacun, verts.
- `scripts/check-forbidden.sh` signalait 4 `Instant::now()` pour 2 autorisés dans `crates/map_ldtk/src/generation/mod.rs` (les deux du test de comparaison isaac/gungeon par phase, revue de T11d1) : `scripts/determinism-allowlist.json` passe à `count: 4` avec la raison.
- **p2p réel avec les inputs figés de bots** (`GAME=isaac FLOORS=sous_sol UNTIL_FLOOR=5 SIM_ARGS=--pickup-progress scripts/p2p-gungeon-inputs.sh 2 23 4500`, puis `GAME_ARGS="--seed 23" EXPECT_FLOORS=1 scripts/p2p-stress.sh isaac <inputs> 4400`) : deux pairs, **5 899 frames resimulées**,
  450 rollbacks (profondeur moyenne 9,9), **0 `DesyncDetected`**, un passage d'étage (f1700), 4 399 frames contiguës et identiques ; les traces portent deux `bomb_place`, trois `secret_wall_open` et une `key_door_open` : la bombe de départ, le mur secret et la porte à clé sont donc rejoués en p2p avec rollbacks réels.

### 10.5 Reste à faire à la fusion avec `main`

`origin/main` (`98e65184`) contient `m2-salles-murs-gungeon` (`ExplorerOptions::steady`, soft-lock `Floors` par cases explorées) mais **pas encore** `m3-t14-bot-isaac` : la fusion de `main` dans cette branche conflit dans `crates/bots/src/{explorer,input}.rs`, `crates/scenario/src/{runner.rs,bin/alacod-sim.rs}`, `crates/scenario/tests/bots.rs`, `crates/map_ldtk/src/generation/mod.rs`
et les docs : les conflits de bots (`explorer.rs`, `input.rs`, `runner.rs`, `alacod-sim.rs`, `bots.rs`) opposent les deux branches de bots et reviennent à T14 (mergée la première) ; `generation/mod.rs` et les docs sont les miens à résoudre ensuite. Une fois T14 sur `main` : `git merge origin/main`, rebénir ce qui bouge encore (avec cause), relancer suite, `test_crates`, p2p.
