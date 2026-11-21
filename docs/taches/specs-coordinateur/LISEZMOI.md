# Specs du coordinateur alacod (plan `docs/taches/plan-1837-cli.md`) — copies pour la reprise sur une autre machine (21 nov 2026)

Copies de ce qui ne vivait que sur le VPS (`~/orca/runs/alpha/`). Les chemins absolus qu'elles citent (`/home/debian/…`, `~/orca/workspaces/alacod/<lot>`,
`~/orca/runs/alpha/logs/`) désignent le VPS : sur une autre machine, lire « worktree du lot », « dossier de journaux du run », « checkout de 1837 », « checkout de 1867_lore ».

- `commun.md` : consignes communes des workers (modèles, disque, jamais de profil debug, contrôle rapide seulement avant merge, suite complète la nuit).
- **Lots livrés** (rapports dans `docs/taches/rapports/`) : P-1d (`p-1d-parite-observee.md`, `p-1d-passation-2026-11-16.md`, `p-1d-reprise-vps-2026-11-19.md`),
  T-3b (`t-3b-suites-integration.md`, `t-3b-vps-2026-11-20.md`), M-7 (`m-7-harmonium.md`, suivi `m-7-suivi-vps.md`), M-7b (`m-7b-musique-vps.md`,
  revue `m-7b-revue-160bc9e2.md`), P-1e (`p-1e-physique-vps.md`, consigne `p-1e-consigne-mesure.md`, **tableaux** `p-1e-tableau.md` et `p-1e-tableau-avant.md`).
- **À poser dans 1837** (P-1e) : `p-1e-1837/physique_labo.{ron,trace}` → `tests/scenarios/` de 1837 (fait par le coordinateur 1837 s'il a réépinglé sur `8778a267`).
- **Lots arrêtés le 21 nov, non mergés** (reprise : la note `docs/taches/rapports/passation-<lot>.md` **sur la branche du lot**) :
  X-6d agrippeur `Grab` (`x-6d-agrippeur-vps.md` ; revue `revue-x-6d-consigne.md` → verdict `revue-x-6d-verdict.md` ; corrections demandées `x-6d-consigne-revue-2550feb4.md`),
  X-6e chef en deux phases (`x-6e-chef-vps.md`).
- `scripts/` : contrôle rapide **découpé** (`p-1d-check-rapide-decoupe.sh` appelle `p-1d-test-crates-decoupe.sh` ; à lancer depuis le worktree ; `LOGS`, `CARGO_TARGET_DIR`) :
  `make check_rapide` d'un bloc montait à ~47 Go de cible sur le VPS. Sur une machine qui a la place, `make check_rapide` suffit.
