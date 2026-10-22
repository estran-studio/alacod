# polissage-session-amis — validation en ligne de bout en bout, dans le navigateur (S1-11)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1, §4), `docs/contrat-lancement-web.md`,
`docs/taches/rapports/m2-p2p-run-complete.md`, `docs/taches/rapports/m2-p2p-deux-machines.md`, `tools/web-smoke`,
`scripts/p2p-stress.sh`, `scripts/p2p-disconnect.sh`, `docs/taches/session-test-manuelle.md`. Branche
`polissage-session-amis` depuis `origin/main`, **après S1-1 à S1-4**. Estimation : 1 j (zombies, semaine 1) puis 1 j par clone
terminé. Machines : Mac + second réseau avec William ; VPS pour le headless.

## À faire

1. Zombies sur le site : solo, puis salon à 2 entre deux réseaux, **à la manette** ; partie jouée 15 min avec William.
2. Réseau dégradé (`NETEM` 120 ms / 20 ms / 1 %) en headless (`p2p-stress.sh`) ; déconnexion d'un pair (`p2p-disconnect.sh`
   et dans le navigateur) : message français, retour au salon, l'autre joueur n'est pas bloqué ; redémarrage d'une partie.
3. Chaque `DESYNC` : graine, frame, trace complète (`ALACOD_STATE_TRACE_FULL=1`) ; dette **D131+**.
4. Répéter pour chaque clone terminé (gungeon et isaac en semaine 2) ; notes de William transformées en dettes.

## Acceptation

- Rapport `docs/taches/rapports/polissage-session-amis.md` : tableau clone × 2 joueurs × réseaux × dégradé × déconnexion ×
  restart, « fait / non vérifié » et preuve ; 0 `DESYNC` ou dette ouverte.
- Ligne de journal `LIVRÉ polissage-session-amis`.
