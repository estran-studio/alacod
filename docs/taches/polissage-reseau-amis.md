# polissage-reseau-amis — signaling joignable par défaut et salon par nom (B1)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 1, §5 décisions 1 et 11),
`docs/jouer.md` (§ « En ligne à deux »), `docs/jouer-a-deux.md`, `docs/taches/rapports/m2-p2p-deux-machines.md`,
`docs/taches/rapports/m2-p2p-run-complete.md`, `docs/taches/dettes.md` (D14, D94, D116), `crates/game/src/jjrs/`
(`p2p.rs`, `allumette.rs`), `Makefile:18-23`. Branche `polissage-reseau-amis` depuis `origin/main`. Estimation : 2 j.
Clones : les quatre (code commun). Machine : Mac avec Docker pour l'essai ; VPS pour héberger (voir 1).

## Contexte

`wss://allumette.estran.studio` (défaut `MATCHBOX_URL`, `Makefile:22`) échoue en ligne de commande (« socket dropped »)
et `--allumette` exige un salon créé par le site (`docs/jouer.md:36-62`) ; la recette qui marche demande Docker sur une
machine et son IP. Il faut un serveur joignable par défaut. STUN seul, pas de TURN (`jouer-a-deux.md:5-6`).

## À faire

1. Appliquer la décision 1 de William. Défaut : `matchbox_server` nu hébergé sur le VPS (service `systemd` ou conteneur,
   TLS/`wss://` devant), URL documentée dans `docs/jouer.md`. Si William choisit de réparer Allumette : n'écrire que
   l'adaptateur client nécessaire, sans toucher au service.
2. Faire de cette URL la valeur par défaut du binaire **et** du Makefile (`MATCHBOX_URL`), pour que `--matchbox` sans
   argument suffise ; retirer de `docs/jouer.md` et `jouer-a-deux.md` les mentions périmées (« ne marche pas »,
   `allumette.bascanada.org`).
3. Salon par nom : `--lobby <nom>` existe déjà ; vérifier qu'un salon de nom identique mais de jeux différents ne se
   mélange pas (préfixe jeu + version `APP_VERSION` dans le nom de salle) [INFERENCE : à vérifier dans `p2p.rs`].
4. D116 : borner l'attente de `wait_for_players` quand des pairs en trop restent connectés ; test unitaire.
5. Journal réseau lisible : les messages d'état (`Connecting to peer…`, `new attempt`) restent des constantes que B3
   traduit ; ne pas les traduire ici.
6. Essai : partie à deux **machines réelles** (Mac + VPS x86-64 pour le headless, ou deux Mac) par le serveur par défaut,
   avec `ALACOD_STATE_TRACE`, pour deux clones au moins ; noter si un NAT résidentiel empêche la connexion (décision 11).

## Acceptation

- `target/release/<clone> --matchbox --lobby salon --players localhost remote` sans URL joint le serveur par défaut depuis
  une autre machine que celle qui l'héberge ; traces identiques sur 600 frames.
- D116 refermée avec son test ; aucune trace de scénario ne bouge (réseau seul : `make test_scenarios` inchangé).
- Rapport `docs/taches/rapports/polissage-reseau-amis.md` avec la procédure, l'URL, ce qui n'a pas été vérifié.
