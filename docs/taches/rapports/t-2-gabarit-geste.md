# T-2 — Gabarit d'arme à geste

Jalon servi : **S1, dimanche 15 novembre 2026, et suite de nuit** (`plan-1837-cli.md` §6).
Branche : `wquintal/t-2-gabarit-geste` ; départ après fusion de `origin/main` **`30f9d1e6`** dans le worktree de T-1.
Tête de code poussée et vérifiée : **`35723126`**.

## Périmètre et résultat

| Point du lot | Issue | Preuve |
|---|---|---|
| Visée conservée jusqu'au départ retardé | Fait | `scenario::generate::fire_gesture_pulses` pose `(128, 48)` sur tous les segments, appuyés ou relâchés, de la frame source 0 à la fin du scénario |
| Clics cadencés selon le geste | Fait | Une frame appuyée, puis relâchement ; période `max(2, raise_frames + fire_frames + 1)`, additions saturantes, dernier segment borné à `frames` |
| Borné aux armes à geste | Fait | `generate_game` transporte l'option `WeaponGesture` jusqu'à `build_weapon_scenario` ; seule la branche `Ranged` avec `Some` utilise le nouveau script ; `None` conserve `fire_pulses` tel quel |
| Régression neutre qui touche réellement | Fait | `testbed/arme_a_gestes` : test ajouté `frames: 200, min_hits: 2` ; attente `EntityHits` du RON généré exécutée et verte |
| Pas de changement aux autres générés | Fait | Régénération des cinq jeux ; empreintes SHA-256 avant/après de 866 fichiers RON/trace : seuls les deux fichiers `weapon_arme_a_gestes` changent |
| Commande externe demandée | Fait | `alacod test <1837-J-2> fusil_chasse --engine <ce-worktree>` vert, référence 1837 autorisée `8ab88ed`, worktree jetable ensuite supprimé |
| Contrôle rapide | Fait | `make check_rapide` vert sur `35723126` déjà poussé, journal SHA ci-dessous, sweep joué |
| Docs | Fait | Conventions §115, ce rapport et ligne LIVRÉ dans `docs/taches.md` |

Aucun crate SIM, système de `GgrsSchedule` ou type rollback modifié. Lecture de `WeaponGesture` comme configuration uniquement. Le constructeur public `build_weapon_scenario` reçoit désormais cette option en dernier paramètre ; ses appelants ont été migrés, dont le test d'arène dans `placement.rs`. Le constructeur `build_scenario` sans configuration d'arme garde son usage historique avec `None`.

Le générateur choisit directement le bon script : il ne construit pas une liste d'impulsions historiques pour l'allouer puis la remplacer. Les gabarits d'objets, larmes, familiers, champions et la mêlée restent sur leur chemin existant. Rechargement, cadence de tir et états de geste restent ceux de la simulation : aucune réimplémentation dans l'outillage.

## Mesure avant / après

Commande de diagnostic jouée avant puis après la correction (sortie dans un dossier temporaire propre à chaque passe) :

```sh
env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 APP_VERSION=v0.0.0 \
  cargo run -q -p scenario --bin alacod-gen --profile headless -- \
  games/testbed --play --no-trace --only arme_a_gestes --out <dossier-temporaire>
```

| Mesure sur `weapon_arme_a_gestes` | Avant | Après |
|---|---:|---:|
| Frames jouées | 200 | 200 |
| Impacts reçus par la cible | 0 | 2 |
| Attente minimale du contenu | absente | 2, satisfaite |

L'ancien scénario pouvait afficher « attentes ok » avec zéro impact car l'arme ne déclarait aucun `test:`. Le test ajouté au contenu neutre rend désormais cette régression visible au harnais de nuit.

L'arme du testbed a 9 frames d'épaulé et 18 de geste de tir : période de 28 frames, sources appuyées 10, 38, 66, 94, 122, 150 et 178 ; les intervalles relâchés gardent la même visée. L'ancien script laissait ces intervalles sans segment, donc avec `pan: (0, 0)` au départ retardé de certaines balles.

## Régénération et traces

**Changement voulu et autorisé par le coordinateur le 8 novembre** : ajout du minimum neutre et régénération de **la seule** référence `tests/scenarios/generated/testbed/weapon_arme_a_gestes.trace`. Mention explicite dans le message du commit `35723126`.

Procédure jouée, un seul Cargo à la fois et `ALACOD_MUTE=1` :

1. Empreintes SHA-256 des 866 fichiers `tests/scenarios/generated/*/*.{ron,trace}` existants avant les modifications.
2. `alacod-gen games/zombies`, puis `games/gungeon`, `games/isaac`, `games/throne` : régénération sans `--play` ni `--bless` ; RON inchangés.
3. `alacod-gen games/testbed --play --bless --only arme_a_gestes` : seule référence autorisée écrite.
4. **`make gen GAME=testbed`**, sans bless : **106 scénarios joués, attentes et traces vertes**, dont la référence nouvellement écrite.
5. Empreintes après : liste complète des différences limitée à :

```text
tests/scenarios/generated/testbed/weapon_arme_a_gestes.ron
tests/scenarios/generated/testbed/weapon_arme_a_gestes.trace
```

Journal complet : **`~/orca/runs/alpha/logs/t-2-gabarit-geste-gen.log`**. Les quatre autres jeux n'ont pas été rejoués en entier ; leurs contenus générés sont identiques, et seule la suite générée du testbed a été jouée/comparée. Aucun autre BLESS, aucun scénario manuscrit modifié. Les dossiers de diagnostic `/tmp/alacod-t2-baseline` et `/tmp/alacod-t2-fixed` ont été supprimés.

## Preuve 1837 isolée

Création initiale du worktree `~/orca/workspaces/1837/t-2-essai` sur `origin/main` `ad7be13`. Ce main déclarait encore `maps/arene_vide.ldtk` comme gabarit sans cible. Après question, le coordinateur a expressément autorisé **`origin/wquintal/j2` @ `8ab88ed`**, détaché dans ce même worktree : il fournit `maps/gabarit_armes.ldtk` et sa cible `counts_hits` au décalage du testbed.

Commande réelle (CLI construite auparavant en `--profile headless`, aucun Cargo parent pendant `alacod test`) :

```sh
target/headless/alacod test ~/orca/workspaces/1837/t-2-essai fusil_chasse \
  --engine ~/orca/workspaces/alacod/t-1-test-jeu-externe
```

Résultat : **code 0**, `weapon_fusil_chasse`, 200 frames, 2 coups, attente minimale 1 satisfaite ; trace ignorée comme prévu pour un généré par `alacod test`, 1 scénario joué et 0 échec.

Transcript : **`~/orca/runs/alpha/logs/t-2-gabarit-geste-1837.log`**. Aucun contenu existant du jeu modifié, aucun commit ou push dans 1837. Le seul fichier généré local a été supprimé, puis `git worktree remove` a retiré le worktree jetable **sans `--force`** ; le checkout du coordinateur reste intact.

## Contrôle rapide sur la tête poussée

```sh
env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 make check_rapide
```

Code **`35723126` poussé avant le contrôle**, sortie 0. Journal intégral : **`~/orca/runs/alpha/logs/t-2-gabarit-geste-check-rapide-35723126.log`**.

Dernières lignes :

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

✓ Contrôle rapide passé (suite de scénarios : la nuit)
```

Après le contrôle : `scripts/target-sweep.sh target` joué ; `target_wasm` absent. Profil `headless` uniquement pour les compilations du lot, aucun build debug, release ou wasm.

## Reste et limites

- Aucun point obligatoire de T-2 restant. Le correctif simulation de balle à direction nulle reste dans **P-1b**, hors périmètre : T-2 donne des inputs valides au runner sans masquer ou modifier ce bug.
- Suggestion conditionnelle reçue du coordinateur (« attentes 4/4 ou 3/4, seulement si une ligne ») : affichage T-1 conservé. Le runner ne publie pas un nombre d'attentes distinctes réussies ; les chaînes d'erreurs comprennent aussi les invariants. Un comptage exact partiel dépasse une ligne, et un chiffre déduit du nombre d'erreurs serait faux.
- Suite complète de l'engine laissée à la nuit conformément au brief ; ni P2P, ni rendu, ni navigateur dans ce lot.
