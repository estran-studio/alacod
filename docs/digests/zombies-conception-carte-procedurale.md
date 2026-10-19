# Démo zombies : conception d'une carte procédurale

Étude du 2026-10-09 avec William. Proposition de conception, pas une carte implémentée.
William valide S5 B, puis donne priorité à S2/S8 : plusieurs salles intéressantes,
un but et une expérience différente à chaque partie. Les valeurs ci-dessous sont
des hypothèses de prototype à éprouver en jeu.

## Ce que fait le dépôt aujourd'hui

LDtk 1.5.3 sert à dessiner les gabarits ; le moteur choisit et assemble leurs copies.
Un niveau LDtk correspond à une salle, sur une grille de 16 px. `Walls` définit les
collisions, `LevelConnection` les passages sur les bords, `Entities` les placements.
Le champ de niveau `spawn` désigne les gabarits de départ. La disposition des niveaux
dans l'éditeur n'est pas le plan de la partie générée.

Deux passages sont compatibles si leurs côtés sont opposés, leur largeur et leur
position sur le bord identiques (`Connection::are_matching`). Le mode `Basic`
place un départ puis ajoute des salles ; un chevauchement ou une sortie des limites
ferme la connexion tentée. Il ne cherche pas un autre candidat pour cette tentative.
`max_room` vaut 10 par défaut : c'est un plafond, pas un nombre garanti.
L'algorithme ajoute une salle à un parent ; il n'a pas d'étape reliant deux salles
déjà placées. On ne peut donc pas compter sur des boucles entre salles générées.
Les gabarits de départ peuvent être réutilisés ensuite (dette D10 connue).

La partie normale utilise `default_seed: 123456` dans `game.ron`. Relancer le
programme avec cette configuration reproduit sa carte. La variété exige aussi
une graine de partie renouvelée, conservée dans l'enregistrement et partagée entre
clients ; les tests doivent garder leurs graines explicites.

Inventaire JSON de `maps/avant_poste.ldtk` effectué pendant cette étude :

| Gabarit | Taille en px | ZombieSpawn | Fenêtres | Récompense utile |
| --- | --- | ---: | ---: | --- |
| Depart | 416 × 304 | 5 | 2 | Une arme murale |
| Couloir | 416 × 304 | 0 | 0 | Une arme murale |
| Cuisine | 416 × 304 | 0 | 0 | CrateLocation non fonctionnelle |
| Chambre | 416 × 304 | 0 | 0 | CrateLocation non fonctionnelle |
| Poste | 416 × 304 | 0 | 4 | CrateLocation non fonctionnelle |
| Jug | 416 × 304 | 0 | 0 | Un soda |
| Armurerie | 416 × 304 | 0 | 0 | Deux armes murales |
| Cave | 416 × 304 | 0 | 0 | CrateLocation non fonctionnelle |
| Cellier | 416 × 304 | 0 | 0 | Un soda |

Correction après essai du prototype : le système ordinaire
`character/enemy/spawning.rs` filtre ses candidats par salle, **mais le mode
vagues appelle son propre sélecteur dans `waves/systems.rs`**, qui ne filtre que
par distance. Ajouter des spawners derrière des portes fermées peut donc faire
apparaître des zombies inaccessibles et bloquer la première vague. La première
lecture avait confondu les deux chemins. La sélection du mode vagues doit être
corrigée avant de valider la démo avec plusieurs salles de combat.
Les fenêtres laissent tirer au travers ; une fenêtre cassée laisse passer les
ennemis, mais bloque encore les joueurs.

Les prix des portes générées sont recalculés : `750 + 250 × profondeur`, et
`electrify` est dérivé de la profondeur. Modifier seulement `price` dans LDtk ne
donne pas le contrôle attendu sur l'économie du parcours. Les armes murales et
sodas sont fonctionnels ; `CrateLocation` ne l'est pas. Aucun terminal d'objectif
ou générateur électrique jouable n'est établi par cette lecture.

Sources locales : `docs/conventions.md` §1 ; `crates/map/src/generation/context.rs`,
`imp/basic.rs`, `mod.rs` ; `crates/map_ldtk/src/generation/from.rs`, `to.rs`,
`game/local.rs` ; `crates/game/src/character/enemy/spawning.rs` ; manifeste zombies.
Lecture et inventaire statique seulement : pas de nouvelle campagne de graines ici.

## Ce qu'on retient de Call of Duty Zombies

La présentation officielle de Terminus décrit la concurrence entre dépenses pour
ouvrir des chemins et dépenses pour améliorer son équipement. Elle mentionne
aussi des espaces de manœuvre pour rassembler les hordes, une quête principale
et une extraction. Ce sont des fonctions à adapter au jeu 2D.

Interprétation de design pour Alacod : chaque ouverture doit changer une décision.
Une salle peut offrir de l'espace, un achat, un raccourci ou une avancée vers le but.
Des voies de fuite lisibles permettent au joueur de comprendre pourquoi il meurt.
La quête donne une raison de quitter un endroit confortable et de prendre un risque.

Références consultées :

- [Treyarch : Terminus, économie, parcours, quête et extraction](https://www.callofduty.com/blog/2024/08/call-of-duty-black-ops-6-zombies-deep-dive-terminus-map-intel).
- [Présentation officielle de Liberty Falls et de ses quêtes](https://www.callofduty.com/au/en/blog/2024/08/call-of-duty-next-black-ops-6-reveal-zombies-liberty-falls-showcase-intel).
- [LDtk : mondes et niveaux](https://ldtk.io/docs/general/world/).

## Proposition : Le Relais

Un poste de communication abandonné. L'équipe veut remettre la radio en service
et évacuer. Cible initiale : une partie de 10–15 minutes, 1–4 joueurs, 8–10 salles
assemblées à partir de 12–16 gabarits. Ce sont des objectifs de conception, pas des
durées mesurées ni des garanties du générateur actuel.

Parcours envisagé : démarrer dans l'accueil, choisir entre deux ailes, atteindre
deux relais de courant, rejoindre la radio puis tenir la cour d'extraction 60 s.
Les relais sont activables dans n'importe quel ordre. Le joueur voit dès le départ
le but « Rétablir 2 relais → appeler les secours → tenir la cour ».

```mermaid
flowchart LR
    A[Accueil] --> H[Carrefour]
    H --> W[Atelier et armurerie]
    H --> I[Infirmerie et réserve]
    W --> R1[Relais A]
    I --> R2[Relais B]
    R1 --> T[Radio]
    R2 --> T
    T --> C[Cour d'extraction]
    W -. Raccourci payant .-> I
```

Schéma de conception : les convergences et le raccourci ne sont pas garantis par
`Basic`. Les zones fonctionnelles restent présentes ; les salles intermédiaires,
l'orientation des ailes et certains emplacements de récompenses varient.

| Salle | Décision de jeu | Géométrie proposée | Risque / récompense |
| --- | --- | --- | --- |
| Accueil | Apprendre à défendre puis partir | Zone centrale libre, 2 sorties, 2–3 entrées ennemies | Arme accessible et pression S5 B |
| Carrefour | Choisir la prochaine dépense | Repère central, 3 sorties, visibilité des directions | Distribution vers les deux ailes |
| Atelier | Tenir ou traverser | Établi central contournable, 2 issues opposées | Arme forte, approche ennemie sur deux côtés |
| Infirmerie | Acheter de la survie | Deux chemins autour d'une cloison | Soda, détour plus long mais plus mobile |
| Réserve | S'engager pour une récompense | Petite poche avec retour visible | Achat facultatif, coût du cul-de-sac |
| Relais A/B | Progresser sous pression | Petite arène avec deux issues, terminal visible | Activation, pression accrue à tester |
| Radio | Achever le parcours | Entrées lisibles, obstacle central bas dans la composition | Appel des secours après les deux relais |
| Cour | Se déplacer pour survivre | Grande boucle autour d'un obstacle, 3–4 entrées ennemies | Défense finale puis victoire |

Les obstacles de décor doivent être lisibles comme solides. Une longue ligne de
tir ne doit pas permettre de couvrir toutes les entrées. Les passages de circulation
principaux visent au moins 3 cellules (48 px) pour les corps de 20 px ; les portes
existantes de 32 px restent des étranglements volontairement courts.
Les variantes d'une salle doivent changer couverture, trajet ou entrées ennemies.
Chaque salle dispose d'un repère visuel et d'une silhouette reconnaissables.

## Où placer l'aléatoire

1. Construire le graphe fonctionnel avec départ, deux relais, radio et cour garantis.
2. Choisir les gabarits compatibles avec chaque rôle et les placer sans chevauchement.
3. Ajouter 1–2 branches facultatives et au moins un raccourci formant une boucle.
4. Choisir des variantes d'obstacles et de récompenses dans des emplacements validés.
5. Vérifier accès aux objectifs, navigation, activation des spawners et économie.

Un même seed doit reproduire salles, placements et objectifs. Deux graines devraient
modifier au moins deux décisions parmi route, achat prioritaire, espace de défense
et chemin de retour. Déplacer la carte entière n'est pas une diversité utile.
Limiter les répétitions d'un gabarit nommé ; garantir un départ unique et les achats
essentiels. Après un échec de placement, réessayer selon une procédure bornée et
déterministe ; utiliser une carte de secours valide si aucune tentative ne convient.

L'économie doit être simulée avec S5 B : 60 points par kill, 10 par hit et par
réparation aujourd'hui. Les dégâts par tir et les choix de dépense affectent le
moment où une porte devient abordable ; le nombre de kills seul ne suffit pas.
Cible à tester : premier choix de route vers V2–V3, premier relais vers V4–V5.
Ne pas figer les prix avant une mesure solo et coop sur les parcours retenus.

## Étapes proposées

**Prototype de salles** : produire une nouvelle carte LDtk de travail avec accueil,
carrefour, atelier, infirmerie et cour ; variantes, fenêtres et spawners dans toutes
les salles de combat. Tester la circulation et le ressenti avec les mécanismes
actuels. Les objectifs restent explicitement une maquette tant qu'ils n'existent pas.

**Assemblage guidé** : définir rôles, contraintes, unicité, boucles et contrôle des
prix à partir des interfaces de M2 lorsque disponibles. Le prompt de branche
interdit de toucher au moteur des salles/objets M2 ; cette étude n'y apporte aucun
changement et ne crée pas un générateur concurrent.

**Objectif jouable** : deux relais, dépendance de la radio, défense finale, HUD de
progression, victoire et enregistrement. Réutiliser les mécanismes moteur disponibles
après vérification de M2 ; ne pas inventer des champs LDtk que le loader ignorerait.

**Validation de la démo** : audit statique de 200 graines (objectifs présents,
connectivité, répétitions, boucles, variété des plans), puis au moins 20 graines
jouées par bots avec objectifs adaptés, synctest et invariants. Comparer trois
graines à la main avec William. Cible : 0 carte inaccessible, 0 soft-lock, 0 desync ;
mesurer temps d'accès, portes achetées, dégâts et durée de partie. Les bots acheteurs
actuels ne prouvent pas à eux seuls qu'une quête est terminable.

Prochaine décision de design : privilégier ce complexe de communication, ou garder
le thème domestique de l'avant-poste et lui donner le même parcours d'évacuation.

## Premier prototype construit

Les cinq gabarits sont dans `maps/le_relais_prototype.ldtk`, avec une vue des
plans et les résultats dans `docs/captures/le-relais/README.md`. Le manifeste
garde avant_poste. Génération native sur 20 graines : 10 salles à chaque fois,
20 plans distincts hors translation, aucun chevauchement. Le pilote révèle le
problème de sélection des spawners du mode vagues décrit ci-dessus ; la série
de combat sur 20 graines n'est pas validée. Aucun code moteur modifié.

## Direction choisie : sécuriser un petit bâtiment

William corrige la direction le 2026-10-09 : les zombies doivent venir de dehors
et l'équipe défendre l'intérieur d'un bâtiment compact. Il approuve cinq pièces
avec fenêtres barricadées, portes et sortie tardive dans la cour.

Le prototype est remplacé par quatre orientations d'un même bâtiment de cinq
pièces. Les huit spawners sont tous à l'extérieur, dans une bande continue qui
leur permet de rejoindre les fenêtres de l'accueil même portes fermées. Pièces
et cour sont dans un seul niveau LDtk ; le générateur choisit un plan complet.
Cette disposition vérifie la boucle de défense avant l'assemblage procédural des
pièces. Le pilote atteint V5 (7236 frames, 27 kills, 15 dégâts, aucune mort ni
failure ni desync). Résultats à jour et aperçu : `docs/captures/le-relais/README.md`.
Le sélecteur de vagues n'a pas été modifié : la topologie extérieure résout le
blocage du premier prototype. Les objectifs interactifs et une variété stratégique
plus forte restent à implémenter.

Campagne complète du bâtiment : 20/20 graines à quatre acheteurs atteignent V5,
0 mort/mise à terre/desync/failure, 134 dégâts cumulés. Entrée V5 médiane f6940.
La graine 11 atteint V5 à f18713 après une V3 de 12754 frames : ce retard de
navigation mérite une revue humaine. Le ressenti de ce nouveau bâtiment n’est
pas encore validé. Le manifeste conserve avant_poste et la graine fixe.

## Retour à l'assemblage de pièces (2026-10-17)

William tranche le 2026-10-16 : la génération revient à 100 % à son algorithme (des niveaux
LDtk qui s'emboîtent par leurs prises `LevelConnection`, assemblés au hasard). Le bâtiment
« Le Relais » à grille fixe et le générateur organique `compound` sont retirés
(`docs/taches.md`, journal du 2026-10-17 ; guide de conception des pièces :
`docs/conventions.md` §1). Cette section relit les cartes de Call of Duty Zombies sous
l'angle de la **structure** (ce qu'un assembleur de pièces doit reproduire), puis propose la
bibliothèque et les deux règles qui manquent.

### Structure des cartes classiques

Sources : [Nacht der Untoten](https://callofduty.fandom.com/wiki/Nacht_der_Untoten),
[Der Riese](https://callofduty.fandom.com/wiki/Der_Riese) (Call of Duty Wiki, « Map Layout »),
[Kino der Toten](https://codzombie.com/maps/kino-der-toten) (CODZombie, « Power »).

| Carte | Salles | Forme | Portes | Ce qui tire le joueur |
| --- | ---: | --- | --- | --- |
| Nacht der Untoten | 3 | Départ → « Help » (boîte mystère) ; départ → étage (deux salles) : un Y, pas de boucle | 1000, 1000, 1000 | boîte mystère, armes murales 200 → 600 → 1200 → 1500/1800 |
| Der Riese | ~10 | Départ (mainframe) avec **deux portes** ; deux ailes symétriques (garage / labo) qui **retombent toutes deux dans la cour** (générateur) ; l'usine au fond ; le courant **ouvre des portes de raccourci** et un pont entre les ailes | 750, 750, 1000, 1000, 1250, 1250 | le courant (au bout), les téléporteurs, le Pack-a-Punch, un perk par zone (Jug loin) |
| Kino der Toten | ~9 | Hall avec **deux routes en miroir** (étage / rez-de-chaussée) qui **finissent toutes deux au théâtre** (courant) ; le courant rouvre les portes théâtre → hall : la carte devient une **boucle** d'entraînement | 750 → 1250 | MP40 sur la route haute, courant, perks, téléporteur → Pack-a-Punch |

Constantes, carte après carte :

1. **Un hub de départ à deux sorties**, puis deux ailes qui convergent dans une **grande salle
   ouverte** (cour, théâtre) : c'est là qu'on « entraîne » la horde. Les petites salles et les
   couloirs sont des transits risqués, pas des lieux où rester.
2. **Gradient de prix** des portes par profondeur (750 → 1000 → 1250) et des armes murales
   (200 près du départ, 1500 au fond). Chaque ouverture est un choix de dépense contre
   l'équipement : c'est tout le jeu des premières vagues.
3. **Les zombies viennent de dehors** : 2 à 5 barricades (fenêtres) par salle, sur les façades
   extérieures. Une salle sans fenêtre ni voisine est un refuge ; une salle à quatre fenêtres,
   un piège.
4. **Une récompense par salle** (arme murale, perk, boîte) et **un but au bout** (courant, puis
   Pack-a-Punch) qui oblige à quitter le refuge.
5. **Raccourcis débloqués plus tard** (portes au courant) qui ferment des boucles : la carte
   change de forme en cours de partie.

### Ce que l'assembleur fait déjà, ce qui manque

Déjà là (`BasicMapGeneration`, grammaire d'étage, M2-E1) : arbre de pièces tiré au hasard, prix
des portes `750 + 250 × profondeur`, fenêtres / armes murales / sodas / apparitions recopiés de
chaque pièce, types de salle (`room_kind`) avec salles obligatoires et boss à distance maximale
en cul-de-sac (`FloorGrammar`), victoire `Waves` quand un boss meurt (`RunContext::objective_done`).

Manque pour la cohérence de CoD, par ordre d'importance :

1. **Les boucles.** L'assembleur construit un **arbre** : une pièce n'est reliée qu'à son
   parent ; deux pièces posées côte à côte avec des prises qui se font face restent séparées
   (`docs/digests`, étude du 2026-10-09, déjà noté). Sans boucle, pas de convergence des ailes
   ni d'entraînement de horde entre salles. Proposition : une **passe de fermeture** après le
   placement, dans `BasicMapGeneration` — pour chaque paire de pièces posées, bord à bord, dont
   deux prises libres se font face (même décalage, même longueur : la règle d'emboîtement
   existante), relier ces prises comme une connexion ordinaire (portes appariées, prix de la
   plus profonde). Déterministe (paires triées par iid de pièce), additive (le placement ne
   change pas), mesurable (l'aperçu compte les boucles). Une porte de boucle est une porte
   comme les autres : le raccourci « au courant » de CoD devient simplement une porte plus
   chère, sans mécanique de courant (le champ `electrify` des portes n'a aujourd'hui aucun
   effet en jeu).
2. **L'extérieur comme pièces.** Le mode vagues choisit un spawner par distance au joueur
   (`waves/systems.rs`, `min/max_player_distance`), pas par salle : un spawner inaccessible fait
   apparaître des zombies perdus. Avec des pièces, l'extérieur se dessine **comme des pièces**
   (`cour`, `ruelle`, `jardin`) : grandes, sans mur intérieur, **sans mur sur leur bord** du côté
   du bâtiment, portant tous les `ZombieSpawn`. Elles s'accrochent au bâtiment par des prises
   (porte extérieure payante, comme la cour de Der Riese) ; les fenêtres des salles sont
   dessinées sur les façades qui touchent une cour, et les zombies les cassent pour entrer
   (`enemy_target_selection`, mécanique déjà en place). Règle de conception à vérifier par
   l'aperçu : chaque cour touche au moins une fenêtre.
3. **Taille et bornes en données.** `max_room` (10) et les bornes ±1000 px sont codés dans
   `configure_map` : 9 pièces de 26×19 au plus. Une carte CoD (10–14 pièces dont deux grandes)
   demande `max_room` et bornes dans `game.ron`.
4. (Plus tard) prix des armes murales dérivé de la profondeur comme celui des portes ; un
   « générateur » qui ouvre d'un coup les portes `electrify` ; la boîte mystère.

### Bibliothèque proposée (v1)

**Standard de prises, une seule famille** : toute prise fait 2 cases ; N/S au décalage **x = 12**,
E/W au décalage **y = 8**. Toute pièce d'au moins 15 × 11 cases peut la porter, les grandes
(52 × 38) comme les petites. Conséquence de la règle « même décalage » : deux pièces se collent
**coin haut-gauche aligné** (`get_connecting_room_position`, décalage nul) ; une petite pièce
accrochée au nord d'une grande se place à son bord gauche, jamais centrée. Si cette asymétrie
gêne à l'usage, le bouton est connu et minuscule : `Connection::are_matching` exige
`starting_at` égaux alors que le placement sait déjà décaler (`offset` dans `room.rs`) ; le
relâcher autoriserait une prise à x = 38 sur une grande pièce, mais c'est une règle de
l'algorithme d'origine, à décider explicitement, pas en passant.

| `room_kind` | Pièces | Prises | Contenu | Rôle CoD |
| --- | --- | --- | --- | --- |
| `accueil` (`spawn`) | 3 | W + E (parfois N) | 4 `PlayerSpawn`, 1 arme à 200–500, 2–4 fenêtres sur les façades sans prise | hub à deux sorties |
| `couloir` | 4 | 2 opposées (W/E ou N/S) | 1 arme murale, 1–2 fenêtres, aucun refuge | transit risqué |
| `salle` | 6 | 2–3 | 1 arme ou 1 soda, 2–3 fenêtres, mobilier contournable | combat, choix de dépense |
| `carrefour` | 2 | 4 | repère central, 1 fenêtre | distribution des ailes |
| `impasse` | 3 | 1 | soda ou arme chère, 1 fenêtre | récompense à aller chercher |
| `arene` | 2 | 3–4, 52 × 38 | boucle autour d'un obstacle, 2 armes, 1 soda, 4 fenêtres | cour / théâtre, entraînement |
| `cour` (extérieur) | 3 | 1–2 | `ZombieSpawn` ×4–6, bord ouvert vers le bâtiment, clôtures | d'où viennent les zombies |
| `boss` | 2 | 1 | `CharacterSpawn boss_zombie` + gardes, 4 fenêtres | salle finale, victoire |

Vingt-cinq pièces, huit types, une famille de prises : chaque prise libre a une dizaine de
candidats, contre un seul dans `avant_poste` (neuf signatures toutes uniques, mesuré le
2026-10-17). Grammaire `zombies` (`FloorGrammar`) : `rooms: (10, 14)`, `required: {arene: 1,
cour: 2, impasse: 1, boss: 1}`, `filler: salle`, `min_boss_distance: 4` ; jouée par
`start_map: "floor:<grammaire>"` en mode `Waves` (chemin `floor:` de `resolve_map_config`, à
vérifier en `Waves` : il n'a servi qu'en `Floors`).

```mermaid
flowchart LR
    A[accueil] --> C1[couloir] --> S1[salle] --> AR[arene]
    A --> S2[salle] --> K[carrefour] --> AR
    K --> I[impasse]
    S1 --- CO1[cour]
    AR --- CO2[cour]
    AR --> C2[couloir] --> B[boss]
    S2 -. boucle fermée .- S1
```

Ce qui varie d'une graine à l'autre : quelle aile est courte, où tombent l'arène et les cours,
combien de boucles se ferment, quelles salles portent soda ou arme. Ce qui ne varie pas :
deux sorties au départ, une arène, deux cours, un boss à au moins quatre portes.

### Ordre proposé

1. **Fait le 2026-10-17** : fermeture des boucles dans `BasicMapGeneration`, prises libres
   murées (porte retirée), aperçu qui compte boucles et prises murées. Reste à l'aperçu : les
   cours sans fenêtre (quand la bibliothèque aura des cours).
2. **Fait le 2026-10-17** : `entry.map` de `game.ron` (`max_rooms`, bornes, `aligned_openings`,
   `close_loops`, `wall_unused_openings`) ; le collage de pièces de tailles différentes est
   corrigé au passage (`get_connecting_room_position`).
3. Bibliothèque v1 : squelettes générés par script (murs de bord, prises au standard, portes
   et fenêtres posées), puis dessin à la main dans LDtk ; `make map_render` à chaque passe.
4. Grammaire `zombies` et `start_map: "floor:..."` en `Waves` ; campagne de bots
   (`alacod-sim`, 20 graines, V5) comme pour Le Relais.

## La Fonderie (2026-10-17)

Bibliothèque réelle, pas la v1 à 25 pièces : 16 sections dessinées, une grammaire, deux
raccords moteur. Le but est une démo jouable, pas un éditeur de niveau générique.

**Deux raccords moteur manquaient pour ce genre de carte :**

1. **Les champs d'activation des spawners étaient détruits par l'assemblage.** Le runtime sait
   déjà ne faire apparaître une vague que dans la section occupée (`activation_area`), mais
   `EnemySpawnConfig` était vide : `Basic` recréait chaque `ZombieSpawn` avec `vec![]`. Désormais
   les quatre champs `active_x/y/width/height` sont conservés (en pixels locaux au niveau dans la
   bibliothèque, convertis en monde à l'export), et leur absence ne change aucune carte existante.
2. **Une salle typée ouvrait gratuitement toutes ses portes.** `room_lock_system` retirait
   collider et interaction des portes proches de toute salle `room_kind`, y compris celles qui ne
   verrouillent pas (`locks: false`). En `Waves`, seules les salles verrouillantes (boss) s'ouvrent
   désormais par défaut ; les ateliers/armureries gardent leurs portes payantes.

**Contenu** (`scripts/construire-fonderie.py`, idempotent, génère
`games/zombies/assets/maps/la_fonderie.ldtk`) : 16 sections de 32×32, 64×32 ou 32×64, chacune une
situation lisible — hall à deux sorties, ateliers aux établis/serpentins, galerie de tir
transversale, vestiaires en coude, poste d'aiguillage, réserve traversante, cour de chargement
(64×32) et halle de convoyeur (32×64) en espaces de manœuvre, deux armureries (fusil 1 000, seul
non possédé au départ), deux infirmeries (Juggernog en impasse), deux finales de boss. Chaque
section porte 2 à 4 **cours extérieures** : le spawner est **dehors**, dans un vide ouvert sur le
bord de la carte, et le mur du bâtiment (avec la **fenêtre**) le sépare de l'intérieur — le zombie
apparaît dehors, casse la vitre et entre ; le joueur ne peut jamais franchir une fenêtre (intacte
ou cassée), donc il sécurise l'intérieur et répare. C'est le modèle CoD, vérifié sur la carte
assemblée : tous les spawners sont atteignables depuis le vide extérieur, aucun n'est enclos. Le
boss (`CharacterSpawn boss_zombie`, dormant) est masqué depuis son entrée par une chicane.

Grammaire `floor:la_fonderie` : 10–12 salles (9–11 avant les boucles), départ, `arene` 1,
`armurerie` 1, `infirmerie` 1, `boss` 1 (distance ≥ 4), remplissage `atelier`, `loops: 2`.
Types `rooms/*.ron` : seul `boss` verrouille.

### Boucles (2026-10-19)

Revue des rendus de six graines (William, 2026-10-19) : 0–1 boucle par graine, une grille de
pièces identiques, 9 à 12 prises murées, des trous dans la silhouette, un seul chemin vers le
boss. Diagnostic sur 20 graines : la fermeture (`close_loops`) n'oublie aucune paire ; sur 190
prises murées, 156 donnent sur du vide et 34 sur une voisine sans prise en face. Le tirage
uniforme des poses étale l'arbre ; un poids vers les poses qui ferment une boucle ne change
presque rien (0,40 → 0,55), ces poses n'existant qu'une à trois fois par plan.

Retenu : `FloorGrammar::loops` (`docs/conventions.md` §40), un nombre de boucles **exigé** ; le
remplissage préfère une pose qui en ferme une tant qu'il en manque, le plan est refusé sinon,
et la distance au boss se mesure raccourcis compris. `la_fonderie` : `loops: 2` avec 10–12
salles (avec 9–11, deux boucles échouent sur la plupart des graines).

| Mesure | Arbre (avant) | `loops: 2` |
| --- | ---: | ---: |
| Boucles par graine (40 / 200 graines) | 0,40 | 2,11 (aucune sous 2) |
| Graines avec au moins une boucle | 15/40 | 200/200 |
| Prises murées par graine | 9,4 | 10,1 |
| `alacod-sim`, 1 `acheteur`, graines 1–10 : victoires | 7 (3 plafonds) | 8 (1 mort, 1 plafond) |

**Limites restantes** (contenu, pas l'assembleur) : toutes les prises sont `@14+4` sur une
grille de 32 cases, d'où l'effet de grille ; les côtés longs des pièces 64×32 / 32×64 n'ont
qu'une prise (moitié gauche/haute), leur autre moitié mure toute voisine ; le départ, les
armureries et les infirmeries ont 1–2 prises ; les prises murées montent légèrement avec la
taille. Plans compacts : quatre pièces autour d'un coin y mettent leurs quatre courettes, qui
forment une cour intérieure fermée (encore une source de zombies derrière des fenêtres, mais
plus « dehors »). Suite : varier les gabarits (prises décalées, seconde prise sur les côtés
longs, couloirs 32×16, pièces à trois ou quatre prises pour le départ).
