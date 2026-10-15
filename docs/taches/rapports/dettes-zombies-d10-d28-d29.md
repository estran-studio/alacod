# Rapport — dettes-zombies-d10-d28-d29 (cloud)

Branche de travail `claude/friendly-bardeen-hq2hxh` (alias de la tâche `dettes-zombies-d10-d28-d29`), partie
de `origin/main`. Trois commits, un par dette, chacun avec sa preuve §5 ; puis fusion de `origin/main`
(`e355be5`), re-bless des traces throne qui en dépendent, campagne de 200 graines.

## 1. Fait

**D29 — multiplicateurs par vague** (`a8e6808`). `wave_spawning_system` multiplie la vie max de l'ennemi
par `WaveState::current_health_multiplier` au spawn. Les dégâts passent par un composant
`combat::weapons::melee::WaveDamageScale(Fixed)` posé au spawn **seulement si** le multiplicateur diffère de
1 (vague 1 : aucun composant, aucun calcul, traces de la vague 1 inchangées) ; lu par
`enemy_melee_attack_system` (dégât de la hitbox de mêlée) et `enemy_attack_damage_translate_system`
(dégât direct). Rollback + checksum en variante **neutre** (`rollback_and_trace_neutral`, parité des
types vides). Valeurs de `wave_config.ron` inchangées (0,05 / 0,03). Les tirs à distance ne sont pas mis à
l'échelle (la fiche vise mêlée et attaque directe) : à décider si on le veut.

**D10 — gabarit de départ exclu** (`7f778da`). `get_next_room_recursize` filtre les gabarits
`LevelType::Spawn` des candidats (connexion fermée `DeadEnd` s'il n'en reste aucun). Décision écrite dans
`docs/conventions.md` §1 et `games/zombies/README.md` (trois salles au lieu de quatre avec la seed des
scénarios, un seul mur `pistol`, un seul Juggernog).

**D28 — dégât direct réparé** (`ca98dc6`). `enemy_attack_damage_translate_system` émet le `DamageEvent`
sans exiger un `DamageAccumulator`. Rééquilibrage en donnée : `attack_damage` des trois zombies
(`zombie`, `zombie_full`, `zombie_hard`) 10 → **1.0**.

Aussi : `dettes.md` (D10/D28/D29 marquées faites), `crates/scenario/tests/expectations.rs`
(`health_player_in_range` : minimum 50 → 30 à f800, le joueur immobile de `idle` est à 44 depuis D28).

## 2. Preuves §5 et blesses

- **D29** (dumps `main` vs branche, comparaison hors `WaveDamageScale`) : `idle` et `points_on_kill`
  identiques (1499 et 599 frames : ils n'atteignent jamais la vague 2 avec des ennemis vivants, leur trace ne
  bouge pas, contrairement à ce que la fiche attendait) ; `clone_solo` diverge à la frame 1070, seulement par
  la vie des zombies de la vague 2 (`MaxHealth` 50 → 52.5) puis ce qui en découle. Blessés : `clone_solo`,
  `clone_duo`, `clone_quad`, `avant_poste_demo`. Attentes `Currency` remesurées (+10) : `clone_solo`,
  `clone_quad`.
- **D10** (dump frame 0 de `idle`, `points_on_kill`) : 117 → 91 entités rollback ; retirées : la 2ᵉ salle
  `Level_0` (3 portes, 5 spawners, 3 fenêtres, 17 murs, pickup pistol, machine Juggernog), ids décalés
  (joueur 103 → 79), `FlowFieldCache` ; une porte perd son `Interactable` (elle menait à la salle
  disparue). 49 traces blessées (toutes les cartes `test_map` de zombies ; testbed et autres jeux
  inchangés). Scénarios adaptés : `window_repair` (fenêtre 28 → 21, joueur arrêté à x 454 et maintenu vers
  le bas, santé 1 à f470), `bots_four_mixed` (900 frames, 2 kills), `clone_quad` (vagues 2/3/4/5 à
  f871/f2360/f3617/f4674, 4674 frames, soldes 1550/1510/1410/3420).
- **D28** (dumps `idle`, `points_on_kill`, base contre correctif seul avec `attack_damage` 10) : première
  frame divergente 468 pour les deux ; un `DamageEvent` de 10 d'un `zombie_1` sur le joueur apparaît
  (`FrameEvents<DamageEvent>` était vide), `Health` 97.5 → 87.5, puis ce qui en découle. Blessés : 61
  traces dont **15 testbed et 9 throne** (leur `attack_damage` n'était presque jamais appliqué ; permis
  pour D28). Scénarios adaptés : `avant_poste_demo` (arrêt à f3400 : le joueur immobile meurt à f3465, après
  la victoire de f2236), `throne_duo_defaite` (le joueur 1 tombe à f625, le joueur 0 meurt à f732, défaite
  f731 ; attentes recalées).
- **Fusion `origin/main` `e355be5`** : 9 conflits, tous sur des `.trace` throne ; version de `main` prise,
  puis re-bless de 8 traces (6 `enemy_*` throne, `loot_floor_table`, `throne_duo_defaite`) ; cause : D28.
  `throne_quad` et `throne_three_floors` passent avec la trace de `main` telle quelle.

## 3. Mesure D28 (20 graines `alacod-sim`, Relais, 4 bots, jusqu'à la vague 5)

Binaires avant (D10) / après, mêmes graines 1..20.

| équipe | variante | objectifs | morts | dégâts moyens |
|---|---|---|---|---|
| 4 acheteurs | avant | 20/20 | 0 | 7,0 |
| 4 acheteurs | correctif seul (attack_damage 10) | 20/20 | 0 | 20,8 (un seul affrontement, graine 17 : 140 → 415) |
| fonceur,fonceur,prudent,immobile | avant | 20/20 | 28 | 314,7 |
| idem | correctif seul (10) | 19/20 (graine 20) | 38 | 461,1 (+47 %) |
| idem | attack_damage 2.5 | 19/20 (graine 12) | 31 | 356,8 (+13 %) |
| idem | **attack_damage 1.0 (retenu)** | **20/20** | 31 | 337,1 (+7 %) |

Les défaites des variantes 10 et 2.5 sont sur des graines différentes (le bruit d'une simulation chaotique à
20 graines n'est pas exclu) ; 1.0 garde le taux de victoire et reste proche des dégâts d'avant. Les valeurs
des autres jeux (testbed, gungeon, throne) ne sont pas rééquilibrées : leurs `attack_damage` sont désormais
appliqués tels quels, à relire par leurs auteurs.

## 4. Campagne 200 graines (état final, fusion avec `main`)

`alacod-sim --game zombies --bots 4 --seeds 1..200 --until-wave 5` (carte `le_relais_modules.ldtk`, 4
`acheteur`), en lots de 5 graines : `docs/taches/rapports/sim-dettes-zombies/final-*.json`.
**200/200 objectifs, 0 mort, 0 desync, 28,1 kills en moyenne, 6559 frames en moyenne ; 7 graines
seulement reçoivent des dégâts (moyenne 1,31)**. La difficulté est donc jouable, mais les `acheteur` sont si
forts que cette campagne ne mesure presque pas la montée de difficulté (D29) : voir les dettes.

## 5. Vérifié (résultats réels)

- Suite complète sur la fusion avec `main e355be5` (`make test_scenarios_par`, 4 tranches) : **310
  scénarios verts, 0 échec** (231 s), lancée après le dernier re-bless.
- `make test_crates` : 710 réussis, 1 échec corrigé ensuite (`health_player_in_range`, voir §1 ; ré-exécuté
  : 5/5 verts pour `health_*`). Le reste des crates est vert au dernier passage complet.
- `make lint` : « aucune erreur » pour les 4 jeux (avertissements préexistants). `cargo fmt --all -- --check`
  : rien. `check-rollback-registration.sh` : OK. `check-forbidden.sh` : 9 occurrences, identiques à celles de
  `origin/main` (aucune nouvelle).

## 6. Non fait / non vérifié

- **p2p à deux clients (Docker) et bench au calme** : hors de portée dans le cloud, à rejouer par
  l'orchestrateur. D29/D10/D28 touchent la simulation : le p2p zombies est à rejouer.
- Le `make test_crates` complet n'a pas été rejoué en entier après le correctif de `health_player_in_range`
  (seul `--test expectations -- health_` l'a été).
- Campagne en lots de 5 graines lancés hors `make sim` (même binaire `alacod-sim`, mêmes arguments ; `make
  sim` est séquentiel et trop lent ici, ~5 min la graine à 20 fps de synctest).

## 7. Dettes / questions

- Les tirs à distance ne reçoivent pas `WaveDamageScale` (fiche : mêlée et attaque directe). Un ennemi
  tireur de zombies n'existe pas aujourd'hui ; à brancher si on en ajoute.
- Les bots `acheteur` ne prennent presque jamais de dégâts : pour mesurer la difficulté montante, prévoir une
  campagne à vagues plus hautes (`--until-wave 10`) ou un profil plus faible.
- D28 a changé le jeu de testbed et throne (15 + 9 traces) sans rééquilibrage de leurs `attack_damage`.
- Atteignabilité : les multiplicateurs de vague sont rencontrés par toute partie ordinaire à partir de la
  vague 2 (`clone_solo`, graines `alacod-sim` du §4) ; le dégât direct par toute attaque de zombie
  (`idle` f468).
