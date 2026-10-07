# Rapport — m0-graine-100-fenetre (zombie figé contre une fenêtre intacte, test_map graine 100)

Branche partie de `origin/main` 4bdbd4f, mergée avec `origin/main` avant la livraison (D53
comprise).

## État en cours

Vérifié, livré. Suite complète relancée après le merge de D53 ; attentes vertes ; target purgé. Traces à bénir par orch : les 5 d'`arena_tir` (prouvées
plus bas) et le nouveau scénario `test_map_fenetre_graine_100`, qui n'a pas encore de trace.
Aucune trace `clone_*`, `equilibrage_*` ou `bots_four_mixed` ne bouge. `clone_quad` est inchangé.

## Diagnostic

Rejeu de la graine 100 avec le binaire `f80b82b` (4 `acheteur`, `--max-frames 20000`) : la vague 1
reste à 5 kills sur 6 et la sim finit en soft-lock à f20 000, comme dans le relevé de b0.

**Moteur : fautif.** Le dernier zombie (`zombie_full`) est à (425,2 ; 402,2). Son collider
(20 × 20, décalage y −6) couvre x 415,2–435,2 et y 386,2–406,2. Il **chevauche de 3,2 px la
fenêtre intacte 41** (440, 400, 16 × 16).

- Dans `move_enemies`, `check_wall_collision` refusait tout pas dont la position d'arrivée
  chevauche un mur ou une fenêtre fermée.
- Un ennemi déjà incrusté chevauche encore l'obstacle après n'importe quel petit pas, **même en
  s'en éloignant**. Il ne pouvait donc plus bouger.
- Il ne frappe pas non plus la fenêtre : elle est **derrière** lui. Sa case suivante, (25, 25), est
  à l'ouest, vers le joueur. La règle de `enemy_target_selection` ne retient que les fenêtres sur
  son chemin.

Cause historique de l'incrustation : **probable, non datée.** La piste est la réparation d'une
fenêtre pendant qu'un ennemi la chevauche, car `handle_window_repair` n'a pas de garde contre ce
cas. Je n'ai pas identifié la frame. Voir D52 plus bas.

**Bots : non fautifs.**

- Les quatre acheteurs sont enfermés dans la salle de départ, avec 630 à 710 pièces chacun.
- Les portes de sortie coûtent 750 ou plus, et aucun ennemi n'est atteignable.
- Rester sur place est donc la conduite attendue d'un `acheteur`. Rien n'est changé dans
  `crates/bots`.

## Correctif (`crates/combat`, `crates/game`)

- `combat::collider` reçoit trois fonctions : `half_extents`, `overlap_area` (aire du recouvrement
  des deux AABB) et `step_blocked_by(from, to, …)`.
- Règle de `step_blocked_by` : un pas est bloqué s'il arrive en collision **et** qu'il entre dans
  l'obstacle, ou qu'il ne réduit pas strictement un recouvrement déjà présent.
- Conséquence : un ennemi incrusté peut s'en dégager, mais ne peut pas s'enfoncer davantage. Hors
  recouvrement au départ, le résultat est identique à l'ancien test.
- `pathing.rs` (`move_enemies`, `check_wall_collision`) : les deux tests contre les murs et les
  fenêtres fermées passent par `step_blocked_by(start, pos, …)`, où `start` est la position du
  début de la frame.
- `slide_axes` n'a pas changé : il essaie toujours le pas complet, puis X seul, puis Y seul.
- Test unitaire `slide_tests::sortie_d_un_recouvrement`, avec la géométrie exacte de la graine 100 :
  - le pas vers l'ouest passe ;
  - le pas vers l'est (s'enfoncer) est bloqué ;
  - entrer dans la fenêtre depuis l'espace libre reste bloqué.
- Scénario figé `test_map_fenetre_graine_100` : le zombie est placé à la position exacte de la
  graine 100 (`CharacterPlacement`, f1), avec un joueur immobile et la grâce de vague prolongée.
  - **Avant le correctif**, les 4 attentes échouent : le zombie reste à 525,37 px du joueur de f2 à
    f599.
  - **Après**, les 4 passent :

    | frame | distance au joueur |
    |---|---|
    | f100 | 397 px |
    | f300 | 162 px |
    | f599 | 31 px |

  - La fenêtre 39 est cassée à f377 et le joueur touché à f483.

**D52 (ouverte par orch) :** empêcher de réparer une fenêtre qu'un ennemi chevauche. Cette garde
est hors de cette tâche. Le correctif ci-dessus traite le symptôme (ennemi figé), quelle que soit
l'origine de l'incrustation.

## Mesures (une sim à la fois)

| | avant | après |
|---|---|---|
| graine 100, `f80b82b` | soft-lock, vague 1, 5 kills, f20 000 | — |
| graine 100, `main` 4bdbd4f | vague 5 à f7158, 49 kills | vague 5 à f7184, 49 kills |
| `test_map` 1..10, 4 acheteurs | 10/10 vague 5, 0 mort | **identique graine par graine** (frames et kills) |
| `test_map` 11..20 (avant seulement, décision d'orch) | 10/10 vague 5 | — |
| `bench_horde`, 2 passages | 33,7 / 36,6 fps | 35,6 / 37,0 fps |
| `bench_horde` après le merge de D53 | — | 44,9 fps (un passage, charge 7,4 : machine pas au calme) |

- Sur `main`, la graine 100 ne reproduit plus le soft-lock : la partie diverge plus tôt depuis
  D48 et D51. Le correctif la change quand même (f7158 → f7184), signe qu'un ennemi s'y dégage
  d'un recouvrement.
- Sur les graines 1..10, rien ne bouge : aucun ennemi n'y est jamais incrusté.
- `bench_horde` : aucun coût mesurable. Le calcul d'aire ne s'exécute que si l'arrivée est déjà en
  collision, c'est-à-dire quand le pas est de toute façon rejeté ou ajusté.

## Traces qui bougent : `testbed/arena_tir.ldtk` (5)

`enemy_ring`, `enemy_ring_quad`, `enemy_telegraph`, `bot_prudent_dodge` et `bot_prudent_nododge`
divergent dès la ligne 1 (f0).

Preuve par trace détaillée d'`enemy_ring` (dump, binaires avant et après) :

- L'archer (`GgrsNetId(30)`) naît en (584, 632). Son collider couvre x 574–594 et y 616–636.
- Il est **incrusté de 2 px** dans deux murs :
  - `ldtk_wall_3x1` (616, 632 ; 48 × 16) : recouvrement de 2 × 12 px ;
  - `ldtk_wall_1x3` (600, 600 ; 16 × 48) : recouvrement de 2 × 8 px.
- **Avant** : il reste à (584, 632) pendant les 130 frames et ne tire qu'une volée (télégraphe à
  f2).
- **Après** :
  - à f0, un pas de dégagement le mène en (583,29 ; 632,71), ce qui fait diverger la trace dès la
    ligne 1 ;
  - dès f51, il marche vers le joueur (528,2 ; 687,8 à f128) ;
  - dans les scénarios de 600 frames, il tire une volée toutes les ~141 frames (télégraphes à f143,
    f284, f425, f566).
- La tourelle (472, 696) ne bouge ni avant ni après.

Attentes :

- `enemy_ring`, `enemy_ring_quad` et `enemy_telegraph` sont inchangées et vertes.
- `bot_prudent_nododge` est remesurée : santé à f599 de **44 → 9**. Le bot sans esquive encaisse
  les volées supplémentaires de l'archer.
- `bot_prudent_dodge` est inchangée : santé 100 à f599, un seul coup reçu (f188). L'esquive tient
  même face aux nouvelles volées. Seuls les commentaires ont été mis à jour.

**Constat (hors tâche, demandé par orch) :** le point de spawn de l'archer d'`arena_tir` est à
2 px d'un mur. Il faudra le corriger dans la carte plus tard, pas ici.

## Suite

`make test_scenarios` : seules les traces citées plus haut diffèrent. Les autres étapes sont vertes :

- tests des crates (hors scénarios) ;
- `make lint`, fmt, scripts interdits et enregistrement rollback ;
- `make gen` ×3 (aucun fichier modifié) ;
- `cargo check -p throne` ;
- exemples.
