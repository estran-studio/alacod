# polissage-textes-fr — textes en français dans le jeu (S1-4)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 6, §5), `crates/game/src/ui/lobby.rs`
(l. 40-233), `ui/game_over.rs` (l. 36-125), `ui/screens/`, `jjrs/p2p.rs` (messages d'état), `docs/jouer.md:64-69`,
`docs/conventions.md` §56, `CLAUDE.md` (« Simulation et présentation »). Branche `polissage-textes-fr` depuis `origin/main`.
Estimation : 1 j. Clones : les quatre. Machine : Mac (captures) ou VPS pour le code ; les captures se font sur Mac.

## Contexte

**Décision du 2026-10-21 : pas d'écran de démarrage dans le jeu** (le site connecte). Reste le mélange français/anglais
(inventaire §6) : « Nouvelle partie », « Lobby », « Rejouer » à côté de « DESYNC », « Connecting to peer… », « new
attempt (2/2) », « back to the lobby », « Waiting for players », « Frame » (`lobby.rs:124-142`, `game_over.rs:120-125`,
`docs/jouer.md:64-69`).

## À faire

1. Lister **tous** les textes visibles du moteur (lobby, états réseau, DESYNC, déconnexion, fin de partie, pause,
   inventaire, aide) par `grep` des littéraux de `ui/` et `jjrs/` ; centraliser dans un module de constantes (pas de système
   i18n).
2. Traduire. DESYNC : une phrase (« la partie commune est perdue ») et une action (« Retour au salon »,
   `ReturnToHostLobby`). Déconnexion d'un pair : message et retour au salon, jamais un écran figé.
3. Les écrans sont présentation seule ; les chaînes lues par les scénarios (`HudText`, `ScreenText`) : mettre à jour les
   attentes, **sans** changer une trace.
4. Textes propres à un clone (`hud.ron`, `names`) : relever ceux qui sont encore en anglais, les traduire.

## Acceptation

- `grep` des chaînes anglaises listées en tête : zéro occurrence dans les écrans ; captures de chaque écran réseau,
  DESYNC, fin de partie.
- `make test_scenarios` sans `BLESS` ; scénarios `ui:` verts.
- Rapport `docs/taches/rapports/polissage-textes-fr.md`.
