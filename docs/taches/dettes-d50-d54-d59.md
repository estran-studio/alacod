# dettes-d50-d54-d59 — munitions de throne, tir ami, étage par défaut de gungeon (décisions du 2026-10-14)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §8 (tir ami), §29 (butin
throne), §40 (étages assemblés), et les dettes **D50, D54, D59** de `dettes.md`. Branche
`dettes-d50-d54-d59` depuis `main`. Estimation : 1 j. Un commit par dette.

## À faire

1. **D54, le tir ami `Always` exclut le tireur** (`bullet.source`, `bullet_rollback_collision_system`
   / `team_allows_hit`) ; `dash_stun_buffer` (qui s'étourdissait avec sa propre balle) redessiné
   avec un second tireur ; preuve §5 des traces qui bougent ; §8 à jour.
2. **D50, chance de butin de munitions de `throne` relevée** (`games/throne/assets/items/`, table
   §29 ; William a choisi une chance plus haute, pas une pondération par les réserves) : régler
   pour que la graine 76 à 2 bots ne tombe plus à sec, mesurer sur 20 graines témoins (`make sim`
   throne, 2 bots) avant/après ; traces `throne` rebénies avec preuve.
3. **D59, la grammaire devient l'étage par défaut de `gungeon`** : champ explicite du manifeste pour
   la séquence `Floors` par défaut (fin du piège « première séquence par ordre d'id »), `gungeon` démarre
   sur une séquence d'étages assemblés ; p2p 2 clients sur `gungeon` (étage assemblé et transition
   d'étage, traces identiques) ; D59 fermée. Prévenir l'orchestrateur si le bot explorateur
   (M2-T16, `m2-t16-bot-explorateur`) n'est pas encore mergé : ne pas toucher à ses scénarios.

## Règles

Pas de `f32`. `cargo fmt --all -- --check`. Vérification README §4 (`make test_scenarios_par` ou
`make test_scenarios`), rapport `docs/taches/rapports/dettes-d50-d54-d59.md`,
`LIVRÉ dettes-d50-d54-d59 <sha>`.
