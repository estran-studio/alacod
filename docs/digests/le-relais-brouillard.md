# Le Relais — refaire le brouillard d'exploration

## Problème constaté par William

La première version pose des rectangles noirs au-dessus des pièces fermées. Elle
laisse l'extérieur visible et recouvre parfois les portes. Le test de propagation
ne vérifie pas ces défauts de rendu. Ce prototype ne satisfait pas le besoin et doit
être remplacé, pas simplement adouci.

## Comportement cible proposé

Un brouillard global couvre toute la carte, intérieur comme extérieur. La visibilité
vient des joueurs et des ouvertures réellement accessibles, pas de rectangles.

Trois états :

- Inconnu : noir opaque ; ni objets, ni zombies, ni noms ne sont révélés.
- Exploré mais hors de vue : décor fixe assombri, sans personnages ni bonus visibles.
- En vue : décor et contenu visibles normalement.

La découverte est partagée en coopération ; la vue courante est l'union des vues des
joueurs. Une porte fermée bloque la vue. Son côté donnant sur une zone visible reste
lisible, avec son prix d'interaction ; son côté caché ne révèle pas la pièce voisine.
Ouvrir révèle progressivement ce qui devient visible à travers le passage. La
connexion seule ne révèle pas instantanément toutes les pièces accessibles.

Les murs bloquent la vue. Les fenêtres permettent une vue limitée vers l'extérieur,
qui montre l'approche des zombies sans dévoiler toute la cour. Un meuble bas ne
bloque pas automatiquement la vue parce qu'il bloque le déplacement.

Réglages initiaux à essayer : portée de vue 320 px (20 tuiles), portée au travers des
fenêtres 128 px (8 tuiles), transition 200 ms, bord doux de 8 px. Ces valeurs sont
proposées, pas encore validées en jeu. L'adoucissement doit rester à l'intérieur des
zones autorisées : aucune fuite derrière un mur ou une porte fermée.

## Données moteur nécessaires

1. Des identifiants stables de pièces et les connexions explicites des portes.
   Retirer la déduction actuelle par proximité avec une marge de 24 px. Conserver le
   plan du générateur à l'exécution ; la migration vers plusieurs niveaux LDtk reste
   un chantier distinct, compatible avec ces mêmes identifiants.
2. Une grille d'occlusion de vue, construite depuis les murs LDtk et les portes,
   distinguée des collisions de déplacement. Fenêtres et obstacles ont une politique
   de visibilité explicite. Les limites de défense des spawners ne définissent plus
   les régions du brouillard.
3. Une configuration opt-in par jeu, pour activer ce système dans zombies et régler
   portées, couleurs et transition. Aucun comportement implicite pour les autres jeux.

## Implémentation par étapes

### 1 — Calcul de visibilité, avant le rendu

Calculer un champ de vision sur la grille à partir des joueurs (shadowcasting ou
algorithme équivalent), avec occlusion par murs et état courant des portes. Produire
un masque visible et un masque exploré couvrant toute la carte.

La découverte historique ne doit pas dépendre des frames prédites : accumuler la
mémoire depuis des états confirmés, ou la reconstruire depuis une histoire confirmée.
La vue immédiate peut suivre la prédiction ; un rollback recalcule cette vue et ne
laisse aucune découverte fantôme. Ne pas introduire de règles de combat dans le rendu.

### 2 — Composition du rendu

Remplacer les sprites noirs par un masque de visibilité en coordonnées monde,
composé avec le rendu du monde. Choisir le raccord Bevy après un petit prototype
technique ; ne pas promettre un shader avant d'avoir vérifié son intégration caméra.
Le HUD reste hors masque. Le masque suit caméra, zoom et résolution.

Le décor exploré peut rester en mémoire. Les entités dynamiques et leurs indices
(barres de vie, chiffres, noms, bonus, télégraphes) sont filtrés par la vue courante.
Ne pas simplement assombrir l'image contenant les ennemis hors de vue : cela
laisserait apparaître leur position. Prévoir le filtrage de tous leurs calques.

Les portes suivent l'occlusion géométrique : afficher leur face accessible grâce à
la limite du champ visible, sans les redessiner systématiquement au-dessus du fog.
Les informations d'interaction ne sont affichées que depuis le côté accessible.

### 3 — Finition visuelle

Interpolation courte, bords doux bornés par l'occlusion, teinte sobre des zones
explorées. Valider d'abord les portes, fenêtres et angles ; ajouter une animation de
brume seulement si elle améliore la lisibilité du jeu.

### 4 — Validation et revue William

Tests : pièce adjacente fermée, porte ouverte, porte déconnectée, angle de mur,
fenêtre, extérieur, exploration coop, rollback avant achat, nouvelle partie.

Captures avant/après ouverture, retour dans une salle explorée, zoom minimum/maximum,
bonus et zombie derrière une porte. Vérifier que ni sprite ni libellé ne fuit.
Comparer les traces : ce chantier de présentation ne doit pas changer le combat.
Mesurer le coût sur la carte de 82 × 70 tuiles ; limiter les recalculs aux changements
de case des joueurs et aux changements des occluders, puis interpoler dans le rendu.

## État

Plan approuvé par William, implémenté sur `m0-revue-suite-contenu`.

La première version à rectangles est remplacée par une texture RGBA unique en
coordonnées monde, couvrant l'ensemble de la grille. Le champ de vision est calculé
par parcours supercover (coins opaques), avec mémoire confirmée, révélation de 200 ms
et adoucissement borné aux cases visibles. Les connexions exactes sont conservées
comme métadonnées LDtk, sans migration vers plusieurs niveaux de rendu.

Portées initiales : 20 tuiles ; après une fenêtre, 8 tuiles maximum, en restant dans
la portée générale. Les occluders sont les murs IntGrid du Relais et les portes
fermées. Le mobilier de cette carte dessiné dans Walls bloque donc encore la vue ;
une grille distincte peut le rendre transparent s'il est requalifié en obstacle bas.
La texture est une solution CPU, sans shader spécifique ni animation de brume.

Les captures du replay `revue-fog.ron` montrent la porte fermée à la frame 600,
l'achat à la frame 1023, puis la vue au travers de la porte à la frame 1200 et les
fenêtres vers l'extérieur. Les chiffres et bonus sont filtrés par la vue courante.
Le ressenti visuel reste à valider par William. Le p2p et le WASM ne sont pas validés
pour ce changement ; voir le rapport de livraison pour les autres contrôles.
