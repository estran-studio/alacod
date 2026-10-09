# Le Relais — prototype des salles

Carte de travail : `games/zombies/assets/maps/le_relais_prototype.ldtk`.
LDtk 1.5.3, grille 16 px, cinq gabarits de 416 × 304 px. Elle ne remplace
pas encore la carte de départ. Les relais, la radio et l'extraction ne sont
pas implémentés ; ce prototype sert à éprouver circulation et combat.

![Plans des cinq gabarits](salles-prototype.svg)

- Accueil : obstacles courts, trois sorties, quatre départs joueurs, pistolet.
- Carrefour : obstacle central et quatre sorties.
- Atelier : deux établis solides contournables, trois sorties, fusil à pompe.
- Infirmerie : cloison en deux sections, trois sorties, juggernog.
- Cour : grand obstacle contournable, quatre sorties, mitraillette.

Chaque gabarit a quatre spawners, dont deux derrière une fenêtre réparable.
Les deux autres créent une pression depuis des positions ouvertes. Les murs
et sols utilisent des tuiles existantes de SunnyLand, comme simple décor
provisoire. Les sprites des achats et fenêtres restent ceux déjà déclarés.

Attention : `WindowVertical` mesure 32 × 16 dans les définitions existantes ;
c'est celle posée dans la cloison horizontale. L'orientation est déterminée
par les dimensions physiques, pas par l'intuition du nom.

Vérifications initiales du 2026-10-09 : lint zombies sans erreur ; flood-fill
de chaque gabarit avec accès à toutes les portes, départs et achats ; génération
native sur graines 1..20 : 10 salles par carte, 20 plans distincts après suppression
de la translation globale, aucun rectangle de salle chevauché. Les départs joueurs
et ennemis ont aussi été contrôlés avec le dégagement des corps de 20 × 20 px,
après correction d'un départ trop proche d'un obstacle. Ces mesures ne
prouvent pas la variété des décisions de jeu. Le générateur peut répéter un
gabarit, y compris Accueil ; il ne garantit pas les cinq rôles dans chaque carte.

**Combat non validé.** Pilote corrigé à quatre acheteurs, graine 1, 500 frames :
aucune failure d'invariant ni desync, mais 0 kill et trois zombies inaccessibles
hors du champ de flux des joueurs. Un pilote plus long observé jusqu'à f6000
reste en V1 sans kill. Le sélecteur du mode vagues (`waves/systems.rs`) accepte
les spawners par distance, y compris dans d'autres salles derrière des portes
fermées. Il n'utilise pas le filtre par salle du système ordinaire
`character/enemy/spawning.rs`. Les essais longs ont été arrêtés pour diagnostic ;
la campagne de combat sur 20 graines n'est pas déclarée réussie. Aucun code
moteur n'a été changé dans ce prototype. Corriger ce chemin est nécessaire avant
de présenter cette carte à portes payantes comme jouable.

Artefacts d'essai dans `target/metrics/le-relais/` (ignorés par git). Assemblage :

```sh
APP_VERSION=x cargo run --example map_generation --profile headless --no-default-features -- \
  games/zombies/assets/maps/le_relais_prototype.ldtk /tmp/le-relais-graine-1.ldtk 1
```

Les cinq gabarits sont éditables dans LDtk. Le cache de tuiles est volontairement
simple ; une régénération des règles dans l'éditeur peut changer leur apparence,
sans changer l'IntGrid de collision. Prévoir une décoration cohérente après
validation du parcours, comme décrit dans `docs/digests/zombies-conception-carte-procedurale.md`.
