# K5-4 — essai vocal : protocole, fiche à remplir, décision du pont MCP

Statut au 2026-10-22 : **guide et protocole écrits ; essai William à faire** (rien ci-dessous n'a été joué : toutes les cases de résultat
sont « non joué »). Guide : `docs/editeur-vocal.md` (§ 3 gungeon, isaac, throne ; § 4 bloc à coller ; § 6 voix ; § 8 dépannage).
Fiche : `docs/taches/k5-4-branchement-vocal.md`. Plan : `docs/taches/plan-editeur-vocal.md` § 3 (D5, D7).

## 1. Protocole de séance (30 à 45 min)

### Préparation, sur la machine de William

Prérequis : le lot **K5-1** mergé (commandes `restart`, `modifier`, `unmodify`, `reload`) ; **K5-2** pour la phrase 3 seulement (sinon « pas encore »).

```bash
cd <dépôt alacod>                         # à jour, branche de travail propre : git status vide
git checkout -b essai-vocal               # les commits « editor: » de la séance restent sur cette branche
cargo build -q -p zombies                 # une fois, avant la séance (la première compilation est longue)
make editor GAME=zombies                  # terminal A : fenêtre, rendu, clavier jouable, remote sur 127.0.0.1:15702
```

Terminal B (même dépôt, à la racine) :

```bash
scripts/alacod-remote brief               # 1re ligne : editor=True online=False check_distance=0 ; sinon relancer make editor
ALACOD_VOICE=1 ALACOD_VOICE_NAME=Thomas claude    # voix macOS ; sous Linux : ALACOD_VOICE=1 claude (espeak-ng installé)
```

À ouvrir : la fenêtre du jeu (terminal A visible pour le journal : `armes rechargées`, `Failed to load asset`), le terminal B de Claude. Avant de
commencer : `say -v '?' | grep fr_FR` (ou `espeak-ng -v fr "bonjour"`) pour vérifier la voix ; dictée système active dans le terminal B ; coller le
bloc de `docs/editeur-vocal.md` § 4 dans le `CLAUDE.md` du dépôt **ou** le dire en première phrase. Couper la dictée pendant la lecture (boucle voix).
Chronomètre ou horodatage à la main : on note le délai de la fin de la phrase à l'effet visible.

Hors séance (non nécessaire) : `alacod lint games/zombies` doit être vert avant de commencer ; à la fin, `git log --oneline -8` liste les commits `editor:`.

### Déroulé

Valeurs de départ de `zombies` à relire le jour même (`games/zombies/assets/…`) : elles peuvent avoir bougé depuis le 2026-10-22.

| # | William dit | Attendu | Effet à voir | Effet à entendre |
|---|---|---|---|---|
| 1 | « C'est un peu trop rapide. » | `alacod-remote modifier --handle 0 --source editor MoveSpeed Mul 0.9` (effet immédiat) **puis** édition de `movement.max_speed` du joueur (150 → 135, `ZombieShooter/Sprites/Character/player_config.ron`), lint vert, commit `editor: vitesse joueur 150 → 135` ; Claude dit que le RON ne change que les prochains spawns et que le modificateur s'applique au joueur vivant | le joueur va nettement 10 % moins vite tout de suite | une ou deux phrases courtes : quoi, où, quand |
| 2 | « Le pistolet fait un peu plus mal. » | `weapons.ron`, `pistol…bullet_type.Standard.damage` 15 → 17 (10 à 15 %), journal `armes rechargées`, lint vert, commit `editor:` | pas de changement visible à l'écran (chiffres de dégâts flottants : 17 au lieu de 15) | idem |
| 3 | « Spawne-moi un zombie_runner devant moi. » (**après K5-2** ; sinon Claude dit « pas encore » et on saute) | **Impossible avec le K5-1 livré :** `alacod-remote` n'a pas de sous-commande `spawn`. Cette phrase reste à jouer après K5-2 quand la commande sera livrée ; ne pas inventer une syntaxe avant sa fiche. | non joué | non joué |
| 4 | « Reset. » | `alacod-remote restart --kind Restart` (éventuellement `--seed N`) ; la partie repart ; les tables dérivées voient les RON modifiés au prochain `GameLoading` ; Claude dit que le modificateur de la phrase 1 est perdu mais que le RON reste | partie neuve, vague 1 | une phrase |
| 5 | « Annule ça. » | vérifier `git log -1 --format=%s` (commence par `editor:`) puis `git revert --no-edit HEAD` ; la valeur du pistolet revient à 15 ; le jeu recharge (`armes rechargées`) | dégâts flottants à 15 | une phrase ; Claude dit quelle valeur est revenue |
| 6 | **Ambiguë** : « C'est bizarre. » | Claude **pose une question** (quoi ? la vitesse, les dégâts, l'affichage ?) ; aucune édition, aucune commande | rien ne change | une question courte |
| 7 | **Refus** : relancer le jeu avec `ALACOD_CHECK_DISTANCE=2 make editor GAME=zombies`, puis « un peu moins vite » | `alacod-remote modifier --handle 0 --source editor MoveSpeed Mul 0.9` échoue ; Claude lit l'erreur et dit en une phrase que la cause est `check_distance=2` (relancer par `make editor` sans la variable) ; aucune édition silencieuse | rien ne change | la cause, dite |

Pour chaque ligne, noter (voir la fiche) : ce que William a dit mot pour mot (le texte de la dictée), ce que Claude a fait (commande ou fichier
édité), ce que William a vu, ce qu'il a entendu, le délai de la parole à l'effet, le verdict (**marche / marche mal / ne marche pas**), et toute
citation de William. Ne jamais compléter une case sans l'avoir observé.

Après la séance : `git checkout main` (ou la branche d'origine), jeter `essai-vocal` si on ne garde rien (`git branch -D essai-vocal` : décision
de William), `git status` vide.

## 2. Fiche à remplir

| Phrase | Dit par William (dictée) | Fait par Claude | Vu | Entendu | Délai | Verdict |
|---|---|---|---|---|---|---|
| 1. « C'est un peu trop rapide. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 2. « Le pistolet fait un peu plus mal. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 3. « Spawne-moi un zombie_runner devant moi. » (K5-2) | non joué | non joué | non joué | non joué | non joué | non joué |
| 4. « Reset. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 5. « Annule ça. » | non joué | non joué | non joué | non joué | non joué | non joué |
| 6. Ambiguë (« C'est bizarre. ») | non joué | non joué | non joué | non joué | non joué | non joué |
| 7. Refus (`ALACOD_CHECK_DISTANCE=2`) | non joué | non joué | non joué | non joué | non joué | non joué |

Citations de William : non jouées. Problèmes rencontrés et correctif ou dette (`D131` / `D132` réservées par le plan, à confirmer dans
`docs/taches/dettes.md` avant d'en ouvrir une) : non joués.

## 3. Décision du pont MCP

Statut : **décision en attente de la réponse de William.**

Trois questions sur son client vocal (Hermes ou autre) :

1. Le client peut-il exécuter un **shell** (lancer `scripts/alacod-remote`, `git`, `cargo`) ? Réponse : *en attente.*
2. Sinon, peut-il faire du **HTTP** local (`POST http://127.0.0.1:15702`) ? Réponse : *en attente.*
3. Sinon, parle-t-il **MCP** (ni shell ni HTTP) ? Réponse : *en attente.*

Critère (plan D5) :

- shell : **pas de pont**, fin ;
- pas de shell mais HTTP : **pas de pont** non plus ; le guide donne les appels JSON-RPC bruts (`rpc.discover` liste les méthodes `alacod/*`, un `POST`
  par méthode) et l'édition des RON passe par un outil de fichiers du client ;
- ni shell ni HTTP, mais MCP : pont nécessaire, **+1 j**. Deux options à chiffrer alors : (a) `bevy_brp_mcp` 0.23.0, après avoir **vérifié sa
  compatibilité avec bevy 0.19.1** (`bevy_remote` 0.19.1 est ce que le jeu embarque) ; (b) un petit pont maison d'environ 150 lignes avec `rmcp`
  3.6.0 (absent du `Cargo.lock`, `tokio` 1.53.1 présent), **hors du processus du jeu**, **sans Python** dans le dépôt. Versions à confirmer sur crates.io au moment d'implémenter (elles viennent du plan, non vérifiées ici).
- Un client sur **une autre machine** a de plus besoin du tunnel SSH du guide § 6 (`ssh -L 15702:127.0.0.1:15702 <machine>`).

Décision : en attente de la réponse de William. Par défaut, le guide est écrit pour « shell » (aucun pont) ; le cas « HTTP seul » est décrit en § 6.

## 4. Vérifications faites

1. **Chemins et champs du guide** (§ 3 gungeon, isaac, throne) : script jetable non commité (`/tmp/verif_k5_4.py`). Pour chaque ligne de tableau, il
   ouvre `games/<jeu>/assets/<fichier>`, cherche dans l'ordre chaque segment du champ (lignes de commentaire ignorées) et exige, sur la ligne du
   dernier segment, la valeur écrite. Résultat : **120 champs vérifiés / 120 écrits** (gungeon 42, isaac 39, throne 39). Le script détecte bien un
   écart (essai : valeur modifiée à la main, 3 lignes rouges, puis retour à 120 / 120).
   Les valeurs ont été lues dans les fichiers le 2026-10-22. Le binaire `alacod` n'était pas construit sur la machine : `alacod describe` n'a pas été
   lancé, les RON et `game.ron` ont été lus directement.
2. `python3 -m unittest scripts/tests/test_claude_say.py -v` : **5 tests, OK** (`test_disabled_is_silent`, `test_reads_last_assistant_and_cleans_markdown`,
   `test_say_ignores_unlisted_voice`, `test_say_uses_system_voice_when_name_absent`, `test_stop_hook_active_is_silent`).
3. Variables de `scripts/claude-say.sh` relues : `ALACOD_VOICE` (`1`, `stop`), `ALACOD_VOICE_NAME`, `ALACOD_VOICE_MAX` (600 par défaut), `ALACOD_VOICE_PIPER_MODEL`
   existent toutes. Constat consigné au guide : `ALACOD_VOICE=stop` ne fait que `pkill say` (macOS) ; sous Linux il ne coupe pas `espeak-ng`.

Non fait (à l'intégration ou en séance) : vérification 3 de la fiche (un agent sans contexte rejoue la boucle sur `make editor GAME=zombies HEADLESS=1` et
colle la trace ici) ; `alacod lint` de chaque jeu ; essai de William ; `docs/taches.md` §8.2 et journal §10 (réservés à l'intégration).

## 5. Points douteux

- Les valeurs de la phrase 1 et 2 (150 → 135, 15 → 17) sont celles de la fiche ; à relire dans les fichiers de `zombies` le jour de la séance.
- La syntaxe K5-1 est vérifiée dans `scripts/alacod-remote --help` : `modifier [--handle HANDLE|--net-id NET_ID] [--until FRAMES] [--source SOURCE] stat {Add,Pct,Mul,Set} value` ; `unmodify [--handle HANDLE|--net-id NET_ID] [--source SOURCE]` ; `restart [--kind {Restart,ToLobby,EnterRun,ToHub}] [--seed SEED]` ; `reload` (sans argument, code 1 si `errors` n'est pas vide). `input` accepte `[buttons]` facultatif ; **`input ""` relâche et rend la main au clavier** en mode éditeur. La fiche précédente indiquait `unmodify` sans ses options : corrigé.
- Le chemin `ZombieShooter/Sprites/Character/player_config.ron` vient de § 3 du guide (zombies), non retouché.
