# Rapport — m1-d26-doublons-generes (scénarios générés en double entre testbed et zombies, D26)

Branche partie de `origin/main` dcf8448.

## État en cours

Vérifié, livré. Suite complète verte (scénarios : aucune trace modifiée, seulement les six
suppressions ci-dessous ; crates, lint ×3, fmt, interdits, enregistrement rollback, `make gen` ×3
sans modification après coup, `cargo check -p throne`, exemples). Target purgé.

## Constat : la prémisse de la fiche ne tient pas

`games/testbed` n'hérite pas des armes de `games/zombies` par un registre fusionné : son manifeste
déclare **ses propres copies** (`ZombieShooter/Sprites/Character/weapons.ron`,
`weapons/melee/melee_weapons.ron`). Dix armes existent dans les deux jeux : `axe`, `bare_hands`,
`club`, `knife`, `sword`, `zombie_claws` (mêlée) et `machine_gun`, `pistol`, `rifle`, `shotgun`
(distance) ; leurs définitions sont identiques **hors commentaires et blocs `test:`** (vérifié par
diff) — `zombies` a les `test:`, le testbed non. `zombies` n'a pas de `generate_template` : ses
scénarios d'arme se jouent dans l'arène du testbed. Pour chaque arme, `generated/zombies/weapon_X`
couvre donc la même définition dans la même arène, **avec** des attentes ; `generated/testbed/
weapon_X` ne vérifiait que les invariants. Quatre paires étaient strictement identiques (`axe`,
`bare_hands`, `knife`, `zombie_claws`), six étaient des sous-ensembles (`club`, `sword` : même
trace, sans attente ; `machine_gun`, `pistol`, `rifle`, `shotgun` : 200 frames contre 600). La
règle 1 de la fiche (« seulement les définitions déclarées par le jeu lui-même ») ne supprimait
rien : chaque jeu déclare sa copie.

## Règle (décision d'orch : (a), seuil de 15 s)

Mesure des dix scénarios testbed en double (binaire des scénarios, un par un, sous la charge des
sims de l'orchestrateur) : mêlée 3,1 à 3,5 s chacun (≈ 19,5 s), distance 1,6 à 1,8 s (≈ 6,7 s),
**≈ 26 s** > 15 s : option (a).

- **Manifeste** : `ContentFolderDecl::generate` (défaut `true`, hors simulation) ; `generate:
  false` : `alacod-gen` ne génère aucun scénario pour les définitions de ce dossier (armes, armes de
  mêlée, personnages à `test:`), qui restent dans le registre et le jeu. Test
  `manifest::tests::parses_generate_flag`.
- **Générateur** (`generate.rs`) : saute toute définition dont le fichier source (`entry.file`) est
  sous un dossier `generate: false`.
- **Testbed** (`game.ron`) : `weapons/melee/melee_weapons.ron` déclaré `generate: false` (le fichier
  ne contient que les six armes de mêlée copiées de `zombies`).
- **Limite** : les quatre armes à distance partagées restent générées en double : elles vivent dans
  le même `weapons.ron` que les armes propres au testbed (`proj_*`, `grenade*`, `status_*`,
  `fireball_gun`, `foreuse`), et le jeu ne charge **qu'une table d'armes par jeu**
  (`game::global_asset` : le fichier de la première entrée du registre) ; les séparer demanderait un
  changement d'engine (plusieurs tables), hors périmètre. Coût restant ≈ 6,7 s.

## Scénarios supprimés (par `make gen GAME=testbed`, pas à la main)

| supprimé (`generated/testbed/`) | couvert par (`generated/zombies/`) |
|---|---|
| `weapon_axe` | `weapon_axe` (identique) |
| `weapon_bare_hands` | `weapon_bare_hands` (identique) |
| `weapon_club` | `weapon_club` (même trace, plus `EntityHits` 2..20) |
| `weapon_knife` | `weapon_knife` (identique) |
| `weapon_sword` | `weapon_sword` (même trace, plus `EntityHits` 3..20) |
| `weapon_zombie_claws` | `weapon_zombie_claws` (identique) |

Six `.ron` et six `.trace`. `make gen` ×3 rejoué ensuite : aucune modification.

## Temps de `make test_scenarios`

Même binaire, deux passes à la suite, machine chargée par les sims de l'orchestrateur (charge
moyenne 7 avant, 13 pendant l'« après ») : 1 307 s avant (188 scénarios), 1 511 s après (182) — les
totaux ne se comparent pas sous cette charge. Mesure fiable, dans la même passe « avant » : les six
scénarios supprimés y prenaient **17,0 s** sur 1 304 s de scénarios (≈ 1,3 %). Le « ~80 s » de la
fiche venait d'une estimation des dix doublons ; avec les quatre armes à distance restantes
(≈ 6,7 s), le doublon total était ≈ 24 s dans cette passe.

D27 : même défaut que D51, fermé par orch (rien ici).
