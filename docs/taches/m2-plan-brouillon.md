# M2 `gungeon` — plan détaillé (brouillon)

**Brouillon** écrit par b0 (m1-m2-plan-brouillon, base `6a8470c`, après la vague 2 de M1 et les
lots D38–D44), à relire par l'orchestrateur et William avant de remplacer le squelette de
`docs/taches.md` §7. Sources : `docs/plan-engine.md` (§5 chantiers B5 v2, B6, C1 v2, C2, C4,
C5, D4, E1, E2, G1, G2, I1, I2, K2 ; ligne M2 du tableau §6 ; §9.8 critères de sortie) et
l'état réel de l'engine. Numérotation des tâches : **T2.x** comme demandé ; attention, M0 a déjà
eu des T2.x (vague 2 de M0) : au journal, préfixer `M2-` en cas d'ambiguïté.

**But** (plan §6) : étages de salles typées et verrouillées, bullet hell, roulade à i-frames,
blanks, coffres, clés, boutique, objets passifs et actifs, synergies, boss à phases et arènes à
tenir, hub et déblocages. Taille XL, **huit à dix semaines** indicatives.

## 0. Ce qui existe déjà, ce qui manque (état après M1)

| Besoin M2 | Déjà là (M0/M1) | Manque |
|---|---|---|
| Bullet hell | `Emitter` rollback, `Aimed`/`Spread`/`Ring`/`Sequence`/`Telegraph`/`Wait`, six modificateurs de projectile, `on_hit`/`on_expire`, grille spatiale (B4), bench à ~155 balles | spirale (angle qui tourne), rebond **sur murs** des balles ennemies en masse, blanks (efface les balles ennemies), bench à 500 balles en salle |
| Roulade à i-frames | dash (`DashState`), `Health.invulnerable_until_frame` lu par la résolution des dégâts | aucune i-frame posée par l'esquive ; charges ; variantes d'esquive en données (B6) |
| Effets et objets | `Effect { on, if, do }` exécuté pour `OnKill`, `OnDamageTaken`, `Tick`, `OnGauge`, `OnLevelUp` (conditions `HpBelow`, `HasTag`, `TargetTag`, `NotHitFor`) ; jauges ; power-ups ; `RefillAmmoOf` ; mutations (choix à trois cartes) | `OnHit`, `OnDodge`, `OnReload`, `OnRoomClear`, `OnPickup`, `OnUse`, `OnEvent` et la condition `Carrying` (contrats seulement, lint `Unsupported`) ; inventaire d'objets, actifs à cooldown par salles, consommables, synergies (C2) |
| Butin et économie | pools pondérés de power-ups (flux `loot`), monnaie, achats muraux, perks | coffres, clés, rareté, tables par salle et par ennemi (C4) ; boutique avec étal et prix (C5 v2) |
| Ennemis et boss | behaviors composables (8), variantes, navigation par profil et par taille, boss simple en données | phases par seuil/timer, timelines, arènes à tenir (D4) ; formations simples (D1 v2) |
| Monde | `Floors` (séquence de niveaux, portail, boucle), cavernes générées (une caverne = un asset), surfaces, cartes LDtk à la main (zombies) | **salles typées** et `RoomState`, verrouillage des portes sur les présents, activation par salle (E1) ; **grammaire d'étage** en graphe de gabarits LDtk, minicarte, transition (E2) ; tables, barils, fosses (E4 v2) |
| Méta | — | profil sauvegardé, déblocages (G1), hub (G2) |
| Présentation | HUD v1 (zombies), HUD throne, écran de mutation, feedback v1 | I1 v1 : choix du toolkit d'UI (plan §7.1), inventaire, minicarte, pause |
| Outillage | générateur v1 (armes, ennemis), bots `prudent`/`fonceur`/`chasseur`/`acheteur` qui naviguent par le champ, `alacod-sim` (Floors, fin de partie), diagnostic de soft-lock D42 | bot **explorateur** qui ouvre des salles et ramasse (K2 v1), gabarits de salles au générateur, attentes `RoomState`, `Inventory`, `BossPhase`, `ProfileHas` |

**Inconnues honnêtes** : (1) le choix du toolkit d'UI (plan §7.1) n'est pas fait et conditionne
I1 ; (2) la grammaire d'étage (E2) est le plus gros risque technique (génération par contraintes
déterministe, sur gabarits LDtk, avec continuité des net ids sur plusieurs salles chargées) ;
(3) les i-frames de la roulade touchent la résolution des dégâts (traces de tous les jeux
possiblement touchées si le dash existant en reçoit par défaut : à poser en données, défaut 0) ;
(4) le coût de 500 balles sous synctest à 4 joueurs n'est pas mesuré ; (5) la sauvegarde (G1) est
le premier état **hors** simulation persistant : format, version, emplacement en wasm.

## 1. Vague 0 : contrats (sériel, ~5 j)

- **T2.0a Contrats de salles et d'étage** — V1d. `crates/world` (ou nouveau `crates/rooms`) :
  `RoomKind` (combat, coffre, boutique, boss, départ, secret, hub), `RoomState` (dormante,
  active, verrouillée, nettoyée) rollback **neutre**, `Entrance` (porte, côté, salle voisine),
  graphe d'étage (`FloorGraph`), `RollbackSystemSet::Rooms`. Acceptation : enregistrés sans
  porteur, traces identiques ; unitaires de transitions d'état.
- **T2.0b Contrats d'objets et d'inventaire** — V1b. `ItemKind` (passif, actif, consommable, clé,
  monnaie), `Inventory` à emplacements (actif unique, passifs en liste, consommables à pile),
  `ItemDef { tags, effects, active: Option<{ cooldown_rooms | cooldown_frames, charges }> }`,
  kind RON `Item` au registre et au lint ; `Synergy { requires: [ids|tags], gives }`. Acceptation :
  lint de fixtures, traces identiques.
- **T2.0c Contrats de boss** — V1c. `BossPhases { phases: [{ until: HpBelow(x) | Frames(n),
  behaviors, emitters, arena: Option<ArenaScript> }] }`, `ArenaScript` (spawns, horloge, « tenir N
  frames »), état rollback neutre. Acceptation : un boss de test à deux phases dans le testbed.
- **T2.0d Contrats de méta** — V1e. `crates/meta` : `Profile { version, unlocks, currency_meta,
  stats }`, événements de fin de partie → écriture hors simulation ; aucune lecture dans la
  simulation sauf au démarrage (pools enrichis). Acceptation : profil vide = comportement actuel.
- **T2.0e Squelette `games/gungeon`** — V2. Manifeste, un personnage (roulade), une arme, un
  ennemi, deux gabarits de salles reliés, mode `Floors` à salles (une salle = un emplacement) :
  le clone démarre vide.

## 2. Vague 1 (parallèle, ~4 semaines)

**V1a combat**
- **T2.1 B5 v2** (4 j) : patterns `Spiral { arms, step, every }` et éventail tournant, rebond
  des balles ennemies sur les murs en masse, **blank** (action qui efface les projectiles ennemis
  dans un rayon et repousse), balles qui se détruisent sur un blank. Acceptation : `enemy_spiral`,
  `blank_clears`, `BulletCount` ; **bench à 500 balles** dans une salle verrouillée, budget fixé.
- **T2.2 B6** (4 j) : roulade à i-frames en données (`dodge: (distance, frames, iframes,
  charges, recharge)`), défaut 0 i-frame pour les jeux existants (traces intactes), `OnDodge`
  branché. Acceptation : `dodge_iframes` (une balle traverse le joueur pendant la roulade),
  `NoDamageBetween`.

**V1b effets et objets**
- **T2.3 C1 v2** (4 j) : `OnHit`, `OnReload`, `OnRoomClear`, `OnPickup`, `OnUse`, `OnEvent`
  exécutés ; condition `Carrying` (objet porté, pour les synergies). Acceptation : un scénario par
  déclencheur, le lint ne les marque plus `Unsupported`.
- **T2.4 C2** (5 j) : inventaire, ramassage au sol, actif à cooldown par salles nettoyées ou par
  frames, consommables, synergies (paire → objet transformé ou effet en plus), polarité par tags.
  Acceptation : `item_active_cooldown`, `synergy_pair`, attente `Inventory`.
- **T2.5 C4** (3 j) : coffres (qualité, clé requise), clés, rareté, tables par salle et par
  ennemi (flux `loot`), déblocages qui enrichissent les pools. Acceptation : `chest_key`, tirage
  identique à 1 et 4 joueurs.
- **T2.6 C5 v2** (2 j) : boutique (étal d'objets à prix, monnaie d'étage), réutilise
  `Currency`. Acceptation : `shop_buy`.

**V1c ennemis**
- **T2.7 D4** (5 j) : phases, timelines, arène scriptée (verrouillage, spawns, « tenir N
  frames »), barre de boss (présentation). Acceptation : `boss_two_phases`, attente `BossPhase`.
- **T2.8 D1 v2** (2 j) : formations simples (groupe qui garde une forme), tir en salve
  coordonné. Acceptation : `formation_line`.

**V1d monde**
- **T2.9 E1** (5 j) : salles typées depuis les métadonnées de gabarit LDtk, `RoomState`,
  verrouillage des portes quand un joueur entre dans une salle de combat (sur les présents),
  activation des entités de la salle, récompense à la sortie. Acceptation : `room_lock_clear`,
  attente `RoomState`.
- **T2.10 E2** (8 j, **risque**) : grammaire d'étage en graphe sur gabarits (types obligatoires,
  distances, branches, secret), génération seedée, minicarte (présentation), transition d'étage ;
  plusieurs salles chargées (réutilise le chargement multi-monde de `Floors` et `cave://`).
  Acceptation : 1 000 graines → graphe valide (unitaire), `floor_graph_walk`.
- **T2.11 E4 v2** (2 j) : tables renversables (couvert), barils explosifs, fosses. Acceptation :
  `barrel_explodes`, `pit_fall`.

**V1e run et méta**
- **T2.12 G1** (3 j) : profil versionné, écriture en fin de partie, lecture au démarrage,
  déblocages. Acceptation : `ProfileHas` après un run de scénario ; migration de version testée.
- **T2.13 G2** (3 j) : hub « la Brèche » (carte LDtk), services activés par drapeaux du profil,
  entrée en partie. Acceptation : `hub_enter_run`.
- **T2.14 F1 v2** (2 j) : `Floors` à salles (une salle = un emplacement, portail par salle de
  boss), résumé de run enrichi (objets, salles).

## 3. Vague 2 (contenu, outillage, présentation ; ~2 semaines, en partie parallèle à la vague 1)

- **T2.15 V2 contenu `gungeon`** (8 j) : vingt armes, vingt objets (dont cinq actifs et trois
  synergies), dix ennemis (dont trois tireurs en patterns), un boss à deux phases, quinze gabarits
  de salles, tables de butin, boutique ; `test:` sur chaque définition ; lint vert.
- **T2.16 V2 lint des nouveaux kinds** (2 j) : `Item`, `Synergy`, `BossPhases`, `RoomTemplate`,
  références croisées, cycles de synergies.
- **T2.17 V3 générateur v2** (3 j) : gabarits « objet seul », « salle vide », « boss en arène » ;
  attentes `RoomState`, `Inventory`, `BossPhase`, `ProfileHas` (T1.15 en exemple).
- **T2.18 V3 bot explorateur K2 v1** (5 j) : ouvre les salles dans l'ordre du graphe, ramasse,
  utilise l'actif, prend la clé du coffre, finit un étage ; `alacod-sim --until-floor` en salles.
- **T2.19 V4 I1 v1** (5 j, **après décision du toolkit**) : test comparatif de deux jours (plan
  §7.1), puis HUD (cœurs, blanks, clés, monnaie, actif), pause, inventaire, minicarte.
- **T2.20 V4 I2 v2** (2 j) : télégraphes de zone des boss, particules minimales, écran de mort
  avec résumé (prolonge T1.17).

**Ordre de merge** : V3 → V2 → V1 (V1a, V1c, V1b, V1d, V1e) → V4, comme M1. E2 en dernier de V1d
(il réutilise E1).

## 4. Vague 3 : intégration (~1 semaine)

Scénarios du clone `gungeon_solo`, `gungeon_duo`, `gungeon_quad` (un étage complet : salles,
coffre, boutique, boss à deux phases), bots sur 200 graines, vidéos, revue humaine, fermeture des
notes, digest `m2-fin-de-vague-3`.

## 5. Critères de sortie de M2

Ceux du plan (§9.8) : lint et tests verts sur `main` ; tous les scénarios du clone en synctest à 2
et à 4 ; les bots finissent le clone (un étage) sur 200 graines sans soft-lock ni desync ; bench
dans les budgets, **500 balles** comprises ; vidéos publiées ; conventions à jour ; notes de M1
fermées. Propres à M2 : un profil créé par une partie et relu par la suivante (déblocage visible
dans le pool) ; une partie entrée depuis le hub.

## 6. Ce qui reste flou (à trancher avant la vague 0)

- **Toolkit d'UI** (plan §7.1) : bloque T2.19 ; à décider pendant la vague 0.
- **Une salle = un emplacement de `Floors`, ou un étage = un monde LDtk assemblé ?** La première
  réutilise `Floors` et `cave://` tels quels mais charge beaucoup de mondes ; la seconde
  rapproche E2 du générateur `map` existant (assemblage de salles du testbed). Prototype d'un jour
  en vague 0 recommandé.
- **Bots** : la remesure après les correctifs de b1 (portail, réanimation, cible hors de vue) dira
  si K2 v1 part de `prudent` ou d'un nouveau profil.
- **Sauvegarde en wasm** (G1) : stockage du navigateur ou export ; aucune décision.
- **Dettes à fermer avant** : les notes de la revue de M1, D40 (`grunt`), et la règle des
  points d'apparition pour les gros agents (D41, faite).
