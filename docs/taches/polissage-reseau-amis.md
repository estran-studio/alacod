# polissage-reseau-amis — online dans le navigateur : signaling `wss://`, deux réseaux (S1-3)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 1, §5 décisions 1 et 9),
`docs/contrat-lancement-web.md` (intégration, preuve du 2026-10-09, « Le catalogue online publié reste soumis à cette
recette »), `crates/game/src/jjrs/browser.rs`, `run_state.rs:338-350,507-513`, `website/static/loader.js`,
`docs/taches/dettes.md` (D116), `tools/web-smoke`. Branche `polissage-reseau-amis` depuis `origin/main`. Estimation :
2,5 j (b2 : serveur et D116, 1,5 j ; b1 : recette dans le navigateur, 1 j). Clones : les quatre (code commun).

## Contexte

Le site est le seul chemin de lancement : il fournit au jeu une URL `wss://` exacte et un ticket (le flux HTTP
Allumette est natif seulement, `jjrs/mod.rs:9-12`). Deux Chromium sur le même hôte ont joué en 2026-10-09 ; **la preuve sur
deux réseaux** est ouverte, et le Mac de recette passait par un tunnel qui échouait même sans engine. `releases.json`
annonce `online: true` pour zombies et throne alors que le contrat dit « désactivé jusqu'à la recette ». Version
Matchbox : client 0.15 (fork), serveur Allumette 0.13.0. L'état actuel d'`allumette.estran.studio` n'est pas vérifié.

## À faire

1. **Constater l'état** du signaling public (b2) : résolution, `wss://`, une session du site. Appliquer la décision 1 de
   William : par défaut, remettre Allumette en service pour le site ; sinon `matchbox_server` nu derrière TLS sur le VPS
   (alors adapter le flux du site : plus de ticket Allumette). Documenter l'URL.
2. **D116** : borner l'attente de `wait_for_players` quand des pairs en trop restent connectés ; test unitaire.
3. **Deux réseaux distincts** (b1 + un second réseau, William ou un ami) : deux navigateurs, deux clones au moins
   (zombies et un autre), partie à 2, `ALACOD_STATE_TRACE`/compteurs de rollback ; retour au salon (`ReturnToHostLobby`)
   et deuxième partie ; refus de build incompatible.
4. `online` dans `releases.json` : **vrai seulement** pour les clones dont la recette est faite (décision 9) ; mettre à
   jour le contrat.
5. Journal du lobby : les messages d'état restent des constantes que S1-4 traduit.

## Acceptation

- Partie à 2 entre deux réseaux distincts, depuis le site, traces ou checksums identiques, 0 `DESYNC` ; sinon dit.
- D116 refermée avec son test ; `make test_scenarios` inchangé (réseau seul).
- Rapport `docs/taches/rapports/polissage-reseau-amis.md` : procédure, URL, ce qui n'est pas vérifié (TURN forcé, 4 joueurs).
