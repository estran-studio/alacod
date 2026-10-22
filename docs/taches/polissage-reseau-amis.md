# polissage-reseau-amis — online dans le navigateur par Allumette (S1-3)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 1, §5 décision 4),
`docs/contrat-lancement-web.md` (intégration, preuve du 2026-10-09, « Le catalogue online publié reste soumis à cette
recette »), `crates/game/src/jjrs/allumette.rs` (client natif, contrat du serveur l. 9-), `jjrs/browser.rs`,
`run_state.rs:338-350,507-513`, `website/static/loader.js`, `website/src/lib/components/UserMenu.svelte` et `website/vendor`
(SDK `@bascanada/allumette-browser`), `docs/taches/m0-v9-allumette-client.md`, `docs/taches/dettes.md` (D12, D94, D116),
`tools/web-smoke`. Branche `polissage-reseau-amis` depuis `origin/main`. Estimation : 2,5 j (b2 : service, D116 ; b1 ou
William : recette dans le navigateur). Clones : les quatre (code commun).

## Contexte

**Décision de William (2026-10-22) : le signaling passe par Allumette, pas de `matchbox_server` nu.** Allumette est un
dépôt à part (`../allumette`, **absent du poste VPS** : D94 « `ALLUMETTE_DIR` introuvable ») ; le site passe au jeu une
URL `wss://` exacte et un ticket, le flux HTTP du client natif n'est pas utilisé dans le navigateur (`jjrs/mod.rs:9-12`).
Deux Chromium sur le même hôte ont joué le 2026-10-09 ; **deux réseaux : non prouvé** (le Mac de recette passait par un
tunnel qui échouait même sans engine). Client Matchbox 0.15 (fork) ; serveur Allumette protocole 0.13.0 : interopérabilité
prouvée en 2026-10-09, **à revérifier**. `releases.json` annonce `online: true` malgré le contrat (« désactivé »).

## À faire

1. **Première étape, avant tout code : l'état réel.** (a) `wss://allumette.estran.studio` : résolution DNS, certificat,
   réponse du service, une session créée depuis le site (`/browser/v1/lobbies`, invitation, ready, start, `/session`) ;
   (b) **où vit Allumette** : dépôt (`bascanada/allumette` ? [INFERENCE : le SDK est publié sous `@bascanada`]), hôte,
   qui l'opère, comment on redéploie, version déployée ; (c) comment la session site → jeu se transmet
   (`HostedSession`, postMessage même origine, `loader.js`).
2. **Demander à William** (message court, sans attendre le reste) : accès au serveur Allumette (SSH ou panneau) et à son
   dépôt, secrets de signature des JWT/tickets si un redéploiement est nécessaire, TURN en service ou non, le choix d'un
   hôte si le service est à relancer, et la version de protocole Matchbox actuellement déployée côté serveur (0.13 attendue)
   contre le client 0.15.
3. Si le service est en panne : le remettre en service **lui-même** (pas un serveur de remplacement) avec William ;
   consigner l'URL, la procédure et l'opérateur dans `docs/jouer.md` et `contrat-lancement-web.md`.
4. **D116** : borner l'attente de `wait_for_players` quand des pairs en trop restent connectés ; test unitaire.
5. **Deux réseaux distincts** : deux navigateurs, zombies puis un autre clone, partie à 2, retour au salon
   (`ReturnToHostLobby`), seconde partie, refus d'un build incompatible.
6. `online` dans `releases.json` : vrai seulement pour les clones dont la recette est faite (§5 décision 4).

## Acceptation

- Partie à 2 entre deux réseaux depuis le site par Allumette, checksums/traces identiques, 0 `DESYNC` ; sinon dit.
- D116 refermée avec test ; `make test_scenarios` inchangé.
- Rapport `docs/taches/rapports/polissage-reseau-amis.md` : état réel d'Allumette, procédure, non vérifié (TURN forcé, 4 joueurs).
