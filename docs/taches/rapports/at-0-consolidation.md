# Rapport at-0-consolidation : consolidation de la séance de réglage

Fiche : `docs/taches/plan-atelier-reglage.md` (§2 constats 3, 6, 7, 10, 13, 14 ; §6 AT-0 ; §7 ; §8 risque 6). Branche `at-0-consolidation`
depuis l'état de `essai-vocal` (base `5bdb201a`), fusionnée dans `main` en avance rapide. Changement de gameplay voulu : `89b2679b`
(« editor: mitraillette dispersion 0.15 -> 0.12 », réglage de William) ; ses 51 traces sont réécrites en **un** commit `BLESS`.

## Livré

`git log --oneline 5bdb201a..HEAD` (avant le commit de ce rapport) :

| Commit | Contenu |
|---|---|
| `89b2679b` | editor: mitraillette dispersion 0.15 -> 0.12 (réglage de William, gardé) |
| `b1aab036`, `d82d434c` | editor: `zombie_1` `movement.max_speed` 120 -> 40 et son revert (preuve d'agent, laissés dans l'historique) |
| `6c0c9a05` | SB-6 : arène vide générique et commande `alacod` seule |
| `b165bb89` | SB-7 : `alacod` outil unique de réglage et de test |
| `76787c56` | Sprite : les personnages posés à z = 0 remontent à la couche Entities |
| `595bc089` | plan-atelier-reglage : chantier K7, lots AT-0 à AT-9 |
| `e31fe047` | editeur-vocal : installation de `alacod`, règles des agents (jeu à part sur `ALACOD_REMOTE_PORT`, pas de `set --commit` chez William) |
| `73913bc6` | `alacod remote screenshot --wait` (5 s par défaut, code 1 au-delà) |
| `86b286c0` | `alacod set` relatif : `x0.9`, `*0.9`, `+10%`, `-10%`, `+=5`, `-=5` |
| `a998e6fe` | `alacod self-check` (binaire installé plus ancien que les sources : code 1) |
| `cb3f5d1e` | docs : preuves d'agent dans `CARGO_TARGET_DIR=target/agent`, objets et power-ups posés vérifiés visibles |
| `08119f9d` | fmt : `crates/content` |
| `d587af7d` | traces : BLESS de 51 scénarios zombies après `89b2679b` ; attentes de `clone_quad` et `la_fonderie_demo` réécrites ; risque 6 du plan |

## Preuves

- `make check` (parties) vert ; `cargo test -p content` : 413 passés ; `make schedule_order_diff` : identique.
- Suite complète, 809 scénarios sans `BLESS` : 758 identiques, 51 traces `zombies` déplacées, toutes par `89b2679b` (les bots tirent
  autrement avec la dispersion 0,12). `BLESS` des 51, puis les 51 rejoués sans `BLESS` : verts.
- Deux scénarios cassaient aussi leurs attentes ; réécrites sur le comportement mesuré (`ALACOD_EVENTS=1`), décision de William :

| Scénario | Attente | Avant | Après |
|---|---|---|---|
| `clone_quad` | `WaveAtLeast(3)` | f2360 | f2383 (vague 3 à f2383) |
| `clone_quad` | `WaveAtLeast(4)` | f3617 | f3623 (vague 4 à f3623) |
| `clone_quad` | `Currency` à f4674, joueurs 0/1/2 | 1550 / 1510 / 1410 | 1570 / 1380 / 1360 (joueur 3 inchangé : 3420) |
| `la_fonderie_demo` | victoire | f5 740 | f6 378 (vague 4, 29 kills) |
| `la_fonderie_demo` | `frames`, `RunState(Ended(Victory))`, `RunSummary` | 6200 | 6800 |

  L'attente M0 de `clone_quad` (vague 5, 13 kills, quatre joueurs debout à f4674) tient : vague 5 dès f4536. Commentaires « À regarder »
  mis à jour.

## Points ouverts

- `6c0c9a05` (SB-6) seul ne compile pas (il dépend de SB-7) : bissection à faire en sautant ce commit.
- `b1aab036` et son revert `d82d434c` restent dans l'historique (aucune réécriture) ; la règle §7 évite d'en ajouter.
- Objets et power-ups posés : visibles, aucune remontée à ajouter (pas de changement).
- La partie de William se ferme quand un autre build relie `target/debug/zombies` (macOS) : les agents compilent dans
  `CARGO_TARGET_DIR=target/agent`.
- `make test_tools` : `test_claude_say` échoue sous le bash 3.2 de macOS (environnement, sans lien avec AT-0).
- Risque 6 du plan : tout réglage `editor:` d'une arme ou d'un personnage de départ déplace des traces ; `BLESS` groupé en fin de séance,
  commande candidate `alacod bless` (AT-8).
