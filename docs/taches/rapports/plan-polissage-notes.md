# Notes d'essai du plan de polissage (William)

Fichier d'accueil des notes de William pour les essais humains du plan (`docs/taches/plan-polissage.md`). Une section par essai ;
les autres voies y ajoutent la leur (conflit de fusion prévisible : additionner les sections).

## E4 manette — essai physique, William, 2026-10-28 (S1-1, voie b0)

Build local `e4-manette` (zombies), Chrome, manette physique, page `http://127.0.0.1:8777/loader.html?game=zombies&build=e4-manette`.

- **Verdict** : « ça marche assez bien ».
- **Note (a)** : le contrôle est « un peu trop rigide » → **à régler dans S1-6** (feel, réglages par clone : seuils de stick, accélération,
  inertie), **pas dans S1-1** : aucun seuil (stick 0,4 par axe, visée 0,3, gâchette 0,5/0,35) ni accélération n'a été touché depuis.
  Entrée pour S1-6 : déplacement au stick = huit directions à vitesse constante (`BoxInput` n'a pas de déplacement analogique), ce qui
  explique en partie la rigidité.
- **Note (b)** : une UI des boutons (icônes) quand une manette est branchée, dans les menus et l'aide → pas maintenant ;
  consignée dans `propositions.md` (racine du dépôt).
- Non couvert par cet essai de 2 min (à voir à l'E4 de vendredi 30) : mutation de throne, fin de partie et retour au salon,
  débranchement/rebranchement, L3 pour le sprint, pause qui ne fige pas la partie (D158).

## S1-9 — écoute des sons de zombies (protocoles n° 1 et n° 2 : `docs/taches/rapports/polissage-zombies-sons.md` § 4)

À remplir par William, une ligne par son : « trop fort / ni trop fort ni muet / trop faible ». Pour les sons de l'étape 2, dire aussi si le **repli public** (Kenney, sans écoute de ma part) est acceptable ou à remplacer.

| Son | Verdict |
|---|---|
| tir mitrailleuse | |
| tir pistolet | |
| tir fusil à pompe | |
| tir fusil | |
| recharge | |
| coup porté (hit) | |
| mort d'un zombie | |
| dégâts reçus (hurt) | |
| début / fin de vague (`wave_start`, `wave_end`) : surcharge Sonniss, puis repli Kenney | |
| achat (`purchase`), ramassage (`pickup`) : surcharge, puis repli | |
| fenêtre cassée / réparée (`window_*`, Kenney) | |
| mort par famille de zombie (`death_zombie_*` ; repli public = `death`) | |
| mort du joueur (`death_player`) : surcharge, puis repli | |
| boss : apparition, mort (`boss_spawn`, `boss_death`) : surcharge, puis repli | |
| menus : naviguer, confirmer, retour (`menu_*`) : surcharge, puis repli | |

## S2-11 — écoute des sons de throne (protocole : `docs/taches/rapports/polissage-throne-sons.md` § 4)

À remplir par William, une ligne par famille : « trop fort / ni trop fort ni muet / trop faible » ; dire si le **repli public** est acceptable ou à remplacer (tout est choisi sans écoute de ma part).

| Son | Verdict |
|---|---|
| armes à balles (revolver, fusil à pompe) | |
| lanceurs (lance-grenades, mortier) : tirs de fusil plus graves | |
| énergie (laser, plasma) : tirs de .22 changés de hauteur | |
| lames (lance-lames, disque) : impacts Kenney | |
| recharges (revolver, lanceur, énergie, lames) | |
| morts par famille (rongeurs, bêtes, humains, machines) : surcharge, puis repli | |
| étage / portail, ramassage | |

