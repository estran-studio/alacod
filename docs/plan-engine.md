# Plan de l'engine alacod

> Proposition du 2026-09-28. Compare ce que l'engine sait faire à ce que demandent les six jeux de
> référence de 1837 (`1867_lore/CLAUDE.md` : Enter the Gungeon, Binding of Isaac, Nuclear Throne,
> Risk of Rain, Hades, Call of Duty Zombies), puis propose un découpage en jalons. Chaque jalon se
> valide par un **clone minimal** d'une référence, joué en scénarios headless et en vidéos.
>
> Décisions du 2026-09-28 (§7) : vue de dessus seulement ; RON au maximum, plugins Rust sinon ; un
> jeu est un crate avec son `main.rs` et son contenu ; Risk of Rain analysé mais pas cloné ; pas de
> Steam ; l'UI reste à trancher (§7.1). La stratégie de tests et la boucle de travail sans
> validation humaine sont au §9.

## 1. Le principe : l'engine se prouve par des clones

- **Deux dépôts, deux rôles.** `alacod` est l'engine : des modules Rust déterministes (GGRS) et des
  outils. Un jeu est un **crate binaire** : un `main.rs` qui assemble le boilerplate de l'engine et
  les plugins choisis, un **dossier de contenu** (RON, LDtk, sprites, audio) chargé par l'engine, et,
  s'il le faut, un petit plugin Rust pour ce qui n'existe nulle part ailleurs. RON au maximum, Rust
  sinon.
- **Vue de dessus seulement.** Mouvement, caméra et navigation sont les modules de la vue de dessus.
  Une autre vue (plateformes, isométrie) serait un autre jeu de modules derrière les mêmes
  vocabulaires : rien n'est fait pour ça maintenant, et rien dans le plan ne l'empêche plus tard.
- **Critère de réussite.** L'engine sait faire un clone minimal de chacune des six références *avec
  du contenu seulement*. Si c'est vrai, 1837 (qui emprunte à chacune) est du contenu, et son plugin
  Rust reste petit.
- **Clone minimal** = la boucle reconnaissable du jeu, pas le jeu : un étage ou une carte, 8 à 12
  ennemis, 15 à 20 objets, un boss, la coop. Chaque clone vit dans `games/<nom>/` du dépôt de
  l'engine, avec ses scénarios et ses vidéos. Ce sont les tests d'intégration vivants de l'engine :
  un vocabulaire qui casse un clone casse un test.
- **Ce qui ne bouge pas.** La simulation reste en fixed-point, rollback, une frame GGRS par update en
  headless ; la présentation se dérive de l'état. Tout nouveau vocabulaire respecte la checklist GGRS
  de `CLAUDE.md`.

## 2. Ce que les six références demandent à l'engine

Légende : ✅ existe · 🟡 partiel · ❌ absent. Colonnes : **G** Gungeon, **I** Isaac, **NT** Nuclear
Throne, **RoR** Risk of Rain (analysé pour ses mécaniques, pas cloné ; ses plateformes sont hors
plan), **H** Hades, **Z** CoD Zombies, **1837**.

### 2.1 Combat

| Système | G | I | NT | RoR | H | Z | 1837 | État |
|---|---|---|---|---|---|---|---|---|
| Tir joueur : modes, chargeurs, rechargement, dispersion | ● | | ● | ● | | ● | ● | ✅ `weapons` |
| Corps à corps : patterns, hitbox, recul | ● | | ● | | ● | ● | ● | ✅ `weapons/melee` |
| Parade et renvoi de projectiles | | | | | ● | | ● | ❌ |
| Esquive avec invulnérabilité, variantes par classe | ● | | | | ● | | ● | 🟡 dash à i-frames, course nerveuse (movement-feel, `docs/conventions.md` §30) ; charges et variantes en données à faire (B6) |
| Projectiles composables (rebond, perce, guidé, division, orbite, taille, durée) | ● | ● | ● | | ● | | ● | 🟡 trois types fixes |
| Patterns de tir ennemis (anneau, spirale, visé, salve, séquence) | ● | ● | ● | ● | ● | | ● | ❌ les ennemis n'ont que la mêlée |
| Équipes, tir ami configurable, dégâts typés, résistances, immunités | ● | ● | ● | ● | ● | ● | ● | ❌ couches de collision seulement |
| Statuts (feu, gel, lent, étourdi, poison, charme, mouillé…) | ● | ● | | ● | ● | | ● | ❌ |
| Stats et modificateurs empilables (objets, classes, vagues, élites) | ● | ● | ● | ● | ● | ● | ● | ❌ |
| Santé riche : cœurs, armure, à terre, réanimation | | ● | | ● | ● | ● | ● | 🟡 santé et regen |
| Capacités : spécial, cast, ultime chargé, cooldowns, charges | | | | ● | ● | | ● | ❌ |
| Porter un objet lourd, canon fixe, monture | | | | | | | ● | ❌ |
| Collisions performantes (centaines de projectiles) | ● | ● | ● | | | | ● | ❌ boucles O(n²) (balles et déplacements) |

### 2.2 Objets et économie

| Système | G | I | NT | RoR | H | Z | 1837 | État |
|---|---|---|---|---|---|---|---|---|
| Effets déclenchés (passifs, actifs, consommables, « pièges ») | ● | ● | ● | ● | ● | ● | ● | ❌ |
| Familiers, orbitaux, tourelles posées, objets lancés, pièges, capture | ● | ● | | | | | ● | ❌ |
| Inventaire à emplacements, ramassage, lâcher, don | ● | ● | ● | ● | ● | ● | ● | 🟡 armes seulement, toutes données au départ |
| Pools de butin seedés, rareté, coffres, clés, déblocages | ● | ● | | ● | ● | ● | ● | ❌ |
| Monnaies multiples, boutiques, prix, dette, artisanat | ● | ● | | ● | ● | ● | ● | ❌ porte avec coût, sans argent |
| Synergies et transformations d'objets | ● | ● | | | ● | | ● | ❌ |
| Munitions typées, réserves partagées entre armes | | | ● | | | ● | ● | 🟡 chargeurs par arme |

### 2.3 Ennemis

| Système | G | I | NT | RoR | H | Z | 1837 | État |
|---|---|---|---|---|---|---|---|---|
| Navigation par flow field, profils, obstacles cassables | ● | ● | ● | ● | ● | ● | ● | ✅ |
| Behaviors composables, perception, choix de cible | ● | ● | ● | ● | ● | ● | ● | 🟡 une boucle poursuite/attaque, `EnemyAiConfig::zombie()` pour tous |
| Variantes (champions, élites, saison, nuit, réfractaire) | | ● | | ● | | | ● | ❌ |
| Résolutions alternatives (délivrer, nourrir, tromper, épargner) | | | | | | | ● | ❌ |
| Boss à phases, scripts d'arène, télégraphes | ● | ● | ● | ● | ● | | ● | ❌ |
| Alliés, neutres, escortes, PNJ, foule | | | | | ● | | ● | ❌ |
| Formations (salve en ligne, ronde) | | | | | | | ● | ❌ |

### 2.4 Monde

| Système | G | I | NT | RoR | H | Z | 1837 | État |
|---|---|---|---|---|---|---|---|---|
| Salles typées, état (vierge, en combat, vidée), verrouillage en combat | ● | ● | | | ● | | ● | 🟡 `RoomBounds` et spawners par salle, sans état |
| Grammaire d'étage (types obligatoires, contraintes, branches) | ● | ● | | | ● | | ● | 🟡 assemblage par connexions, sans types |
| Terrain destructible, cavernes procédurales | | | ● | | | | | ❌ |
| Surfaces (boue, glace, neige, eau) et dangers (piques, fosses, feu) | ● | ● | | ● | ● | | ● | ❌ |
| Feu qui se propage selon le matériau | | | | | | | ● | ❌ |
| Eau dynamique (marée, écluse), glaces et billots qui dérivent | | | | | | | ● | ❌ |
| Objets physiques (billots, barils, traîneaux), véhicules (barque, canot) | | | | | | | ● | ❌ |
| Décors destructibles avec butin (arbres, rochers, caisses, urnes) | ● | ● | ● | | ● | | ● | ❌ |
| Visibilité (brouillard, noir, lanternes, cônes) et bruit dans la simulation | | | | | | | ● | ❌ lumière = présentation seulement |
| Minicarte et carte d'étage | ● | ● | | | | | ● | ❌ |
| Activation par zone, plusieurs étages par partie | | | | | | ● | ● | ❌ (point 4 de `milestone.md`) |
| Élévation (toits, pentes) | | | | ● | | | ● | ❌ |

### 2.5 Structure de partie

| Système | G | I | NT | RoR | H | Z | 1837 | État |
|---|---|---|---|---|---|---|---|---|
| Mode vagues : rounds, scaling, spawns | | | | | | ● | ● | ✅ `waves` |
| Étages successifs, sortie ou portail, graine et état de run | ● | ● | ● | ● | ● | | ● | ❌ |
| Horloge de difficulté et horloges d'étage (nuit, marée, aube, minuit) | | | | ● | | | ● | ❌ |
| Choix de route (portes-récompenses, carte de campagne) | | | | | ● | | ● | ❌ |
| Modificateurs de run (heat, curses, couronnes, Carême), défis | | ● | ● | | ● | | ● | ❌ |
| Fin de partie, résumé, relance en moins de 10 s | ● | ● | ● | ● | ● | ● | ● | 🟡 écran game over |
| Équilibrage selon le nombre de joueurs | ● | | | ● | | ● | ● | ❌ |

### 2.6 Méta, hub et narration

| Système | G | I | NT | RoR | H | Z | 1837 | État |
|---|---|---|---|---|---|---|---|---|
| Profil persistant : déblocages, monnaies méta, codex, carte | ● | ● | ● | ● | ● | | ● | ❌ |
| Hub avec services et PNJ (Brèche, maison d'Hadès, loge) | ● | | | | ● | | ● | ❌ |
| Codex débloqué par événements | | ● | | | ● | | ● | ❌ |
| Répliques vocales déclenchées par événements (barks) | | | | | ● | | ● | ❌ |
| Musique adaptative, tempo lisible par la simulation | | | | | | | ● | 🟡 `harmonium` non porté, `audio/adaptive` |
| Tutoriel scénarisé (initiation dans le noir) | ● | | | | ● | | ● | ❌ |

### 2.7 Coop et réseau

| Système | G | I | NT | RoR | H | Z | 1837 | État |
|---|---|---|---|---|---|---|---|---|
| Rollback déterministe 1 à 4 joueurs, local et p2p | ● | ● | ● | ● | | ● | ● | ✅ GGRS + allumette |
| Caméra par joueur en ligne, salle verrouillée sur les présents | | | | ● | | | ● | 🟡 caméra multi-locale |
| Ressources partagées (jauges, monnaies), don, à terre, fantôme, réanimation | | | | ● | | ● | ● | ❌ |
| Rejoindre en cours, spectateur | | | | ● | | | ? | ❌ |
| Lobbies (publics, privés, amis) | ● | | ● | ● | | ● | ● | ✅ allumette (Steam hors plan) |

### 2.8 Présentation et outillage

| Système | État |
|---|---|
| HUD, menus, inventaire, carte, codex pilotés par données | ❌ UI minimales (lobby, game over, prompts) |
| Feedback : télégraphes, flash, secousse, hit stop, chiffres, particules | ❌ |
| Manette, remappage, localisation | 🟡 leafwing ; pas de localisation |
| Headless, scénarios, traces, vidéos, revue, contrôle remote, télémétrie | ✅ mais synctest et desync p2p sans checksum d'état (§9.1) |
| Manifeste de jeu, registre, lint du contenu, rechargement à chaud | ❌ chemins codés dans `global_asset.rs` |
| Benchmarks de performance en scénario | ❌ |
| Bots (inputs réactifs) pour simulations de masse | ❌ |
| Pipeline sprites (assetgen → config engine) | 🟡 `sprite.json` : 8 directions en lignes, pivot, fps par animation ; engine : une planche par calque, retournement gauche/droite |

**Lecture.** Le socle (déterminisme, réseau, navigation, armes, vagues, outillage) est solide, et
aucune référence ne l'offre gratuitement. Ce qui manque est le **vocabulaire de contenu** : stats,
effets, statuts, projectiles, behaviors, salles, run, méta. Les six jeux, comme 1837, sont surtout
faits de ça.

## 3. L'engine aujourd'hui : ce qui est codé en dur

À défaire avant de pouvoir « assembler par fichiers texte » :

- `crates/game/src/global_asset.rs` : chemins des sprites, animations, personnages et armes écrits en
  Rust ; un personnage « player » et trois zombies.
- `character/player/create.rs` : chaque joueur reçoit **toutes** les armes de `weapons.ron`, triées
  par nom, plus `bare_hands`.
- `character/enemy/create.rs` : tout ennemi reçoit `EnemyAiConfig::zombie()` et `zombie_claws` ;
  `EnemyAiConfigRon` est défini mais jamais chargé. `pathing.rs` garde des constantes (séparation,
  ralentissement) et `behavior.rs::enemy_movement_system` n'est pas branché.
- `collider/mod.rs` : six couches numérotées et une matrice fixe ; balles et déplacements testent tous
  les colliders (`weapons/mod.rs`, `player/input.rs`).
- `waves/` : un seul mode, activé par la ressource `WaveModeEnabled` ; `map_explorer` et le runner de
  scénarios l'activent en dur.
- `map_ldtk` : entités reconnues par identifiant LDtk (`DoorHorizontal`, `WindowVertical`,
  `ZombieSpawn`, `CrateLocation`, `WeaponLocation`, `SodaLocation`) ; les trois dernières ne font rien.
- Lumière (`bevy_light_2d`) et audio sont présentation seulement : rien dans la simulation ne « voit »
  ni n'« entend ».

## 4. Architecture cible : le jeu en texte, le vocabulaire en Rust

### 4.1 Un jeu = un crate et un dossier de contenu

```
games/<jeu>/
  Cargo.toml            crate binaire (membre du workspace pour les clones ; dépôt à part pour 1837)
  src/main.rs           le boilerplate : CoreSetupPlugin, les plugins de l'engine choisis, ceux du jeu
  src/*.rs              (si besoin) le plugin du jeu : ses kinds inédits
  assets/
    game.ron            manifeste : nom, modes, plugins Rust requis, dossiers à charger, joueurs 1..4
    characters/*.ron    joueurs et classes : stats, collider, skins, esquive, capacités, effets passifs
    enemies/*.ron       stats, navigation, perception, behaviors, attaques, variantes, résolutions, butin
    weapons/*.ron       modes de tir → projectile, munition, corps à corps, polarité, effets (« le piège »)
    projectiles/*.ron   vitesse, dégâts, modificateurs, on_hit, on_expire, visuel
    patterns/*.ron      émetteurs de tir : anneau, spirale, visé, salve, séquences
    items/*.ron         genre (passif, actif, consommable, familier, posé, lancé), effets, rareté, pool, synergies
    status/*.ron        durée, empilement, modificateurs, effets par tick
    rooms/*.ldtk +.ron  gabarits de salles et leurs métadonnées (type, biome, tags, spawns, entrées)
    floors/*.ron        grammaire d'étage : nombre de salles, contraintes, horloges, événements
    regions/*.ron       (campagne) étages + arène + âmes + règles de région
    modes/*.ron         vagues | étages | campagne ; conditions de fin ; scaling
    bosses/*.ron        phases, timelines, arènes
    loot/*.ron          pools et tables pondérées, conditions de déblocage
    economy/*.ron       monnaies (portée, validité), boutiques, recettes
    hub/*.ron           services et ce qui les ouvre
    barks/*.ron         répliques : déclencheur, conditions, priorité, cooldown, audio
    codex/*.ron         entrées et déclencheurs
    ui/*.ron            HUD et menus
    sprites/ audio/ fonts/
```

`main.rs` est court et presque le même d'un jeu à l'autre : ce sont le manifeste et le contenu qui
font le jeu. L'engine ne contient plus de contenu : `assets/ZombieShooter` devient `games/zombies/`,
le premier clone. Le dépôt `1837` (à créer) suit le même schéma et dépend de l'engine par git, comme
`allumette`.

### 4.2 Le manifeste et le registre

- `game.ron` liste ce qui compose le jeu ; l'engine charge tout dans un **registre** typé
  (`CharacterId`, `ItemId`, `RoomId`…) et refuse de démarrer si une référence est cassée, un id
  dupliqué, une valeur hors plage ou un « kind » inconnu.
- `alacod lint games/1837` fait la même validation sans lancer le jeu (l'équivalent de
  `lint_lore.py`), en CI et en hook.
- Rechargement à chaud en dev : modifier un RON recharge la définition, avec redémarrage de la partie
  si elle est en cours (les snapshots rollback ne se réécrivent pas à chaud).
- Le vocabulaire est **enregistré par les plugins Rust** : chaque plugin déclare ses kinds
  (behaviors, effets, conditions, modificateurs de projectile…). Un plugin de jeu ajoute les siens ;
  le lint connaît la liste.

### 4.3 Les vocabulaires

Chaque vocabulaire est un enum Rust sérialisé en RON, avec un système qui l'exécute dans
`GgrsSchedule`, dans un `RollbackSystemSet` fixe. Exemples courts, pour fixer le style.

**Stats et modificateurs** (la base de tout : objets, classes, statuts, élites, vagues, difficulté)

```ron
stats: { move_speed: "150", damage: "1.0", fire_rate: "1.0", reload: "1.0",
         dodge_distance: "60", dodge_iframes: 12, dodge_charges: 1, max_hp: "100", luck: "0" }
// Un modificateur : (stat: MoveSpeed, op: Mult, value: "0.8", source: "engelure.2", until: Status)
// Résolution : (base + Σ flat) × (1 + Σ pct) × Π mult, en fixed-point, dans un ordre fixe.
```

**Effets** (objets, classes, statuts, armes, salles, ennemis ; la moitié d'Isaac et de Gungeon tient
là-dedans)

```ron
effects: [
  (on: OnKill, if: [TargetTag("damne")], do: [Heal("5"), Bark("loup.delivre")]),
  (on: OnRoomEnter, do: [ApplyStatus(to: Enemies, status: "fige", frames: 120)], cooldown: Rooms(3)),
  (on: Tick(60), if: [Carrying("main-de-gloire")], do: [Block(Heal)]),
]
```

Déclencheurs (`OnHit`, `OnKill`, `OnDamageTaken`, `OnDodge`, `OnReload`, `OnRoomClear`, `OnPickup`,
`OnUse`, `OnGauge(id, Above(x))`, `Tick(n)`, `OnEvent(name)`), conditions et actions : trois listes
fermées, extensibles par plugin. Un effet a des charges, un cooldown (frames, salles, dégâts infligés)
et des stacks.

**Projectiles et patterns** (le bullet hell de Gungeon et d'Isaac, les salves de 1837)

```ron
"balle-benite": (speed: "400", damage: "12", life_frames: 120, damage_type: Benit,
  modifiers: [Pierce(All(tag: "damne"))])
"salve-de-ligne": Sequence([Telegraph(60), Aimed(count: 4, spread: "0.1", projectile: "plomb"), Wait(180)])
"ronde": Ring(count: 12, speed: "150", projectile: "braise", every: 90)
```

Les patterns se calculent à partir de la graine de run et de l'id de l'émetteur, jamais transmis
(contrainte de la bible §11). Les modificateurs se cumulent (rebond, perce, guidé, division, orbite,
boomerang, taille, gravité…) : c'est l'empilement des larmes d'Isaac.

**Statuts** : `(id: "mouille", frames: 600, stacking: Refresh, modifiers: [(ColdRate, Mult, "2.0")],
on_expire: [], visual: "goutte")`.

**Behaviors** (ennemis et alliés ; remplace la boucle unique de `behavior.rs` et `pathing.rs`)

```ron
"fantassin-de-ligne": (
  perception: (sight: "400", hearing: "300", needs_light: false),
  targeting: Nearest(ignore: [Disguised, Ghost]),
  behaviors: [
    (if: [HpBelow("0.2"), HasTag("refractaire")], do: Flee),
    (if: [SquadSize(3), TargetInRange("250")], do: Formation(Line, then: Shoot("salve-de-ligne"))),
    (if: [TargetInRange("40")], do: Melee("baionnette", cooldown: 60)),
    (do: Chase(profile: Ground)),
  ],
  resolutions: [(when: NotHitFor(180), if: [HasTag("refractaire")], become: Leave, reward: Hint)],
  variants: { "hiver": (modifiers: [(MoveSpeed, Mult, "0.8")], ignores_surface: ["neige-profonde"], skin: "capote") },
)
```

Sélection par priorité (première règle vraie), état de chaque behavior dans un composant rollback.
Un boss = les mêmes behaviors + `phases: [(until: HpBelow("0.6"), behaviors: [...], on_enter: [...])]`
+ des timelines (`At(frames, [...])`).

**Salles et étages**

```ron
// rooms/coupe-03.ron
(ldtk: "chantier.ldtk", level: "Coupe_03", kind: Combat, biome: "chantier", tags: ["froid", "arbres"],
 entrances: [N, S], spawns: [(table: "coupe-jour", count: "4 + players")])
// floors/chantier-1.ron
(rooms: 5..7, required: [(kind: Safe, count: 1)],
 constraints: [MaxConsecutive(tag: "froid", 3), Reachable(kind: Safe, within: 3)],
 clocks: [(id: "nuit", at: Seconds(150), then: [NightWaves(table: "nuit-chantier")])])
```

**Modes et état de run** : `Waves(...)`, `Floors(sequence: [...])`, `Campaign(map: "bas-canada",
steps: 3, finale: "quebec")`. L'état de run (graine, étape, route, monnaies, jauges, horloges,
drapeaux) est une ressource rollback ; la fin de partie et le résumé en dérivent.

**Barks, codex, HUD** : présentation dérivée des événements de simulation (`FrameEvents` relus hors
rollback, dédupliqués par frame), jamais l'inverse.

### 4.4 Expressions numériques

Les quantités acceptent une petite expression parsée au chargement et évaluée en fixed-point :
`"4 + players"`, `"max_hp * 0.3"`, `"gauge.sacre >= 0.75"`. Pas de flottants à l'exécution, pas
d'état caché. Les conditions restent des enums Rust ; seules les quantités sont des expressions.

### 4.5 RON au maximum, Rust sinon : pas de langage de script dans la simulation (décidé)

Un interpréteur (Lua, Rhai) dans `GgrsSchedule` casserait ce qui fait la valeur de l'engine :
flottants, ordre d'itération des tables, état hors snapshot, coût en wasm. Les listes de behaviors,
d'effets et les timelines en RON couvrent l'échelle de Gungeon, d'Isaac et d'Hadès. Ce qui est
vraiment inédit (la danse du Diable au tempo, la jauge de sacre partagée) va dans le plugin Rust du
jeu, avec la checklist GGRS. À revoir seulement si la vitesse de production de contenu l'exige, et
alors avec un langage entier-seulement et sans état propre.

### 4.6 Déterminisme et performance : règles pour les nouveaux vocabulaires

- Tout état de vocabulaire vit dans des composants ou ressources rollback ; les événements passent par
  `FrameEvents`, émis en ordre de `GgrsNetId`.
- Un flux de RNG **par usage** (patterns, butin, variantes), dérivé de la graine de run : ajouter un
  système ne change pas les tirages des autres (aujourd'hui, un seul `RollbackRng`).
- **Grille spatiale** (cellules de 32 px) pour balles, mêlée, auras et perception : premier chantier
  de performance.
- Mesurer le coût des snapshots avec 500 balles et 200 ennemis ; si trop cher, stocker les balles en
  tableau compact dans une ressource plutôt qu'en entités.
- Scénarios `bench_*` : frames simulées par seconde en headless, affichées dans la page de revue à
  chaque commit. Cibles à fixer au premier jalon (proposition : 4 joueurs, 200 ennemis, 500 balles à
  plus de 600 frames/s headless sur le poste de dev ; 60 fps en wasm avec 100 balles).

### 4.7 Comment 1837 devient du contenu

| 1837 (design) | Primitive de l'engine | Chantier |
|---|---|---|
| Jauges de sacre, de faim, d'excommunication, engelure, fou rire, dette | `Gauge` : valeur bornée, plancher, seuils qui émettent des événements, partage d'équipe optionnel | B2, F1 |
| Bénit, maudit, neutre | tags d'objet + effet `OnEquip: GaugeFloor(sacre, +x)` ; dégâts typés Benit et Maudit | B1, C1 |
| Pacte de la chasse-galerie, statut « Damné » | statut permanent + modificateurs + `OnGauge(sacre, Full) → Lose` | B3, C1, F1 |
| Délivrer le loup-garou, réchauffer le jenu, épargner le réfractaire | `resolutions` d'ennemi : condition → devenir allié, partir, se transformer ; récompense ; codex | D3 |
| Renvoyer une balle à la hache | parade au corps à corps : fenêtre parfaite, le projectile change d'équipe | B5 |
| Salve en ligne des fantassins, ronde des sorciers | formations + patterns `Sequence([Telegraph, Aimed])` | D1, B5 |
| Feu follet : traversé par les balles, imite une lumière, trois contres | immunité par tag, behavior `Lure`, résolutions (croix d'objets, aiguille, énigme) | B1, D1, D3 |
| Horloges de nuit, de marée, d'aube, de minuit | horloges d'étage avec événements planifiés | F2 |
| Camp, poêle, feu de camp | salle `Safe`, aura de chaleur qui baisse une jauge, objet posé `sac-a-feu` | E1, C3 |
| Boue, glace, neige profonde, batture | surfaces par cellule → modificateurs de stats et d'esquive | E4 |
| Le feu gagne les maisons de bois, la pierre l'arrête | automate de feu + matériau par cellule ; extinction selon le type d'eau | E5 |
| Fleuve béni, gué, marée qui ouvre la batture | règles de traversée par tag d'équipe ; niveau d'eau dynamique | E6 |
| Reviré | statut : miroir de rendu, inversion des axes d'input, minicarte menteuse | B3, I1 |
| Chapelet, poule noire, bâton tape | familiers (orbital, compagnon, projectile autonome) | C3 |
| Poêle à deux ponts, banc, tabatière, cruche | tourelle posée, piège au sol, nuage et flaque lancés | C3 |
| Sac du cordonnier | action `Capture` → `SpawnAlly` à la relâche | C3, D5 |
| Canon de bois, canon pris, cage de la Corriveau | objet lourd porté (modificateurs), canon fixe servi | B8 |
| Balles de cuillères, forge, cercler un canon | recettes d'artisanat avec coût sur d'autres objets | C5 |
| Sous, billets de paroisse, piastres, mots de passe, dette de la Compagnie | monnaies à portée et validité ; dette = jauge + intérêt par étage | C5 |
| Loge, âmes, services, grades | hub par drapeaux de profil ; déblocages qui ouvrent services, classes et pools | G1 à G4 |
| Carte du Bas-Canada, régions libérées, route par partie | graphe de campagne persistant, mode `Campaign` | F3 |
| Jos Violon, répliques en joual, veillée | barks événementiels ; codex à deux voix | H1, G3 |
| Le Diable beau danseur, violon ensorcelé | horloge de tempo, fenêtres « sur le temps », phases de boss | H2, D4 |
| Le mort revient en feu follet | état fantôme : entité contrôlée aux capacités réduites, réanimation | B6 |
| Civils masqués, foule de la place d'Armes | équipe neutre qui bloque les tirs ; `OnHit(Neutral) → Gauge(faim, +)` | B1, D5 |
| La grave qui résonne, la Hère à l'oreille, surdité | événement `Noise`, perception par l'ouïe, statut sourd qui coupe les sons | E8, B3 |
| Lanternes des patrouilles, initiation dans le noir | visibilité et cônes de lumière dans la simulation | E8 |
| Barque, canot, billot, glaces qui dérivent, traîneau | véhicules et plateformes mobiles | E6, E7 |
| Classes : esquives et ultimes différents | stats par classe, variantes d'esquive, emplacements de capacités | B6 |
| Variantes d'ennemis (hiver, nuit, gelé raide, réfractaire) | variantes = modificateurs + tags + skin | D2 |
| Testament de Lorimier, sursis, Carême | effets de run et de profil ; modificateurs de run | F4, G1 |
| Plugin Rust `1837` | la jauge partagée en coop et le « oui » du Diable, si le vocabulaire ne suffit pas | — |

## 5. Les chantiers

Chaque chantier : ce qu'il couvre, ce qui existe, ce qu'il faut faire, ce qu'il sert. Tailles
(en solo, avec les agents) : **S** quelques jours, **M** une à deux semaines, **L** trois à cinq
semaines, **XL** plus.

### A. Fondations du contenu

- **A1. Manifeste, registre, lint** (L). Remplacer `global_asset.rs` par le chargement de `game.ron`
  et de ses dossiers ; ids typés ; validation au chargement ; `alacod lint` ; rechargement à chaud.
  Sert à tout.
- **A2. Extraction de `games/zombies/`** (S). Sortir le contenu actuel de `assets/` ; `map_explorer`
  et le runner de scénarios prennent un jeu en paramètre.
- **A3. Expressions numériques** (S). Parseur et évaluateur fixed-point.
- **A4. RNG par flux** (S). `RollbackRng` dérivé par usage.
- **A5. Pipeline sprites** (M). Convertir le `sprite.json` d'assetgen (8 directions en lignes, pivot,
  fps par animation) en config engine ; étendre `animation` aux 8 directions et à l'ancrage au pivot.
  Sert à 1837.

### B. Combat

- **B1. Équipes et dégâts** (M). `Team`, politique de tir ami par arme et par polarité, `DamageEvent`
  typé (source, genre, montant), résistances et immunités par tag, invulnérabilité, armure. Remplace
  la matrice de couches par des règles lisibles. Sert à tous.
- **B2. Stats et modificateurs** (M). Le composant `Stats`, les modificateurs sourcés et datés, la
  résolution ; brancher mouvement, armes, esquive et santé dessus ; retirer les constantes de
  `pathing.rs`. Sert à tous.
- **B3. Statuts** (M). Composant générique, tick, empilement, expiration, visuel dérivé. Feu, gel,
  lent, étourdi, mouillé, charme, danse, aveugle, sourd.
- **B4. Grille spatiale** (M). Pour balles, mêlée, auras, perception ; scénarios de bench.
- **B5. Projectiles composables et patterns** (L). Modificateurs, `on_hit` et `on_expire`, émetteurs
  et séquences seedés, projectiles ennemis, télégraphes ; renvoi et parade au corps à corps (fenêtre
  parfaite), blanks. Sert à G, I, NT, H, 1837.
- **B6. Esquive et capacités** (M). I-frames, charges, variantes d'esquive en données (distance,
  durée, effet spécial : bousculade, téléport, leurre, sur place) ; emplacements de capacités
  (spécial, cast, ultime) avec cooldown ou charge par dégâts ; état « à terre » et fantôme,
  réanimation. Sert à G, H, RoR, 1837.
- **B7. Munitions typées et inventaire d'armes** (S). Réserves par type de munition, deux
  emplacements, lâcher, ramasser, donner, coup de crosse pendant le rechargement.
- **B8. Objets lourds, canons fixes, montures** (M). Sert à 1837 (canon, cage, poêle, cavalier).

### C. Objets et économie

- **C1. Effets** (L). Déclencheurs, conditions, actions ; cooldowns et charges ; ordre d'exécution
  fixe. C'est le chantier qui fait Isaac.
- **C2. Objets et inventaire** (M). Genres d'objets, emplacements, pickups au sol, actifs,
  consommables, polarité (tags), synergies (règle de paire → objet transformé).
- **C3. Familiers, posés, lancés** (M). Orbitaux, compagnons qui tirent, tourelles, pièges au sol,
  nuages, flaques, capture et relâche en allié.
- **C4. Butin** (M). Pools pondérés et seedés, rareté, coffres et clés, tables par salle et par
  ennemi, déblocages qui enrichissent les pools.
- **C5. Économie** (M). Monnaies à portée (partout, par paroisse, méta), boutiques, prix selon la
  réputation, recettes d'artisanat avec coût (les cuillères), dette avec intérêt et rappel.

### D. Ennemis, alliés, boss

- **D1. Behaviors composables** (L). Sélection par priorité, état par behavior, perception (vue,
  ouïe, lumière), ciblage, formations, patrouilles, fuite, pillage, escorte. Remplace `behavior.rs`
  et `pathing.rs` ; les zombies actuels deviennent un fichier. Sert à tous.
- **D2. Variantes et élites** (S). Modificateurs + tags + skin, tirage seedé (saison, nuit, champion).
- **D3. Résolutions alternatives** (M). Conditions (parade parfaite, nourrir, ne pas frapper N
  secondes, objet, offrande) → devenir allié, fuir, se transformer, récompense, événement codex. Sert
  à 1837 ; Hadès en use un peu.
- **D4. Boss** (M). Phases par seuil ou timer, timelines, arène scriptée (spawns, terrain, horloge),
  offre au joueur (accepter ou refuser sans texte), régénérations.
- **D5. Équipes non joueuses** (M). Alliés qui suivent et combattent, neutres (foule, PNJ) avec
  conséquences, prisonniers à libérer (interaction à N coups), escortes, marchands.

### E. Monde

- **E1. Salles typées et état** (M). Métadonnées de gabarit, `RoomState`, verrouillage des portes
  pendant le combat sur les joueurs présents, vagues internes, récompense à la sortie, activation
  des entités par salle (le point 4 de `milestone.md`).
- **E2. Grammaire d'étage** (L). Générateur par contraintes sur les gabarits LDtk (types
  obligatoires, distances, branches, secrets), minicarte, transition d'étage, plusieurs étages par
  partie ; Isaac en grille, Gungeon en graphe, 1837 en étages courts avec camp.
- **E3. Terrain destructible et cavernes** (M). Cellules destructibles par explosion, génération
  type Nuclear Throne, flow field incrémental. Sert à NT ; utile à 1837 (murs minces, arbres).
- **E4. Surfaces et dangers** (M). Tags de cellules (boue, glace, neige, eau peu profonde, huile),
  effets sur mouvement et esquive ; piques, fosses, barils, feu.
- **E5. Feu et matériaux** (M). Automate de propagation, bois et pierre, extinction eau ou eau bénite
  (règle par tag).
- **E6. Eau dynamique et plateformes** (L). Niveau d'eau par zone (marée, écluse), cellules qui
  basculent et flow field à jour, glace mince, glaces et billots qui dérivent, zones sûres par
  équipe ou tag (le fleuve béni).
- **E7. Objets physiques et véhicules** (L). Poussées, roulements en pente, barque et canot avec vie
  propre et chavirement, traîneau. Sert à 1837 seulement ; tard.
- **E8. Visibilité et bruit dans la simulation** (M). Rayon de vue par joueur, brouillard, noir et
  lanterne (initiation), cônes des lanternes ennemies, événement `Noise(radius)` qui alerte les
  salles voisines, surdité.
- **E9. Élévation** (M). Couches (toits, pentes) avec rampes ; tir plongeant. Tard.

### F. Structure de partie

- **F1. Modes et état de run** (M). Ressource `Run` rollback ; `Waves`, `Floors`, `Campaign` ;
  conditions de victoire et de défaite ; résumé ; relance rapide.
- **F2. Horloges** (S). Horloges d'étage et de run (nuit, marée, aube, minuit, difficulté),
  événements planifiés, lisibles par l'UI.
- **F3. Choix de route et carte de campagne** (M). Graphe de régions, brouillard, routes, libération
  persistante, passages latéraux, portes-récompenses à la Hadès.
- **F4. Modificateurs de run et défis** (S). Heat, curses, Carême, mots de passe.
- **F5. Équilibrage par joueurs** (S). `players` disponible dans les expressions partout où ça compte.

### G. Méta et hub

- **G1. Profil et sauvegarde** (M). Fichier versionné (déblocages, monnaies méta, codex, carte,
  grades), un profil par joueur en coop, écriture hors simulation à partir des événements de fin de
  partie.
- **G2. Hub** (M). Une carte de hub (LDtk) avec services activés par drapeaux, PNJ, entrée en partie,
  arrivée des âmes.
- **G3. Codex** (S). Entrées, déclencheurs, UI.
- **G4. Grades et déblocages** (S).

### H. Narration sans texte et son

- **H1. Barks** (M). File de répliques par locuteur, priorité, cooldown, conditions, audio ;
  sous-titres optionnels.
- **H2. Tempo** (M). Horloge de temps musical dans la simulation (frames par temps), fenêtres « sur
  le temps » pour tirer, frapper, esquiver ; musique synchronisée en présentation ; le port
  d'`harmonium` s'y branche.
- **H3. Audio de gameplay** (S). Sons dérivés des événements, spatialisés, coupés par la surdité.

### I. Présentation et UI

- **I1. HUD et menus en données** (L). Barres (vie, jauges, chaleur), munitions, objets, minicarte,
  prompts sans texte (icônes), inventaire, carte, codex, hub, options, remappage, localisation fr et
  en. Le plus gros chantier de présentation ; choisir le toolkit (§7).
- **I2. Feedback** (M). Télégraphes de zone, flash, secousse, hit stop, chiffres, particules, dérivés
  des événements.
- **I3. Caméra par joueur en ligne** (S). Chaque client suit son joueur ; salle verrouillée sur les
  présents.

### J. Réseau

- **J1. Coop 4** (S). Valider 4 joueurs p2p en scénarios (aujourd'hui testé à 2).
- **J2. Partage et don** (S). Jauges partagées, monnaies d'équipe, don d'objets, tir ami par polarité.
- **J3. Lobbies** (S). Allumette reste la seule couche de lobbies (web, natif) ; Steam est hors plan.
  Ce qu'il reste : boucler lobby → partie → résumé → lobby sans relancer le binaire, et vérifier le
  lobby à 4.
- **J4. Rejoindre en cours, spectateur** (L). Seulement si la question 10 de la bible se décide oui :
  GGRS ne le fait pas nativement, il faut une synchro d'état au prochain camp.

### K. Outillage

- **K0. Déterminisme vérifiable** (S). Checksums GGRS sur tout l'état rollback, une extension unique
  `rollback_and_trace`, `SyncTestMismatch` observé par le runner, trace d'état générique (§9.6).
  Avant tout le reste : sans ça, le synctest et la détection de desync ne voient rien.
- **K1. Scénarios étendus** (M). Attentes sur salles, objets, jauges, horloges, run ; scénarios par
  jeu (`games/<jeu>/scenarios`).
- **K2. Bots** (M). Source d'inputs réactive (aller vers, tirer sur, ramasser) pour des parties à
  1..4 sans script fixe ; simulations de masse headless pour l'équilibrage (taux de mort par salle,
  durée d'étage). C'est la suite du flow agentique.
- **K3. Benchmarks** (S). Scénarios `bench_*`, métriques dans la page de revue.
- **K4. Doc des conventions** (S). LDtk (couches, entités, champs), dossier de jeu, checklist d'un
  nouveau vocabulaire.
- **K5. Éditeur de tuning** (S). Egui : modifier une définition en jeu et la sauver en RON.
- **K6. CI** (M). Réparer le jeton, puis deux étages (rapide à chaque commit, lent chaque nuit),
  artefacts par commit et page de revue publiée ; notes de revue dans un fichier du dépôt (§9.9).

## 6. Découpage en jalons

Chaque jalon livre un clone minimal jouable de 1 à 4 en ligne (M3 se limite à 2, comme la bible le
demande), ses scénarios headless verts, ses vidéos dans la page de revue, et son contenu en RON
seulement. Les critères de sortie détaillés sont au §9.8.

| Jalon | Clone | Ce qu'il prouve | Chantiers | Taille |
|---|---|---|---|---|
| **M0** | `zombies` complet (CoD Zombies) | Le jeu est un crate et un dossier ; points, achats, perks, à terre et réanimation, power-ups ; 4 joueurs en ligne, caméra par joueur ; le déterminisme se vérifie tout seul et la CI tourne | K0, K6, A1 à A4, B1, B2, B4, B6 (à terre), B7, C5 (v1), F1 (`Waves`), F5, I3, J1, K1, K3, K4 | L |
| **M1** | `throne` (Nuclear Throne) | Cavernes destructibles, projectiles ennemis, deux armes et cinq munitions, niveaux et mutations, portail, run seedée, horloge de difficulté | B3, B5 (v1), C1 (v1), C4 (v1), D1 (v1), D2, E3, E4 (v1), F1 (`Floors`), F2 | L |
| **M2** | `gungeon` (Enter the Gungeon) | Étages de salles typées et verrouillées, bullet hell, roulade à i-frames, blanks, coffres, clés, boutique, objets passifs et actifs, synergies, boss à phases et arènes à tenir, hub et déblocages | B5 (v2), B6, C1 (v2), C2, C4, C5, D4, E1, E2, G1, G2, I1 (v1), I2, K2 (v1) | XL |
| **M3** | **1837 : « La coupe »** (tranche verticale, bible §13) | Engelure et jauge de sacre, camp et poêle, 5 à 8 objets, mistigris (horde, variantes), feu follet (immunité, contres), loup-garou (parade → délivrance), 1 et 2 joueurs en ligne, sprites assetgen | A5, B5 (parade), D3, D5 (v1), E4 (neige, chaleur), E8 (v1), F2 (nuit), H1 (v1), J2, plugin `1837` | L |
| **M4** | `isaac` (Binding of Isaac) | Effets à l'échelle (100 objets en RON), empilement de modificateurs de projectiles et par nombre d'exemplaires, familiers et orbitaux, rochers et bombes, cœurs, clés, sous, marchés du diable, curses, champions et élites | B5 (v3), C1 (v3), C3, D2 (v2), E4 (v2), F4, G1 (v2) | L |
| **M5** | `hades` (Hades) | Capacités (attaque, spécial, cast, dash, call), bienfaits à rareté, niveaux, prérequis et duos, portes-récompenses, hub avec PNJ et barks, miroir, pacte de punition, souvenirs, armure, télégraphes, codex | B6 (v2), C1 (v4), D4 (v2), F3 (v1), F4, G2 (v2), G3, G4, H1 (v2), I2 (v2) | L |
| **M6** | **1837 : une région, puis la campagne** | Les chantiers en entier (3 étages et la digue), carte de campagne, loge, âmes, codex, classes, Reviré, feu, marée, barques, Diable au tempo, bots à 4 pour l'équilibrage | A5, B8, D5, E5 à E7, E9, F3, G1 à G4, H2, I1 (v2), J3, K2 (v2), plugin `1837` | XL |

Notes.

- **M3 avant Isaac et Hadès, exprès.** La bible dit « savoir si la boucle est amusante avant de
  produire le reste ». Après M2, tout ce que la tranche verticale demande existe, sauf la délivrance,
  la chaleur et les barks, qui sont petits. Isaac et Hadès enrichissent ensuite le vocabulaire que M6
  utilisera.
- **Risk of Rain : analysé, pas cloné.** Ses mécaniques atterrissent ailleurs : l'horloge de
  difficulté en M1 (F2), l'empilement par nombre d'exemplaires et les élites en M4 (C1, D2),
  l'événement « tenir la zone » en M2 (D4), les quatre joueurs en ligne avec caméra par joueur dès
  M0 (J1, I3, F5). Ses plateformes sont hors plan.
- **Chaque clone reste petit.** Le but est le vocabulaire, pas la fidélité : dix ennemis et vingt
  objets bien choisis exercent plus de kinds qu'un catalogue.
- **La performance se mesure dès M0** (bench de vagues à 4 joueurs) et se vérifie à chaque jalon
  (M2 : 500 balles ; M6 : une région entière).

## 7. Décisions

### Prises (2026-09-28)

| # | Question | Décision |
|---|---|---|
| 1 | Risk of Rain | Pas de clone, pas de module plateformes : ses mécaniques sont analysées et placées dans les autres jalons (§6). Vue de dessus seulement ; d'autres vues restent possibles plus tard, rien maintenant. |
| 2 | Script ou RON | RON composé au maximum, plugins Rust sinon. Un jeu est par défaut un `main.rs` avec le boilerplate de l'engine et son dossier de contenu (§4.1). |
| 3 | Steam | Hors plan. Allumette reste la couche de lobbies. |

**Prise ensuite (2026-10-03) — Consoles.** Cibles futures : Steam Deck (immédiat, le build
natif actuel suffit), puis Switch 1 et 2, PS4/PS5 en officiel ; la 3DS et la Vita restent des
terrains de jeu homebrew, jamais des canaux de release. La frontière sim/présentation de
`CLAUDE.md` est la frontière de portage : **rien à faire maintenant**, mais la simulation ne doit
jamais dépendre de `bevy_render`, `bevy_winit`, `bevy_asset`, `bevy_audio`, `bevy_ui` ni `winit`
(elle ne dépend de `bevy` qu'en ECS seul). Le jour venu, un port console remplace ce qui est
derrière `PresentationPlugin` : wgpu-Vulkan sur Switch 1/2, renderer custom (ou porteur) sur
PS4/PS5. Le prérequis réel n'est pas technique mais commercial : un jeu fini sur Steam pour la
candidature Nintendo, puis le kit (~440 $US).

### Ouvertes

| # | Question | Recommandation |
|---|---|---|
| 4 | Toolkit d'UI | Voir §7.1 : `bevy_ui` comme base, HUD maison, test comparatif de deux jours au début de M2 pour les écrans. |
| 5 | Où vit le contenu de 1837 | Dépôt `1837` (crate et contenu), engine en dépendance git comme allumette ; le lore reste dans `1867_lore`, un script y puise codex et barks. |
| 6 | Balles : entités ou stockage compact | Entités d'abord, bench à M0 et M2 ; compact seulement si les snapshots coûtent trop. |
| 7 | Web : cible ou vitrine | Vitrine (démos des clones) ; les cibles de perf sont natives, le web reste compilable. |
| 8 | Format de sauvegarde | RON versionné, un profil par joueur (pubkey allumette), écrit hors simulation. |
| 9 | Rejoindre en cours | Non pour la version 1 ; J4 seulement si la bible tranche oui (question ouverte 10). |

### 7.1 L'UI : de quoi choisir

Ce que le plan demande à l'UI : un HUD lisible sans texte (vie, jauges, munitions, objet actif,
minicarte, prompts en icônes), des écrans (lobby, pause, options et remappage, inventaire, carte,
codex, hub, résumé de partie), la manette partout, le web, et une mise en page modifiable sans
recompiler, dans l'esprit du reste.

L'état de l'écosystème, vérifié le 2026-09-28 :

- **`bevy_ui`** (intégré) : mise en page flex et grid ; `bevy_ui_widgets`, les widgets sans
  habillage (boutons, curseurs, cases), activés par défaut depuis 0.19 ; `EditableText` pour la
  saisie ; fonctionne en wasm. Pas de format de fichier : la mise en page se décrit en Rust ou par un
  chargeur maison.
- **BSN** (0.19) : la syntaxe de scènes de Bevy, utilisable aujourd'hui par la macro `bsn!` ; le
  chargeur de fichiers `.bsn` n'est pas encore livré, il est annoncé pour une version future. Quand il
  arrivera, ce sera le format texte officiel pour décrire scènes et UI, avec rechargement à chaud : la
  cible naturelle pour un engine piloté par fichiers.
- **`feathers`** : la collection de widgets de Bevy, pensée pour l'éditeur (look d'outil), portée sur
  BSN. Bonne pour les outils de debug et de tuning, pas pour l'habillage d'un jeu.
- **`bevy_hui`** : gabarits en pseudo-HTML par-dessus `bevy_ui`, rechargement à chaud, Bevy 0.15 à
  0.19 (v0.7 pour 0.19). Le seul toolkit « fichiers texte » à jour.
- **`bevy_cobweb_ui`** : le format COB et son hot reload, mais archivé et plus maintenu depuis
  janvier 2026, dernier Bevy supporté 0.17. À écarter.
- **`egui`** : déjà là pour le debug ; rapide à écrire, look d'outil. À garder pour le debug et le
  tuning (K5), pas pour le jeu.

Recommandation :

1. **Base `bevy_ui`** dans tous les cas : c'est ce qui vivra le plus longtemps, en wasm comme en
   natif, et c'est sur quoi BSN s'appuie.
2. **HUD** : une couche maison mince, `ui/hud.ron` → arbre `bevy_ui`, liée aux données de la
   simulation (jauges, munitions, objet actif). Le HUD est petit et stable ; l'écrire soi-même coûte
   moins qu'une dépendance, et il migrera vers `.bsn` le jour où le chargeur sort.
3. **Écrans** : test comparatif de deux jours au début de M2 (I1 v1). Le menu de pause avec navigation
   à la manette et l'inventaire de Gungeon, faits deux fois : avec `bevy_hui`, et avec la couche
   maison étendue. Critères : wasm, rechargement à chaud, manette, lignes par écran, et ce qu'il
   faudra jeter quand les fichiers `.bsn` arriveront.
4. **M0 n'a besoin que du HUD** (points, prompts d'achat, lobby existant) : la décision peut attendre
   M2 sans rien bloquer.

## 8. Risques

- **Chaque vocabulaire est une source de desync.** Mitigation : le déterminisme vérifiable (K0,
  §9.6), la checklist GGRS par kind, un scénario à 2 et à 4 joueurs par vocabulaire, les traces, un
  lint qui refuse les flottants dans les définitions.
- **Coût des snapshots** avec des milliers d'entités rollback (balles, cellules de feu). Mitigation :
  bench dès M0, stockage compact si besoin, cellules de terrain en ressource et non en entités.
- **Trop de clones, trop gros.** Mitigation : la définition de « minimal » au §1 ; un clone est fini
  quand ses scénarios passent, pas quand il est beau.
- **L'UI est toujours sous-estimée.** I1 est un chantier à part entière ; le décider au début de M2
  (§7.1), pas plus tard.
- **Dépendances forkées** (`bevy_ecs_ldtk`, `matchbox`, `harmonium` non porté) : chaque montée de
  Bevy coûte ; grouper les montées entre deux jalons.
- **Contenu autochtone** : rien dans l'engine ne bloque ; les éléments concernés restent hors des
  clones et de M3 tant que la validation des nations n'a pas eu lieu (bible §15.8).

## 9. Tests et validation : développer avec le moins de validation humaine possible

### 9.1 La base, et ses trous

Ce qui existe déjà, et sur quoi tout repose :

- **Headless** (`ALACOD_HEADLESS=1`, profil `headless`, sans rendu des tilemaps) : une frame GGRS par
  update, aussi vite que le CPU le permet.
- **Trace d'état** : un hash par frame d'un sous-ensemble fixe de composants (transform, santé,
  vitesse, état du monstre, cible, fenêtre, obstacle), de l'état des vagues et du RNG ;
  `ALACOD_STATE_TRACE_FULL=1` pour l'état détaillé.
- **Scénarios** (`tests/scenarios/*.ron`, douze aujourd'hui) : inputs scriptés par joueur, attentes
  à une frame, trace de référence, `weapon_overrides`, commentaire « À regarder », `BLESS=1` pour
  les changements voulus.
- **Enregistrement** d'une session jouée ou pilotée en scénario rejouable ; **contrôle remote**
  (`brief`, `state`, `input`, `step`, `screenshot`, `save`).
- **Vidéos** : capture déterministe hors écran (image n = frame n), montage, avant/après entre deux
  références, moments clés détectés (vague, kills, coups, morts, rechargements, armes, fenêtres,
  portes), page de revue avec lecture synchronisée et notes.
- **Multijoueur** : `test_multiplayer` lance deux clients par matchbox et compare leurs logs
  `ggrs{...}` (`diff_log`) ; télémétrie des crashs et des desyncs.
- **Diagnostics** ignorés par défaut : `nav_map`, `nav_stats`, `nav_probe`, `weapon_probe`,
  `map_probe`.

Trois trous, constatés dans le code :

1. **Le synctest ne voit rien.** Les scénarios tournent déjà en session GGRS synctest (rollback et
   resimulation à chaque frame, distance 2), mais aucun composant ni ressource n'a de checksum
   enregistré : le checksum GGRS ne couvre que le nombre d'entités. Une divergence d'état après
   rollback passe inaperçue, et la détection de desync p2p (`DesyncDetection::On`) compare ce même
   checksum vide. `SyncTestMismatch` n'est observé nulle part : au mieux un `warn`.
2. **La trace d'état est manuelle** : chaque nouveau vocabulaire devrait être ajouté à la liste de
   `state_trace.rs`, et rien ne le rappelle.
3. **La CI est cassée** (jeton ghcr expiré) et ne joue que `make test` : ni lint, ni bench, ni
   vidéos, ni p2p. Les notes de la page de revue vivent dans le `localStorage` du navigateur : un
   agent ne peut pas les lire.

### 9.2 Les principes

1. **Le déterminisme est le testeur.** Même graine et mêmes inputs donnent la même trace ; sinon
   c'est un bug, jamais un test « flaky ». Une divergence bloque.
2. **Un chantier n'est pas fini sans son scénario**, et le scénario s'écrit avant le code : ses
   attentes décrivent le comportement voulu, son « À regarder » dit quoi vérifier à l'œil.
3. **La simulation se teste sans rendu.** Le rendu ne sert qu'aux vidéos et aux captures.
4. **L'humain regarde des vidéos et des chiffres**, pas du code ni des logs. Sa validation se
   concentre aux sorties de jalon, aux décisions de design et au fun ; le reste est automatique.
5. **Ce qu'un humain a vérifié deux fois devient un test** : une note sur la page de revue se
   transforme en attente, en invariant ou en scénario.
6. **Les clones sont les tests d'intégration ; le contenu de test est à part** (§9.5), pour que les
   scénarios de vocabulaire ne cassent pas quand un clone change.

### 9.3 La pyramide

| Niveau | Quoi | Durée | Quand | Outil |
|---|---|---|---|---|
| **N0 Lint** | références, ids, plages, kinds inconnus, flottants interdits, un `test:` par définition | instantané | hook après chaque écriture, pre-commit, CI | `alacod lint` (A1) |
| **N1 Unitaires** | logique pure : expressions, résolution des stats, RNG par flux, grille spatiale contre la force brute, flow field, grammaire d'étage (1000 graines : contraintes et atteignabilité), tables de butin (distribution), automate de feu, horloges, sélection des barks, fenêtres de tempo, sauvegarde aller-retour | secondes | chaque commit | `cargo test` |
| **N2 Déterminisme** | chaque scénario en synctest avec checksums complets ; joué deux fois, même trace ; à 2 et à 4 joueurs | secondes | chaque commit | runner, après K0 |
| **N3 Scénarios** | inputs scriptés, attentes, trace de référence, invariants vérifiés à chaque frame | secondes par scénario | chaque commit | `make test_scenarios` |
| **N4 Scénarios générés** | pour chaque objet, arme, ennemi, statut, salle du jeu : un gabarit instancié dans le testbed (apparition, N frames, attentes déclarées dans la définition, sinon invariants seulement) | minutes | chaque commit du jeu touché | générateur (K1) |
| **N5 Bots et masse** | parties complètes à 1..4 bots, centaines de graines, métriques (durée, morts par salle, sources de dégâts, objets pris, softlocks, desyncs) et seuils statistiques | dizaines de minutes | chaque nuit, et avant une sortie de jalon | `alacod sim` (K2) |
| **N6 Bench** | frames/s headless, taille des snapshots, entités et balles max, sur `bench_*` | minutes | chaque commit sur `main`, seuils dans `budgets.ron` | K3 |
| **N7 P2P réel** | 2 puis 4 clients headless par allumette en local (docker compose), même scénario, `diff_log` identique ; déconnexion ; délai d'input | minutes | chaque nuit, avant jalon | `test_multiplayer` étendu |
| **N8 Captures et vidéos** | vidéos des scénarios et d'un échantillon de parties bots, avant/après, captures des écrans d'UI comparées à des références | minutes | chaque commit sur `main` | `make videos`, page de revue |
| **N9 Humain** | jouer le clone ; le fun (M3), la direction artistique, la lisibilité sans texte | heures | sorties de jalon | page de revue, notes |

### 9.4 Ce qu'on teste, chantier par chantier

Pour chaque groupe du §5 : les tests unitaires, les scénarios et leurs attentes nouvelles, ce qui se
génère ou se simule, et ce que la vidéo doit montrer.

| Chantier | Unitaires (N1) | Scénarios (N3) et attentes nouvelles | Générés, bots (N4, N5) | Vidéo (N8) |
|---|---|---|---|---|
| A. Registre, lint, expressions, RNG | chargeur (contenu valide, et chaque erreur possible avec son message), évaluateur (précédence, fixed-point, débordements), flux RNG indépendants | un scénario par clone qui charge tout son contenu | — | — |
| B1. Équipes et dégâts | matrice des règles de tir ami | `Health`, `NoDamageBetween` : l'arme ordinaire n'atteint pas l'allié, la maudite oui ; la balle traverse l'immunisé | générés : chaque arme contre chaque équipe | — |
| B2. Stats | ordre de résolution, empilement, sources datées | `PlayerPosition` après N frames avec un modificateur de vitesse ; `Stat(handle, nom, valeur)` | générés : chaque objet à modificateurs | — |
| B3. Statuts | durée, empilement, expiration | `HasStatus`, `StatusStacks` ; le tick fait ce qu'il dit | générés : chaque statut sur le mannequin | flash et icône visibles |
| B4. Grille spatiale | équivalence avec la force brute sur des jeux aléatoires | les scénarios existants gardent leur trace | `bench_bullets`, `bench_horde` | — |
| B5. Projectiles, patterns, parade | chaque modificateur, les séquences | `BulletCount`, `HitsAtLeast`, `BulletsInside` ; la parade change l'équipe du projectile ; un pattern est identique à graine égale | générés : chaque projectile et chaque pattern dans l'arène ; bench à 500 balles | le motif se lit ; la parade se voit |
| B6. Esquive, capacités, à terre | fenêtres d'i-frames, charges, recharge | `NoDamageBetween` pendant la roulade ; `PlayerDowned`, `PlayerRevived` ; `AbilityCharge` | bots : la réanimation à deux | la roulade et l'ultime |
| B7, B8. Munitions, objets lourds | réserves par type | `Ammo` par type ; ramasser, lâcher, donner ; vitesse réduite en portant | — | — |
| C1. Effets | dispatch par déclencheur, cooldowns, charges, ordre | un scénario par déclencheur avec un objet fixture (`OnKill` → `Health` monte de 5) ; `Event(nom)` | générés : chaque objet du jeu a son scénario ; sa définition déclare l'attente (`test: (after: KillOne, expect: Health(">= 55"))`) | — |
| C2 à C5. Objets, familiers, butin, économie | distribution des tables (10 000 tirages dans la tolérance), aller-retour de l'inventaire | `Inventory`, `Currency`, `EntityCount(tag)` ; achat, don, capture et relâche | bots : ce qui est ramassé et acheté sur 200 graines | l'orbital tourne, la tourelle tire |
| D1, D2. Behaviors, variantes | sélection par priorité, perception (vue, ouïe, lumière) | `EnemyState`, `EnemyDistance`, `EnemyTeam` ; les diagnostics de navigation deviennent des attentes (contact avant N frames, jamais dans un mur) ; le bruit alerte la salle voisine | générés : chaque ennemi et chaque variante, seul contre le joueur immobile puis mobile ; bots : temps de contact | la formation, la patrouille |
| D3. Résolutions | conditions | parade parfaite → `EnemyTeam(ally)`, `Event(resolution)` ; nourrir trois fois | générés : chaque résolution déclarée | la délivrance se comprend sans texte |
| D4. Boss | phases par seuil et timer | `BossPhase` à la frame attendue ; timeline ; offre acceptée ou refusée | bots : le boss est battable (taux de victoire par graine) | chaque phase |
| D5. Alliés, neutres | — | l'allié suit et combat ; le civil bloque et coûte ; le prisonnier se libère en N coups | bots : escorte réussie | — |
| E1, E2. Salles, étages | solveur de grammaire (1000 graines), hash de disposition par graine | `RoomState` : entrée → verrou → vidée → déverrou ; activation par salle ; `FloorIndex` après la sortie | bots : chaque étage se finit (softlock = échec) | la minicarte |
| E3 à E9. Monde | automate de feu, niveau d'eau et flow field, surfaces | `CellState` ; vitesse sur la boue ; le damné ne traverse pas l'eau ; la glace casse | bots : parties dans chaque biome | le feu se propage, la marée monte |
| F. Run, horloges, route, modificateurs | machine d'états de run, horloges | `RunStep`, `Clock(id, fired)`, `Gauge(id, valeur)` ; la nuit lève des vagues dans les salles vidées ; la partie se perd à jauge pleine | bots : durée d'une expédition, victoire par graine | — |
| G. Profil, hub, codex, grades | aller-retour de sauvegarde, versions | `ProfileHas(flag)` après une partie ; le service s'ouvre ; le pool s'enrichit | bots : progression sur vingt parties d'affilée | la loge se remplit |
| H. Barks, tempo, audio | priorité, cooldown, fenêtres de temps | `Event(bark)` dans les moments clés ; le coup sur le temps est critique | — | la vidéo garde la piste audio |
| I. HUD, menus, feedback | liaison des données | — (présentation) | captures de chaque écran comparées à des références, à trois résolutions ; l'agent regarde les images | télégraphes visibles avant le coup |
| J. Coop 4, partage | — | scénarios à 4 en synctest ; jauge partagée ; don | p2p réel à 4 (N7) | — |

**Invariants vérifiés à chaque frame** par le runner, sans rien écrire dans les scénarios : santé dans
`[0, max]` ; aucune entité rollback hors de la carte ; aucun joueur dans un mur ; `GgrsNetId` uniques ;
emplacements d'inventaire cohérents ; état de run cohérent avec le mode. Les interdits (`HashMap` de
la bibliothèque standard, `f32`) dans un composant rollback sont bloqués par les types de l'engine
quand c'est possible, et par un grep en CI sur `crates/game/src` sinon.

### 9.5 Le testbed

`games/testbed/` : un jeu qui ne sert qu'aux tests. Des salles minimales (arène vide, couloir, deux
salles et une porte, une fenêtre, de l'eau, du bois qui brûle), un mannequin immobile à santé
réglable, une cible qui compte les coups, un ennemi suiveur basique, un allié et un civil. Les
scénarios générés (N4) et les micro-scénarios de vocabulaire tournent dedans ; ils ne changent pas
quand un clone change. Les clones gardent leurs propres scénarios, plus gros et moins nombreux (une
partie type, un boss, une coop à 4).

### 9.6 Rendre le déterminisme vérifiable (K0, en premier)

- Une seule extension pour enregistrer un état rollback : `app.rollback_and_trace::<T>()` enregistre
  le rollback, le checksum GGRS (`checksum_component`, `checksum_resource`) et la trace d'état. Les
  appels directs à `rollback_component_*` disparaissent de l'engine ; un test compare la liste des
  types rollback à la liste des types tracés.
- Le runner observe `SyncTestMismatch` : un scénario échoue à la première frame divergente, avec le
  composant en cause (le runner rejoue en trace complète et affiche la première ligne qui diffère).
- Un mode « stress » du synctest (distance 8) sur les scénarios du testbed.
- La même trace nourrit la détection de desync p2p : un desync en partie réelle devient un rapport de
  télémétrie avec la frame et le composant, pas seulement deux checksums.

### 9.7 La boucle de travail d'un agent

Pour un chantier ou un lot de contenu :

1. Lire la définition (§5) et **écrire le scénario d'abord** : attentes, invariants, commentaire
   « À regarder ». Pour du contenu, déclarer le `test:` de chaque définition ; le générateur fait le
   reste.
2. Implémenter : RON d'abord, un kind Rust seulement s'il manque.
3. Vérifier en local, sans rendu : lint (automatique par hook), `cargo test`, `make test_scenarios`
   (synctest compris), bench si la simulation est touchée.
4. `BLESS=1` seulement pour un changement de gameplay voulu, scénario par scénario, avec la raison
   dans le commit. Un bless de plus de trois scénarios se justifie dans le message.
5. Produire la vidéo du scénario ; **l'agent regarde lui-même** les images clés (les captures sont
   des PNG) et vérifie le « À regarder » avant de dire que c'est fini.
6. Commit dans `main` ; la CI rejoue tout et publie vidéos, traces et métriques sur la page de revue.

Pour explorer sans script : `make remote HEADLESS=1`, puis `brief`, `input`, `step`, `screenshot` ;
`save` transforme la session en scénario dès qu'un comportement mérite d'être gardé.

Les **bots** (K2) sont une source d'inputs déterministe (profils : fonceur, prudent, explorateur,
immobile, aléatoire seedé ; décisions prises sur l'état et un flux RNG dédié). Un run de bot est
rejouable et s'enregistre en scénario : un bug trouvé par la masse devient un scénario fixe le jour
même. `alacod sim --game zombies --bots 4 --seeds 1..200` sort un JSON de métriques ; `budgets.ron`
fixe les seuils (aucun softlock, aucun desync, durée d'étage dans une fourchette, taux de victoire
entre deux bornes) ; la page de revue affiche l'évolution par commit.

### 9.8 Ce que l'humain fait encore, et quand

| Moment | Ce qu'il regarde | Ce qu'il en sort |
|---|---|---|
| Sortie de jalon | joue le clone trente minutes ; regarde le montage des scénarios et un échantillon de parties bots | notes sur la page de revue → attentes, invariants ou scénarios (principe 5) ; go ou pas |
| Tranche verticale de 1837 (M3) | les cinq critères de réussite de la bible §13, en jouant à deux | décisions de design |
| Décisions (§7) | les options écrites par l'agent | une réponse |
| Direction artistique, son, lisibilité sans texte | vidéos et captures | notes |
| Chaque semaine | un digest écrit par l'agent : ce qui a bougé, les métriques, les vidéos à voir (liens vers la page de revue) | dix minutes |

Tout le reste est automatique. Pour que la boucle note → test fonctionne, les notes de la page de
revue passent du `localStorage` à un fichier du dépôt (`tests/review-notes/<commit>.md`, écrit par
le serveur de revue) que l'agent lit et ferme note par note.

**Critères de sortie d'un jalon** : lint et tests verts sur `main` ; tous les scénarios du clone en
synctest à 2 et à 4 ; les bots finissent le clone sur 200 graines sans softlock ni desync ; bench
dans les budgets ; vidéos publiées ; doc des conventions à jour ; les notes du jalon précédent
fermées.

### 9.9 La CI (K6)

1. Réparer le jeton et garder le conteneur `alacod-builder`.
2. Étage rapide, à chaque commit et PR (runner ubuntu, moins de cinq minutes) : lint de tous les
   jeux, `cargo test`, scénarios en synctest, grep des interdits.
3. Étage lent, chaque nuit et sur `main` (self-hosted) : bench avec seuils, bots de masse, p2p réel
   à 2 et 4 par allumette (docker compose), vidéos et captures, publication de la page de revue
   (Tailscale, ou Cloudflare Pages comme le site).
4. Les vidéos sur un runner sans GPU : `play_scenario --capture` avec Mesa lavapipe (Vulkan
   logiciel), à vérifier ; sinon, le self-hosted les fait.
5. Artefacts conservés par commit : traces, métriques JSON, vidéos, captures, digest.

## 10. Par où commencer

1. Décisions prises : vue de dessus, RON et plugins Rust, pas de Steam, Risk of Rain analysé sans
   clone. Restent l'UI (§7.1, à trancher au début de M2) et le dépôt de 1837 (§7, question 5).
2. M0, dans cet ordre : K0 (checksums, `SyncTestMismatch`, trace générique : le filet avant tout
   le reste), K6 (la CI rapide qui rejoue lint, tests et scénarios), A2 (sortir `games/zombies/`
   avec son `main.rs`), A1 (manifeste, registre, `alacod lint`), B4 (grille spatiale) avec K3
   (bench), B1 (équipes et dégâts), B2 (stats), J1 (le lobby et un scénario à 4), puis les mécaniques
   manquantes de `milestone.md` (points, achats, perks, à terre) comme premier contenu écrit en RON
   seulement, chacune avec son scénario écrit d'abord (§9.7).
3. Chaque chantier terminé ajoute son scénario, sa vidéo et sa ligne dans la doc des conventions (K4).
4. Le découpage en tâches, les voies parallèles, les vagues et l'ordre de merge sont dans
   `docs/taches.md` (M0 et M1 en fiches, M2 en tâches).
