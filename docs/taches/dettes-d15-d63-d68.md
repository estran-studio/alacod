# dettes-d15-d63-d68 — retirer la grille de collision, contenu de test, soft-lock de réanimation

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, les lignes D15, D63 et D68 de
`docs/taches/dettes.md` (décisions de William du 2026-10-15), le rapport
`docs/taches/rapports/dettes-outillage-d15-d34-d60-d63.md` (mesures de D15, proposition de D63) et
`docs/taches/rapports/bot-explorateur-v2.md` (graine 90). Branche `dettes-d15-d63-d68` depuis
`origin/main`. Estimation : 1,5 j. Un commit par dette.

## À faire

1. **D15, retirer la grille spatiale de collision** (`crates/combat`).
   - `query_aabb` devient un balayage, résultats triés par `net_id` comme aujourd'hui.
   - Les traces ne doivent pas bouger : mêmes paires, même ordre. Si une trace bouge, c'est un défaut,
     pas un bless.
   - Mesurer `bench_bullets`, `bench_horde` et `bench_bullets_room` avant et après ; tableau au rapport.
   - Si un scénario devient nettement plus lent (plus de 10 %), le dire au rapport et s'arrêter là,
     sans retirer la grille.
2. **D63** :
   - `zombie_scaled` (zombies) marqué `test_only: true`.
   - L'analyse d'atteignabilité (`content::reach`) compte comme atteintes les armes citées par les
     règles (`Shoot`, `SpawnPattern`…) des ennemis atteints : `arsenal` et `arsenal_lourd` ne sont plus
     signalées.
   - Une fixture.
   - `alacod lint` des quatre jeux : liste des avertissements restants au rapport.
3. **D68, soft-lock de réanimation (graine 90)** :
   - Commande : `alacod-sim --game gungeon --bots 2 --profiles explorateur,explorateur --floors
     etages_deux --seeds 90..90 --until-floor 1 --log`.
   - Établir pourquoi le joueur à terre se déplace et pourquoi la réanimation s'interrompt.
   - Corriger, dans le bot ou dans la règle de jeu, avec la preuve §5 si une trace bouge.
   - Rejouer la campagne e2 (200 graines, 2 bots, `etages_deux`) : 0 soft-lock, et chiffres au rapport.

## Règles

Pas de `f32`, `order_iter!`, `BTreeMap`. `cargo fmt --all -- --check`.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1).

Pas de Docker : le p2p est rejoué par l'orchestrateur. Il est nécessaire si D15 ou D68 changent la
simulation ; le dire au rapport.

Campagnes :
- au premier plan ;
- par lots de 10 graines, poussés un par un.

Voie rapide du README §6 : fusionner `origin/main` et rejouer la suite juste avant le commit
`LIVRÉ dettes-d15-d63-d68`, avec le rapport `docs/taches/rapports/dettes-d15-d63-d68.md`.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
scripts, `make gen` ×4).
