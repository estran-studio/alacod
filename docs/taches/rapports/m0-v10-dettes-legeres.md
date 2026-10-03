# Rapport m0-v10 — dettes légères : CI allumette (reste de D12) + documentation checksum (D5)

**Base de la branche** : `origin/main` à jour au moment de la livraison (`2db0b23`, merge
d'hygiène fait juste avant — règle apprise en m0-v9). Aucune ligne Rust modifiée : les traces
ne peuvent pas changer ; aucun bless, aucun enregistrement rollback.

## A. CI allumette (reste de D12)

### `docker-compose.ci.yaml` — commentaire d'en-tête

L'ancien texte affirmait que le jeu natif « n'implémente pas encore » le flux allumette — devenu
faux depuis m0-v9. Le nouvel en-tête dit :

- `signaling` = matchbox_server nu : reste la recette p2p du README §4 (chemin `--matchbox`),
  intacte et rejouable telle quelle ;
- `allumette` (profil optionnel) = serveur de matchmaking (API HTTP /auth, /lobbies, JWT dans
  le chemin du WebSocket) : le client natif implémente ce flux depuis m0-v9 (`--allumette`), et
  `scripts/nightly.sh` STEP C passe par ce profil (port hôte 3537).

Service `signaling` et recette README §4 non touchés (hors périmètre respecté).

### `scripts/nightly.sh` STEP C — bascule sur le profil allumette

- Résolution d'`ALLUMETTE_DIR` : explicite (worktree/runner) sinon `../allumette` (layout
  meta-repo, défaut du compose). `ALLUMETTE_DIR/Dockerfile` introuvable → STEP C **SKIPPED
  propre** (message + ligne de résumé), jamais un échec muet. Même chose si docker/compose
  manquent ou si le démarrage du conteneur échoue (résumé `⚠ SKIPPED`).
- Démarrage : `docker compose -f docker-compose.ci.yaml --profile allumette up -d allumette`
  (log dans `allumette.log`), attente 3 s.
- Clients : `--allumette http://127.0.0.1:3537` (remplace `--matchbox ws://127.0.0.1:3536
  --lobby …`), `--number-player`, `--players`, `--cid`, `--name`. Le créateur (client 0) part
  d'abord, les rejoigneurs en décalé (~6 s) : la découverte/création simultanée est une course
  connue en v1 (deux créateurs simultanés créeraient deux lobbies), recette m0-v9.
- **Restart du conteneur entre deux comptes de joueurs** (`NIGHTLY_P2P=2,4`) : le serveur est à
  état en mémoire — sans restart, le lobby du run précédent repasse `Waiting` à la déconnexion
  et la découverte du run suivant rejoindrait un lobby éventé au lieu d'en créer un. Limite v1
  documentée en m0-v9, contournée ici.
- Comparaison des traces conservée à l'identique (`diff -q` client_0 vs les autres, `❌ DESYNC
  DETECTED` sinon) ; arrêt du serveur via `docker compose down` en fin de step.

### Correction du profil en local

Le profil `allumette` démarre en local sans modification : l'image `alacod-allumette:latest`
existe déjà (87,4 Mo) et le conteneur écoute `0.0.0.0:3536` mappé sur l'hôte 3537 — aucun secret
à poser (la clé JWT est générée au démarrage du serveur). Rien à corriger dans le compose.

## B. D5 — documentation des exclusions du checksum

Lecture du code : les deux faits de la fiche sont confirmés, aucun des deux champs n'a de valeur
de gameplay.

- **`HitCount`** (`crates/game/src/character/health/mod.rs`, T2.9) : compteur de coups reçus,
  posé à la création uniquement si `CharacterConfig::counts_hits`
  (`crates/game/src/character/create.rs`), faux par défaut — aucun personnage zombie/joueur ne
  le pose ; seul `target` de `games/testbed` le déclare (`crates/scenario/src/generate.rs`).
  Enregistré via `rollback_and_trace_no_checksum::<HitCount>()`
  (`crates/game/src/character/mod.rs`) : rollback et trace, mais hors checksum GGRS. Il sert aux
  attentes `EntityHits` des scénarios (`crates/combat/src/weapons/expectations.rs`) ; aucune
  décision de jeu ne le lit.
- **`EnemyAiConfig::stationary`** (`crates/game/src/character/enemy/ai/state.rs`, T2.9) :
  immobilité totale d'un ennemi de testbed (`dummy`/`target`/`ally`/`civilian`), exclue du
  `Hash` manuel du composant. Aucun contenu zombie ne la pose ; hacher un champ de plus
  déplacerait le checksum de toute entité `Enemy` sans aucun changement de gameplay (vérifié
  empiriquement à l'époque : sans l'impl manuel, `idle.ron` diverge dès f181).

Écrit dans `docs/conventions.md` §10 « Blesser une trace : la preuve », paragraphe
« Exclusions assumées du checksum » (zone distincte de celle de la branche m0-v7-p2, qui
touche d'autres sections). Ligne D5 de `docs/taches/dettes.md` fermée :
« fait dans `m0-v10-dettes-legeres` : documenté dans `docs/conventions.md` §10 ».

## Recette locale (critère d'acceptation 1)

Profil allumette démarré depuis le worktree avec `ALLUMETTE_DIR` pointant sur le clone
principal : `ALLUMETTE_DIR=/home/wq/Project/bascanada/alacod_root/allumette docker compose -f
docker-compose.ci.yaml --profile allumette up -d allumette` (conteneur `alacod-allumette-1`,
« listening on 0.0.0.0:3536 », port hôte 3537).

Deux clients 600 frames, créateur d'abord puis joiner ~6 s plus tard, lancés avec le **binaire
seed direct** (aucune compilation : b1 occupait le créneau machine-wide — aucune ligne Rust
modifiée, le binaire est à jour pour le gameplay) :

```
ALACOD_HEADLESS=1 ALACOD_STATE_TRACE=<trace> ALACOD_EXIT_AT_FRAME=600 \
  target/headless/zombies --allumette http://127.0.0.1:3537 \
  --number-player 2 --players "localhost remote" --cid client_N --name client_N
```

Flux observé dans les logs : lobby `9498132d…` créé → « 1/2 joueurs, attente » → « complet
(2/2) » → WebSocket avec le JWT dans le chemin → partie 600 frames → sortie propre (exit 0 des
deux côtés).

**Traces** : `cmp` → identiques ; 23 850 octets, 599 lignes chacune.

```
55ec099d391671440c0f5f848e57585afd590cb35cbec053ca91d8d419360b64  client_0.trace
55ec099d391671440c0f5f848e57585afd590cb35cbec053ca91d8d419360b64  client_1.trace
```

Même sha256 que la référence m0-v9 / m0-dettes (chemin allumette identique à `--matchbox`) :
les traces ne changent pas. Le conteneur a été arrêté (`docker compose down`) après le test.

## Vérification limitée

- `make format` (`cargo fmt --all -- --check`) : **0 diff**.
- `make check_forbidden` : 4 occurrences — les 4 avertissements **préexistants** des lots
  précédents, état inchangé.
- `make check_rollback_registration` : OK (aucun appel direct hors `crates/utils/src/rollback.rs`).
- `bash -n scripts/nightly.sh` : syntaxe OK.
- `make lint` : exit 0, aucune erreur de contenu (zombies et testbed) ; un avertissement rustc
  préexistant (`unused_variables` dans `crates/utils/src/web/mod.rs`, hors périmètre, code
  non modifié par ce lot).
- `make gen GAME=zombies` : exit 0, les 6 scénarios générés « ok ok » (attentes et traces),
  **sans modification** (comparaison avec les références, aucun bless).
