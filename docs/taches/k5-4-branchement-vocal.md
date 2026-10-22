# k5-4-branchement-vocal — guide complété, essai des cinq phrases avec William, décision du pont MCP (K5, V3 + William)

Lire d'abord `docs/taches/README.md` (§1, §7), `docs/editeur-vocal.md` en entier (le livrable à compléter), `docs/taches/plan-editeur-vocal.md`
§3 (D5 : pas de pont MCP par défaut, D7 : on ne code pas le vocal) et §6 (risques), `scripts/claude-say.sh` (sortie vocale, branche `k5-0-voix`
fusionnée dans K5-0) et `docs/taches/rapports/k5-audit-hot-reload-assets.md`. Branche `k5-4-branchement-vocal` depuis `main` **après le merge de
`k5-1-partie-pilotable`** (K5-2 pour la phrase « spawne » seulement). Estimation : 1 j d'agent + une séance avec William (+ 1 j si pont MCP).
Cette tâche est surtout de la documentation et de l'essai ; **aucun code de simulation**.

## But

William parle (dictée du système, ou client vocal type Hermes qui héberge Claude) ; Claude modifie les RON et pilote la partie par
`scripts/alacod-remote` ; la réponse est lue à voix haute. Il faut que le guide soit assez précis pour qu'une instance de Claude sans contexte
le suive, et que William ait vu les cinq phrases marcher.

## À faire

1. **Compléter `docs/editeur-vocal.md`** (état à l'arrivée : sections 1 à 7, vocabulaire de `zombies` seulement) :
   - §3 « Vocabulaire → champs RON » : ajouter **gungeon, isaac, throne** (un tableau par jeu, mêmes colonnes : phrase, fichier, champ, valeur
     actuelle). Chaque chemin et chaque champ se **vérifient dans le fichier** (`games/<jeu>/assets/...`) avant d'être écrits ; commande utile :
     `cargo run -q -p content --bin alacod --profile headless -- describe games/<jeu> --json` (kinds et champs). Ne pas recopier la sortie
     entière : dire à Claude de l'appeler. Ajouter les unités (px/s, frames à 60 Hz, secondes) et un barème « un peu » = 10 à 15 %, « beaucoup » = ×2.
   - §4 « Règles pour Claude » : ajouter un **bloc prêt à coller** (`CLAUDE.md` du dépôt ou prompt système du client vocal), 20 lignes au plus :
     une modification à la fois ; lire avant d'écrire ; écrire sous `games/<jeu>/assets` seulement ; lint vert (`alacod lint games/<jeu>`) ; un commit
     `editor: <résumé>` ; « annule ça » = `git revert --no-edit HEAD` si HEAD commence par `editor:` ; dire en une phrase quoi/où/quand l'effet se voit
     (immédiat / au `restart`) ; réponses courtes (elles sont lues à voix haute : pas de tableau, pas de code, pas d'URL).
   - Nouvelle section **« Dépannage »** : le jeu ne répond pas (port 15702 : `curl -s -X POST -d '{"jsonrpc":"2.0","id":1,"method":"rpc.discover"}'
     http://127.0.0.1:15702`), commande refusée (`online` / `check_distance=N` : relancer avec `make editor`, sans `ALACOD_CHECK_DISTANCE`),
     RON invalide (journal `Failed to load asset … Could not parse RON` : l'ancienne valeur reste, corriger ou `git checkout -- <fichier>`),
     `reload` avec `errors` non vide (registre non remplacé), rien ne change (consulter §5 : « au `restart` » ?), lint rouge.
   - §6 « Voix » : relire `scripts/claude-say.sh` et vérifier que chaque variable décrite existe encore (`ALACOD_VOICE`, `ALACOD_VOICE_NAME`,
     `ALACOD_VOICE_MAX`, `ALACOD_VOICE_PIPER_MODEL`) ; ajouter l'installation d'une voix française (macOS : Réglages > Accessibilité > Contenu énoncé ;
     Linux : `espeak-ng`, voix `fr`).
2. **Protocole d'essai avec William** (séance de 30 à 45 min, jeu lancé par `make editor GAME=zombies` sur sa machine, un terminal avec
   `ALACOD_VOICE=1 claude` ouvert à la racine du dépôt) : lui faire dire, dans l'ordre, les **cinq phrases** (le plan § 1 en cite quatre ; la
   cinquième est choisie ici) et noter pour chacune dans le journal d'essai (étape 3) : ce qu'il a dit, ce que Claude a fait (commande ou fichier
   édité), ce qu'il a vu à l'écran, ce qu'il a entendu, le délai de la parole à l'effet, le verdict (marche / marche mal / ne marche pas).
   1. « C'est un peu trop rapide. » → attendu : `alacod-remote modifier --handle 0 MoveSpeed Mul 0.9` (effet immédiat) **puis** édition de
      `movement.max_speed` du joueur (150 → 135) + lint vert + commit `editor: vitesse joueur 150 → 135` ; Claude dit que le RON ne change que les
      prochains spawns et que le modificateur s'applique au joueur vivant (**K5-1** ; avec **K5-2** le RON s'applique aussi au vivant).
   2. « Le pistolet fait un peu plus mal. » → attendu : `weapons.ron`, `pistol…bullet_type.Standard.damage` 15 → 17 ; rechargement immédiat
      (journal `armes rechargées`) ; commit `editor:`.
   3. « Spawne-moi un zombie_runner devant moi. » (**après K5-2** ; sinon dire « pas encore » et sauter) → `alacod-remote spawn …` ; l'ennemi apparaît.
   4. « Reset. » → `alacod-remote restart` ; la partie repart ; les tables dérivées (vagues, objets) voient les RON modifiés.
   5. « Annule ça. » → `git revert --no-edit HEAD` après avoir vérifié que HEAD commence par `editor:` ; la valeur revient, le jeu la recharge.
   Aussi, une fois : une phrase **ambiguë** (« c'est bizarre ») : Claude doit demander quoi, pas deviner ; et un **refus** : relancer en
   `ALACOD_CHECK_DISTANCE=2` et vérifier que l'erreur nomme la cause (`check_distance=2`).
3. **Journal d'essai** : `docs/taches/rapports/k5-4-essai-vocal.md` (tableau des cinq phrases + ambiguë + refus, verdicts, citations de William,
   problèmes rencontrés et leur correctif ou dette). Un agent seul ne peut pas faire la séance : préparer le protocole, le déroulé, la fiche
   à remplir, et marquer « non joué » ce qui ne l'est pas ; **ne jamais inventer un résultat**. Si William est absent, livrer le guide et le protocole,
   statut « essai à faire ».
4. **Décision du pont MCP** (écrire une section « Décision » dans le journal d'essai, trois lignes + le critère appliqué). Questions à poser à William
   sur **son client vocal** (Hermes ou autre) :
   - le client peut-il exécuter un **shell** (lancer `scripts/alacod-remote`, `git`, `cargo`) ? → si oui : **pas de pont** (D5), fin.
   - sinon, peut-il faire du **HTTP** local (`POST http://127.0.0.1:15702`) ? → si oui : décrire dans le guide les appels JSON-RPC bruts
     (`rpc.discover` donne la liste, un exemple `curl` par méthode `alacod/*`), **pas de pont** non plus ; l'édition des RON reste à faire par un outil
     de fichiers du client.
   - sinon (ni shell ni HTTP, mais il parle MCP) : un pont MCP est nécessaire, **+1 j**. Deux options à chiffrer au moment de la décision :
     (a) `bevy_brp_mcp` 0.23.0 (annoncé par la recherche du 2026-10-22) : **vérifier sa compatibilité avec bevy 0.19.1** (`bevy_remote` 0.19.1 est ce que
     le jeu embarque) avant tout ; (b) un petit pont maison d'environ 150 lignes avec `rmcp` 3.6.0 (absent du `Cargo.lock`, `tokio` 1.53.1 présent) :
     `tools/list` = les méthodes de `rpc.discover`, `tools/call` = un `POST` JSON-RPC. **Hors du processus du jeu** (le pont doit survivre au jeu
     pour le relancer) et **sans Python** dans le dépôt (règle du plan § 7 ; `scripts/alacod-remote` reste en `urllib`). Les numéros de version
     viennent du plan : les confirmer sur crates.io au moment d'implémenter.
5. **Index** : `docs/taches/plan-editeur-vocal.md` (statut du lot K5-4), `docs/taches.md` §8.2 (une phrase) et la ligne de journal.

## Règles

- Documentation et essai seulement ; ne pas modifier `crates/`, les scénarios ni leurs traces. Si l'essai révèle un défaut du remote ou d'un
  lot précédent, **le noter** (rapport + dette `D131`/`D132` réservées par le plan, à confirmer dans `docs/taches/dettes.md` avant d'en ouvrir une)
  sans le corriger ici.
- Les valeurs chiffrées du guide viennent des fichiers (`games/<jeu>/assets`) lus le jour même, pas du plan ni de la mémoire.
- Réponses de Claude dans le guide : en français, courtes, lisibles à voix haute.
- Ne pas committer d'enregistrement audio ni de clés d'API du client vocal.

## Vérification

1. Tous les chemins et champs du guide : script de contrôle jetable (non commité) ou relecture, pour chaque ligne de tableau :
   `grep -n "<champ>" games/<jeu>/assets/<fichier>`. Les compter au rapport (n vérifiés / n écrits).
2. `scripts/tests/test_claude_say.py` passe (`python3 -m unittest scripts/tests/test_claude_say.py`).
3. Un agent sans contexte lit le guide et rejoue, sur `make editor GAME=zombies HEADLESS=1`, la boucle « lire l'état, modifier une valeur,
   voir le rechargement, committer `editor:`, `git revert` » : la commande de chaque étape est dans le guide, sans question. Coller la trace
   (commandes + extraits de journal) au journal d'essai.
4. `alacod lint games/zombies` (puis les autres jeux) reste vert ; `git status` propre après les `git checkout --` des valeurs d'essai.

## Livraison

Commit `k5-4 : guide vocal complet, protocole d'essai, décision du pont MCP` ; rapport = `docs/taches/rapports/k5-4-essai-vocal.md` ;
journal `docs/taches.md` §10 ; `LIVRÉ k5-4-branchement-vocal <sha>` (avec « essai William : fait / à faire »). Jamais de push.

## Pièges

- **Le remote ne répond qu'à `127.0.0.1`** (`http.rs:103-131` de `bevy_remote`, `with_address` possible) : un client vocal sur une autre machine
  (Hermes distant) a besoin d'un tunnel SSH (`ssh -L 15702:127.0.0.1:15702 <machine>`) ; l'écrire au guide, **ne pas** ouvrir le port.
- **Dictée du terminal** : la dictée macOS transcrit sans ponctuation ni casse ; les noms d'ennemis (`zombie_runner`) se disent mal. Prévoir au
  guide que Claude **liste** les ids (`alacod describe`) et propose le plus proche plutôt que de deviner.
- **Boucle voix** : la sortie vocale lit la dernière réponse ; si le micro est ouvert, il se transcrit lui-même. Dire à William de couper la
  dictée pendant la lecture (`ALACOD_VOICE=stop scripts/claude-say.sh` coupe la voix en cours).
- **Hook de lint asynchrone** (`.claude/settings.json`, 300 s) : il rend la main avant la fin ; Claude doit attendre le verdict (ou lancer
  `alacod lint` lui-même) avant de dire « fait ».
- **`git revert` après un commit `editor:`** : le fichier est restauré, le file watcher recharge ; mais une modification faite **par
  `alacod-remote modifier`** (modificateur vivant) n'est pas un commit : « annule ça » doit alors faire `alacod-remote unmodify`.
  Le guide doit distinguer les deux sortes de changement.
- **Après un `restart`**, `Modifiers` repart de zéro (nouveau personnage) : un réglage voulu durable passe par le RON.
- Ce qui reste hors périmètre : p2p, web, reconnaissance et synthèse vocales (client), Music Tester (Harmonium), rollback vérifié en mode éditeur.
