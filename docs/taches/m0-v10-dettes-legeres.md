# m0-v10 — dettes légères : CI allumette (reste de D12) + documentation checksum (D5)

## Contexte

m0-v9 (`2cb0897`) a apporté le client allumette natif (D12) : les clients headless savent se
connecter au vrai serveur de matchmaking (JWT, lobby, WebSocket), et l'orchestrateur a rejoué la
recette réelle allumette avec des traces identiques au chemin `--matchbox` (même sha256).
Reste de D12 : la CI de nuit (`scripts/nightly.sh`, STEP C) utilise encore `matchbox_server` nu
(service `signaling` de `docker-compose.ci.yaml`), et le commentaire d'en-tête du compose affirme
encore que « le jeu natif n'implémente pas encore » le flux allumette — devenu faux.

D5 : `HitCount` et `ai.stationary` sont hors checksum. La voie « documenter » a été choisie :
ce sont des champs de test seulement, sans valeur de gameplay. Il faut l'écrire dans
`docs/conventions.md` et fermer la ligne D5 de `docs/taches/dettes.md`.

## Périmètre

### A. CI allumette (reste de D12)

1. `docker-compose.ci.yaml` : corriger le commentaire d'en-tête (le client natif implémente le
   flux allumette depuis m0-v9). Dire aussi que `signaling` (matchbox) reste la recette
   README §4 et `allumette` celle de la CI de nuit.
2. `scripts/nightly.sh` STEP C : basculer du service `signaling` au profil `allumette`
   (`docker compose -f docker-compose.ci.yaml --profile allumette up -d allumette`, port 3537).
   Les clients passent de `--matchbox ws://127.0.0.1:3536` à `--allumette http://127.0.0.1:3537`.
   Le premier client crée le lobby, les suivants le rejoignent en décalé (recette m0-v9 :
   créateur d'abord, ~6 s d'écart) ; garder la comparaison des traces inchangée.
3. Si le profil `allumette` ne démarre pas en local (secret JWT, build, port), le corriger dans
   le compose (c'est dans le périmètre). Si `ALLUMETTE_DIR` n'est pas posé, nightly.sh cherche
   `../allumette` (layout meta-repo) ; introuvable → STEP C SKIPPED proprement (message + résumé),
   jamais un échec muet.

Hors périmètre : ne pas toucher au service `signaling` ni à la recette README §4 (le chemin
`--matchbox` reste supporté et doit rester rejouable tel quel). Ne pas toucher au code Rust du
client allumette.

### B. D5 — documentation checksum

Lire le code, confirmer les deux faits, puis les écrire :

- `HitCount` (`crates/game/src/character/health/mod.rs`, posé par `create.rs` seulement si
  `CharacterConfig::counts_hits`, faux par défaut — aucun personnage zombie/joueur ne le pose) :
  compteur posé par les scénarios/testbed pour leurs attentes (`crates/combat/src/weapons/
  expectations.rs`, `crates/scenario/tests/expectations.rs`). Hors rollback/checksum : le laisser
  hors checksum ne change aucune décision de jeu.
- `ai.stationary` (`crates/game/src/character/enemy/ai/state.rs` : exclu du `Hash` manuel,
  T2.9) : aucun contenu zombie ne pose `stationary` (testbed seulement) ; l'inclure décalerait
  toutes les traces sans changer aucun comportement.

L'écrire dans `docs/conventions.md`, près du §10 « Blesser une trace : la preuve » : un court
paragraphe « Exclusions assumées du checksum » (les deux champs, leur usage, la raison, les
références de code). Puis fermer la ligne D5 de `docs/taches/dettes.md` (statut
« fait dans `m0-v10-dettes-legeres` »).

ATTENTION : si la lecture du code montre que l'un des deux champs a une valeur de gameplay
réelle, s'arrêter sur ce point et livrer quand même le reste du lot, en signalant la découverte
dans le rapport — ne pas enregistrer de composant en rollback sans bénédiction.

## Critères d'acceptation

1. Le commentaire du compose est exact ; le profil `allumette` démarre en local
   (`docker compose -f docker-compose.ci.yaml --profile allumette up -d allumette`) et la recette
   deux clients (`--allumette http://127.0.0.1:3537`, lobby, 600 frames) produit deux traces
   identiques (`cmp`), comme en m0-v9.
2. `scripts/nightly.sh` STEP C passe par le profil allumette ; la comparaison des traces est
   conservée ; un `ALLUMETTE_DIR` absent/invalide donne un SKIPPED explicite, jamais un échec
   muet.
3. `docs/conventions.md` documente les deux exclusions avec références de code ; ligne D5 de
   `dettes.md` fermée.
4. Aucune ligne Rust modifiée : les traces ne peuvent pas changer. Pas de bless. Si du Rust doit
   quand même bouger (correction nécessaire au point 3 du périmètre A, par exemple), le signaler
   dans le rapport.
5. Vérification standard limitée (aucun code de simulation touché) : `make lint`, `make fmt`,
   `make scripts`, `make gen` (sans modification) ; la recette locale du point 1 tient lieu de
   p2p. L'orchestrateur rejouera la suite complète (62 scénarios, tests des crates, p2p) au
   LIVRÉ.

## Règles du worktree

- Le worktree `alacod_tasks/m0-v10-dettes-legeres/` est créé par l'orchestrateur : `source
  ../env.sh` avant toute compilation, `CARGO_BUILD_JOBS=4`, jamais depuis un target vide.
- Une seule compilation à la fois machine-wide : b1 valide m0-v7-p2 en ce moment — vérifier
  qu'aucun autre cargo ne tourne avant de lancer le tien (`pgrep -x cargo`), et libérer vite.
- `docs/conventions.md` est aussi modifié par la branche `m0-v7-phase2-bots-finisent-le-clone` :
  écrire la section D5 dans une zone distincte (§10), et merger `origin/main` dans la branche
  juste avant de livrer (règle d'hygiène) ; en cas de conflit, le résoudre en gardant les deux.
- Ne pas merger toi-même ; ne pas bénir de trace.

## Livrer

`LIVRÉ m0-v10-dettes-legeres <sha>` — avec le rapport
`docs/taches/rapports/m0-v10-dettes-legeres.md` sur la branche : ce qui a été fait (A et B), le
résultat de la recette locale (sha256 des deux traces), l'état de lint/fmt/scripts/gen, et la
base de la branche (doit être `origin/main` à jour).
