# polissage-session-amis — validation en ligne de bout en bout (B9)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1, §4),
`docs/taches/rapports/m2-p2p-run-complete.md`, `docs/taches/rapports/m2-p2p-deux-machines.md`, `docs/jouer.md`,
`scripts/p2p-lib.sh`, `scripts/p2p-stress.sh`, `scripts/p2p-disconnect.sh`, `docs/taches/session-test-manuelle.md`.
Branche `polissage-session-amis` depuis `origin/main`, **après B1, B2 et B3** (l'essai se fait sur les archives de B2,
sans argument). Estimation : 2 j. Clones : les quatre. Machines : deux machines réelles (Mac + VPS x86-64 pour la
partie headless, Mac + Mac avec William pour la partie jouée) ; `b2` prépare, William joue.

## À faire

1. Par clone : archive de B2 → menu de B3 → salle commune → partie à **2** puis à **4** (décision 3) ; headless avec
   `p2p-stress.sh` (rollbacks réels, `NETEM` dégradé 120 ms / 20 ms / 1 %) et partie jouée avec William.
2. Déconnexion d'un pair en cours de partie (`p2p-disconnect.sh`) : message en français, retour au menu, l'autre joueur
   n'est pas bloqué.
3. Redémarrage de partie en ligne (`RunRequest::Restart`, D14) : une nouvelle partie démarre sans relancer les binaires.
4. Chaque `DESYNC` : graine, frame, trace complète (`ALACOD_STATE_TRACE_FULL=1`) ; ouvrir une dette (**D131+**).
5. Notes de William (30 min en coop par clone, décision 13) transformées en dettes, jamais en correction sans fiche.

## Acceptation

- Rapport `docs/taches/rapports/polissage-session-amis.md` : un tableau (clone × 2 / 4 joueurs × deux machines /
  réseau dégradé / déconnexion / restart) avec « fait / non vérifié » et la preuve (traces, durées) ; 0 `DESYNC` ou une
  dette ouverte par cas.
- Ligne de journal `LIVRÉ polissage-session-amis`.
