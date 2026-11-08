# T-1 — Scénarios écrits et traces d'un jeu externe

Jalon servi : **S1, dimanche 15 novembre 2026, et suite de nuit** (`plan-1837-cli.md` §6).
Branche : `wquintal/t-1-test-jeu-externe`, base `a1cee150`.
Implémentation : `80be6330`, corrections de revue et tête de code vérifiée : **`3e8e2ffd`**.

## Livré et preuves

| Point du lot | Issue | Preuve |
|---|---|---|
| Jouer les écrits externes et leurs scripts multijoueurs | Fait | `alacod test` passe `--written` à `alacod-gen`, qui joue les `.ron` triés de `<jeu>/tests/scenarios` via `scenario::run`, exactement comme `crates/scenario/tests/scenarios.rs` |
| Attentes, invariants et fin atteinte | Fait | Même `ScenarioOutcome::failures` que le harnais ; preuve à deux sur 90 frames, `PlayerAlive` pour chaque handle, puis `PlayerDead` volontairement faux → code 1 |
| Trace sœur obligatoire et comparaison complète | Fait | Absente → code 1 ; `--bless` → écrite ; rejeu → code 0 ; référence altérée → code 1 |
| Bless protégé sur les écrits | Fait après revue | Attente fausse sous `--bless` → « non bénie », code 1 ; le test compare aussi la référence avant/après, inchangée |
| Générés inchangés | Fait | Inclusion sans filtre vérifiée en JSON : générés à `trace: "ignorée"`, écrit à `trace: "ok"` ; `alacod-gen` sans `--written` garde son régime précédent ; aucun changement du harnais de l'engine |
| Filtres | Fait | Id qualifié `scenarios/duo` et nom court `duo`, tous deux joués même quand l'arène du générateur n'a pas sa cible |
| Erreur d'un écrit isolée | Fait après revue | RON invalide puis fichier non UTF-8 avant `duo` : code 1, résultat faux avec diagnostic, `duo` toujours joué/comparé ; JSON `played: false` pour l'écrit illisible |
| Compteurs et verdicts | Fait | Ligne avec verdict des attentes / total et verdict de trace ; résumé des scénarios réellement joués, traces réellement comparées et échecs ; JSON `ok`, `played`, `trace_compared` |
| Génération en erreur | Fait | Sans filtre : erreur conservée, écrits tout de même joués ; test vérifiant aussi qu'un ancien généré n'est pas effacé |
| Documentation | Fait | Conventions §114 et section CLI antérieure corrigée, `docs/jouer.md`, D205, ligne LIVRÉ |
| Aucun changement de simulation ni trace engine | Fait | Seulement CLI, binaire d'outillage, test CLI, Makefile et docs ; aucun crate SIM, système GgrsSchedule, type rollback ou fichier `.trace` de l'engine modifié |

## Preuve de bout en bout

Commande jouée avec `CI` retiré, `CARGO_BUILD_JOBS=4`, `CARGO_INCREMENTAL=0`, `ALACOD_MUTE=1` :

```sh
env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1 make test_alacod_external
```

Le Makefile compile `content/tests/cli.rs` en `--profile headless --no-run`, récupère l'exécutable dans les messages JSON de Cargo puis le lance directement : aucun Cargo parent pendant les appels `alacod test` du test ignoré. Aucun chemin de harnais présumé (`deps/` n'est pas l'emplacement des exécutables de cette toolchain).

Dernière passe après revue :

```text
trace manquante : EXIT=1
trace blessée : EXIT=0
trace ok : EXIT=0
trace différente : EXIT=1
attentes ÉCHEC / 2, trace non bénie : EXIT=1
frame 60: PlayerDead { handle: 1, at_frame: 60 } : joueur vivant
test test_command_compares_written_external_two_player_scenarios ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 155.17s
```

Le même test vérifie l'inclusion sans filtre, le JSON, le rejet isolé des RON invalides/illisibles, les noms courts et la conservation des fichiers générés sur erreur. Les assertions ont effectivement été exécutées ; ce n'est pas une simple compilation.

## Transcript 1837, sans toucher au checkout du coordinateur

Aucune branche distante `origin/wquintal/j2*` présente au relevé. Sur instruction du coordinateur, création d'un **worktree détaché propre à T-1**, `~/orca/workspaces/1837/t-1-essai`, sur `origin/main` **`61563bf`**. Ajout local non commité de `tests/scenarios/t_1_duo.ron` : jeu `1837`, carte `maps/arene_vide.ldtk`, 90 frames, deux scripts de visée opposés et deux attentes `PlayerAlive` à la frame 60. Aucun contenu du jeu existant changé.

Transcript intégral : **`~/orca/runs/alpha/logs/t-1-test-jeu-externe-1837.log`**. Chaque appel utilisait `target/headless/alacod test <worktree-1837> --engine <ce-worktree-engine>`.

| Sélection / option | Résultat observé |
|---|---|
| `scenarios/t_1_duo`, sans référence | `attentes ok / 2, trace manquante` ; 1 scénario joué, 0 trace comparée, 1 échec ; code 1 |
| `scenarios/t_1_duo --bless` | `attentes ok / 2, trace blessée` ; code 0 |
| Sans filtre | Écrit vert et trace comparée ; **code 1** pour l'erreur du générateur d'armes préexistante, non masquée |
| `scenarios/t_1_duo`, rejeu | `attentes ok / 2, trace ok` ; **1 scénario joué, 1 trace comparée, 0 échec ; code 0** |

L'erreur globale observée concerne `brown_bess_fantassin` : cible `target` (`HitCount`) introuvable. Son diagnostic nomme `testbed/arena.ldtk` et `games/testbed/…/target.ron`. Elle empêchait initialement tout écrit de s'exécuter avant le filtre ; T-1 isole désormais les ids écrits et conserve l'échec du générateur en mode global. La correction de l'arène/cible/generate_template de 1837 appartient au dépôt du jeu, pas à ce lot engine.

Le worktree d'essai et ses seuls fichiers de preuve ont été supprimés avec `git worktree remove`, **sans `--force`**. Aucun commit ni push dans 1837 ; son checkout principal n'a pas été modifié.

## Contrôle rapide sur les têtes poussées

- `80be6330` : `make check_rapide` vert avant la revue.
- **`3e8e2ffd` : `make check_rapide` vert après toutes les corrections de revue**, tête poussée avant le contrôle, mêmes variables Mac que ci-dessus.
- Journal complet final : **`~/orca/runs/alpha/logs/t-1-test-jeu-externe-check-rapide-3e8e2ffd.log`**.
- Journal précédent : `~/orca/runs/alpha/logs/t-1-test-jeu-externe-check-rapide-80be6330.log`.
- Après chaque contrôle : `scripts/target-sweep.sh target` joué ; `target_wasm` absent. Aucun build debug, release ou wasm dans ce lot. Un seul Cargo actif à la fois.

Dernières lignes du contrôle final :

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

✓ Contrôle rapide passé (suite de scénarios : la nuit)
```

## Reste / limites

- **D205, seul reste de diagnostic** demandé en revue : pour un jeu externe sans `generate_template`, le message du générateur doit guider vers `generate_template` et conventions §28 plutôt que suggérer le fichier du testbed de l'engine. Il provient de `generate.rs` et remonte par deux chemins du binaire ; laissé explicitement en dette plutôt que changer davantage la génération dans T-1. Le code de sortie est déjà correct et cette erreur ne masque plus les écrits.
- La commande globale sur le contenu 1837 `61563bf` n'est **pas verte**, pour le prérequis de cible ci-dessus ; la commande filtrée d'un écrit l'est. Le coordinateur 1837 prend la correction de son contenu.
- Suite complète des traces engine laissée à la nuit, conformément au brief. Aucun BLESS de trace de l'engine, aucun test P2P, aucun changement de rendu, aucun essai navigateur dans ce lot.
