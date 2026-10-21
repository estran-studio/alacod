# polissage-demarrage — écran de démarrage et textes en français (B3)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 6, §5 décisions 3 et 8),
`crates/game/src/ui/lobby.rs` (écran local et lobby réseau), `ui/game_over.rs`, `ui/screens/`, `jjrs/p2p.rs` (messages
d'état), `docs/conventions.md` §56 (écrans), `CLAUDE.md` (« Simulation et présentation »). Branche
`polissage-demarrage` depuis `origin/main`, **après le merge de B1** (API réseau). Estimation : 3 j. Clones : les quatre.
Machine : Mac avec affichage (captures de chaque écran).

## Contexte

Aujourd'hui salon, nombre de joueurs, nom et adresse viennent d'arguments de ligne de commande (inventaire §1, §6) ; les
textes mélangent français et anglais (« Nouvelle partie », « Lobby », « Rejouer » à côté de « DESYNC », « Connecting to
peer… », « back to the lobby », « Waiting for players », « Frame » : `lobby.rs:124-142`, `game_over.rs:120-125`).

## À faire

1. Écran de démarrage (présentation seule, `PresentationPlugin`, navigable clavier et manette comme `ui/screens`) :
   « Jouer seul », « Jouer à deux sur cette machine », « Héberger / Rejoindre en ligne » (nom de salon, nom de joueur,
   nombre de joueurs 2 à 4, décision 3), « Quitter ». Les arguments de ligne de commande restent prioritaires (scripts,
   tests, CI) : sans argument, le menu apparaît ; avec argument, comportement d'avant.
2. Choix d'un personnage seulement là où le jeu en a plusieurs avec un choix déjà codé ; **sinon rien** (pas de
   mécanique nouvelle).
3. Traduire tous les textes visibles : lobby, `Connecting to peer…`, tentatives, retour au lobby, `DESYNC` (avec une
   phrase : « la partie commune est perdue » et un bouton « Retour au menu »), fin de partie, pause. Centraliser dans un
   module de constantes (pas de chaîne éparse), sans système i18n.
4. Déconnexion d'un pair en cours de partie : message et retour au menu, jamais un écran figé ; vérifier avec
   `scripts/p2p-disconnect.sh`.
5. Retour au menu depuis la fin de partie et depuis la pause (`RunRequest::ToLobby` existe).
6. Retirer le script `.command`/`.bat` de B2 s'il existe.

## Acceptation

- De l'archive de B2, sans aucun argument, on atteint une partie locale puis une partie en ligne à deux, pour les quatre
  clones ; captures de chaque écran dans `docs/captures/polissage-demarrage/`.
- `grep` des textes anglais visibles listés ci-dessus : zéro occurrence restante.
- Aucune trace ne bouge (`make test_scenarios` inchangé : présentation seule) ; les scénarios `ui:` d'écrans restent verts.
- Rapport `docs/taches/rapports/polissage-demarrage.md`.
