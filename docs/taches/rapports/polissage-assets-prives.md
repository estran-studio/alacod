# Rapport : polissage-assets-prives (S-0b, voie b3, Mac)

Branche `polissage-assets-prives` (depuis `origin/polissage-integration` `a892c9a5`), dépôt privé `estran-studio/alacod-assets` (`main`, commit `d3d89c6`). Spec : `runs/alacod-polissage/b3-s-0b-assets-prives.md`
(y compris la mise à jour du coordinateur du 29 oct). Outillage seulement : aucun code de simulation, **aucune trace ne bouge**. Convention : `docs/conventions.md` § 99.

## Verdict

Fait : dépôt privé rempli et poussé, `scripts/assets-prives.sh` (clone/pull, copie idempotente, `--ci`), étape de CI dans `release.yaml` **et** `main.yaml`, drapeau `ALACOD_WEB_PRIVATE_ASSETS=1` (D161 fermée),
HUD de surcharge lu par le web sous le drapeau (D169, ouverte puis fermée dans le lot, accord du coordinateur), `assets.yaml`, § 99. Prouvé : builds web avec et sans drapeau, CI avec et sans secret (run GitHub), lint avec et sans surcharge.
**Non prouvé** : un run des vrais jobs `release.yaml` / `main.yaml` (runner auto-hébergé hors ligne), l'affichage dans Chrome d'un site publié avec la surcharge (aucun navigateur ni son sur le Mac pour ce lot).
**Retouche avant merge** : l'étape de CI ne tourne que si la variable de dépôt `ALACOD_WEB_PRIVATE_ASSETS` vaut `1` (non posée : repli public même avec le secret, nightly compris) ; commande d'activation dans le « Protocole pour William » ; la dette « hud.local.ron non lu sur le web » est **D169** (D167 et D168 sont celles de S2-8).

## Points de la spec

| Point | Issue | Preuve |
|---|---|---|
| Dépôt privé `alacod-assets` : `<jeu>/…` = chemins de la surcharge, `licences/`, `README` | fait | `estran-studio/alacod-assets@d3d89c6` : 178 fichiers, 1,1 Mo ; `zombies/` 79 (TheLazyStone 63 : `sprites/postapo/` 55 + `hud/` 8, `sprites.local.ron`, `ui/hud.local.ron` ; Sonniss 14), `throne/` 89 (Aymen B + Ittai) ; `licences/{thelazystone-postapocalypse,aymen-b-zombiepack,ittai-zombie-apocalypse,sonniss-gdc-2026}/` (`LICENCE.md` + texte lu dans l'archive ; PDF Sonniss + `pdftotext`) |
| Contenu identique à ce que les scripts écrivent | fait | zombies : `repack-postapocalypse.py` rejoué dans ce worktree, 63 fichiers, octet pour octet identiques aux 63 de b1 (`cmp`) ; throne : `repack-throne.py` de `origin/polissage-throne-visuel` rejoué dans un dossier jetable (`/tmp/throne-repack`), 89 fichiers identiques à ceux du worktree b1 ; Sonniss : les 14 `.ogg` copiés tels quels du worktree b1 (`polissage-zombies-sons`, sortie de `repack-zombies-sons.py`) |
| `scripts/assets-prives.sh` : clone ou pull, copie, idempotent, accès refusé = message + code non nul | fait | voir transcript A |
| Étape de CI avec `ALACOD_ASSETS_DEPLOY_KEY`, avant `make build_wasm_apps`, sautée sans secret | fait | `release.yaml` et `main.yaml` ; preuve par run : voir CI |
| `assets.yaml` : `statut: dépôt privé alacod-assets` pour chaque pack payant | **partiel** | l'entrée `thelazystone-postapocalypse` des deux jeux (seule sur la base) ; `aymen-b-zombiepack`, `ittai-zombie-apocalypse` (throne) et `sound-sonniss-gdc-2026` (zombies) n'existent que sur les branches de b1, non mergées : à faire au merge, texte plus bas |
| D161 : inclusion explicite de la surcharge | fait | transcript B ; D161 fermée dans `dettes.md` |
| Doc § 99 | fait | `docs/conventions.md` § 99 (et § 92 mis à jour) |
| Aucune image payante ni extrait Sonniss dans le dépôt public | fait | transcript C |
| D169 (demandée par le coordinateur) : HUD de surcharge lu sur le web sous le drapeau | fait, sans vérification visuelle | `hud::hud_overlay_wanted` + test ; compile wasm avec le drapeau |

### Transcript A : `scripts/assets-prives.sh`

```
$ scripts/assets-prives.sh                      # 1re exécution, surcharge supprimée avant
assets-prives : throne : 89 copiés, 0 déjà à jour.
assets-prives : zombies : 79 copiés, 0 déjà à jour.
assets-prives : alacod-assets @ d3d89c6 : 168 fichiers copiés, 0 déjà à jour.
$ scripts/assets-prives.sh                      # 2e : idempotent
assets-prives : alacod-assets @ d3d89c6 : 0 fichiers copiés, 168 déjà à jour.     rc=0
$ ALACOD_ASSETS_URL=https://github.com/estran-studio/inexistant-xyz.git scripts/assets-prives.sh     # pas d'accès
assets-prives : clone de … impossible : le dépôt est privé, il faut un accès (gh auth login, ou ALACOD_ASSETS_URL en SSH). Rien n'a été copié.     rc=1
$ scripts/assets-prives.sh --source <dossier avec zombies/sprites/public.png>
assets-prives : zombies/sprites/public.png n'est couvert par aucune règle de scripts/assets-prives.patterns : refus     rc=1
$ scripts/assets-prives.sh --source <dossier avec zombies/feedback.local.ron>
assets-prives : games/zombies/assets/feedback.local.ron n'est pas gitignoré (ou déjà suivi par git) : refus     rc=1
$ scripts/assets-prives.sh --ci   (sans clé)     → ::notice:: étape sautée, rc=0, rien écrit
$ ALACOD_ASSETS_DEPLOY_KEY="pas une clé" scripts/assets-prives.sh --ci   → clone impossible, rc=1 (le secret présent mais faux n'est jamais masqué)
```

Tests permanents (`make test_tools`, `scripts/tests/test_assets_prives.py`, 10 tests verts) : aucun fichier de la surcharge suivi sous `games/` ; chemins de la liste gitignorés ; script dans un dépôt jetable (copie idempotente,
refus d'un chemin non couvert, refus d'un chemin non gitignoré, source absente, `--ci` sans clé) ; `web-assets.mjs` avec surcharge sur le disque (sans drapeau, drapeau `true`, drapeau `1`).

### Transcript B : build web local avec et sans le drapeau (D161)

`make build_wasm_apps TARGET=web PROFILE=dev VERSION=proof-sans|proof-avec` (`WEB_OPTIMIZE=0`, `wasm-bindgen-cli 0.2.129` installé dans `/tmp/wbg`, `./target_wasm` du Makefile), surcharge présente sur le disque dans les deux cas :

| Build | `ALACOD_WEB_PRIVATE_ASSETS` | zombies : fichiers d'assets / dont surcharge | throne : idem | dossiers `postapo`, `sonniss` | lignes `sprites/postapo/` dans `wasm_bg.wasm` (zombies / throne) |
|---|---|---|---|---|---|
| `proof-sans` | absent | 93 / **0** | 101 / **0** | 0 | **0 / 0** |
| `proof-avec` | `1` | 172 / 79 | 190 / 89 | 2 (zombies), 1 (throne) | 70 / 105 |

Journal du build : `throne : surcharge privée exclue (89 fichiers présents sur le poste, non copiés).` / `… incluse (89 fichiers).` Le registre embarqué (`EMBEDDED_CONTENT`) contient `sprites.local.ron` et ses entrées seulement avec le drapeau
(`cargo:rerun-if-env-changed`). Les dossiers de test (`website/static/builds/proof-*`) et `releases.json` ont été remis à l'état du dépôt ; `./target_wasm` supprimé en fin de lot.

### Transcript C : rien de payant dans le dépôt public

```
$ git ls-files games | grep -c postapo                      0
$ git ls-files games | grep -cE 'sonniss|\.local\.ron'      0
$ git ls-files | grep -E '\.local\.ron$|sprites/postapo/|sounds/sonniss/'
crates/content/tests/fixtures/hud_local_overlay_missing_image/assets/ui/hud.local.ron      (fixtures de test du lint, deux fichiers de 2 lignes, sans image)
crates/content/tests/fixtures/sprite_sheet_local_overlay/assets/sprites/sprites.local.ron
```

## CI : avec et sans le secret

- **Run GitHub** (workflow dédié `assets-prives.yaml`, runner `ubuntu-latest`, push de la branche) : https://github.com/estran-studio/alacod/actions/runs/40916538335 — **success**, deux jobs.
  - `Étape de clone (secret du dépôt)` : `scripts/assets-prives.sh --ci` avec `ALACOD_ASSETS_DEPLOY_KEY` : `throne : 89 copiés`, `zombies : 79 copiés`, `alacod-assets @ d3d89c6 : 168 fichiers copiés`,
    `ALACOD_WEB_PRIVATE_ASSETS=1 posé pour les étapes suivantes` ; puis copie web : `zombies … surcharge privée incluse (79 fichiers)`, `throne … incluse (89 fichiers)`. La clé n'apparaît pas dans le journal (0 occurrence de `PRIVATE KEY`).
  - `Étape de clone sans le secret` (clé forcée vide) : `::notice::ALACOD_ASSETS_DEPLOY_KEY absent … surcharge privée sautée, le repli public est construit`, code 0, aucun fichier, drapeau absent ; avec 4 faux fichiers de surcharge sur le disque, la copie web sans drapeau en emporte **0** et avec drapeau 4.
- **Les vrais jobs ne sont pas prouvés.** `release.yaml` (tag) et `main.yaml` (Build & Deploy : `workflow_dispatch` + nuit, **le déploiement réel du site** : Cloudflare Pages et R2) tournent sur `[self-hosted, alacod-builder]` ; `gh api repos/estran-studio/alacod/actions/runners` à 2026-10-29T17:08Z :
  `vps-alacod-builder` **offline**. Aucun tag (consigne). Lancer `main.yaml` depuis ma branche aurait déployé en production : écarté.
  - `main.yaml` ajouté (accord du coordinateur) : le tag n'est que l'ancien chemin ; sans l'étape dans `main.yaml`, le site déployé la nuit resterait en repli public.

- **Retouche avant merge : garde par variable de dépôt.** L'étape « Assets privés » de `release.yaml` et `main.yaml` porte `if: vars.ALACOD_WEB_PRIVATE_ASSETS == '1'` **en plus** du secret ; la variable n'est **pas posée** (`gh variable list -R estran-studio/alacod` : vide, 2026-10-29), donc le nightly de `main.yaml`
  publierait le repli public au retour du runner, même avec le secret (pas de confirmation de TheLazyStone pour un build web public). Une étape « Assets privés désactivés » écrit un `::notice::` dans le journal à la place. Preuve (cinq lignes, run https://github.com/estran-studio/alacod/actions/runs/40922436192, job `Étape gardée par la variable de dépôt`, commit `a76e15d7`) :
  ```
  Assets privés (alacod-assets), même condition que le site    skipped      (condition : vars.ALACOD_WEB_PRIVATE_ASSETS == '1', secret ALACOD_ASSETS_DEPLOY_KEY présent dans l'env de l'étape)
  variable ALACOD_WEB_PRIVATE_ASSETS='' ; étape : skipped
  variable absente : étape sautée même avec le secret, rien copié, repli public.
  Résultat attendu                                             success      (aucun fichier de surcharge sur le disque, ALACOD_WEB_PRIVATE_ASSETS absent)
  run : success (3 jobs : gardée, sans secret, avec secret)
  ```
  Limite : ce job rejoue la **même condition** sur un runner GitHub ; les étapes des deux workflows du site, sur le runner auto-hébergé hors ligne, ne sont pas exécutées. `assets-prives.yaml` garde le secret seul pour ses deux autres jobs (clone avec et sans clé) : il ne déploie ni ne publie rien.

### Protocole pour William (vrai job, un geste)

0. **Activer l'étape** (aujourd'hui sautée) : `gh variable set ALACOD_WEB_PRIVATE_ASSETS --body 1 -R estran-studio/alacod` (Settings > Secrets and variables > Actions > Variables fait la même chose). **Ce que ça change** : à partir du run suivant de `main.yaml` (dispatch **ou nightly 00:30 UTC**) et de tout tag `v*`, l'étape clone `alacod-assets` avec le secret, copie les 168 fichiers, pose `ALACOD_WEB_PRIVATE_ASSETS=1`, et le build publie les images TheLazyStone, Aymen B et Ittai et les 14 extraits Sonniss sur le site public (Cloudflare Pages, R2, image ghcr). Pour défaire : `gh variable delete ALACOD_WEB_PRIVATE_ASSETS -R estran-studio/alacod` ; les builds déjà publiés restent à retirer à la main (les buildId sont immuables). Ne la poser qu'après la confirmation de TheLazyStone (point 5).
1. Remettre le runner `vps-alacod-builder` en ligne ; vérifier qu'il a un client `ssh` et sort en 22 vers github.com (`ssh -V`, `ssh -T git@github.com` avec n'importe quelle clé : « Permission denied (publickey) » suffit).
2. Après le merge de `polissage-assets-prives` dans `main` : Actions > **Build & Deploy** > Run workflow (déploie le site réel). Sans la variable, le journal montre « Assets privés désactivés (variable ALACOD_WEB_PRIVATE_ASSETS absente) » et le repli public est publié.
3. Avec la variable, dans le journal du job `Build WASM & Deploy` : l'étape « Assets privés (alacod-assets) » doit afficher `168 fichiers copiés` (puis `0 copiés, 168 déjà à jour` si le workspace n'a pas été nettoyé) et « ALACOD_WEB_PRIVATE_ASSETS=1 posé » ; l'étape « Build WASM in container » doit finir par `zombies : surcharge privée incluse (79 fichiers)`.
4. Sur le site : zombies en solo dans Chrome (muet) : sprites du pack, murs et sol (D169), icônes du HUD. Sans secret (run d'un fork) : l'étape écrit « étape sautée » et le repli public est publié.
5. Avant de publier un build public avec la surcharge : confirmation de TheLazyStone (plan-polissage § 0.3, décision 1) ; l'image Docker `ghcr.io/bascanada/alacod` de `release.yaml` contiendrait aussi les images.

**`ui/feedback.local.ron` de zombies (S1-9 étape 2, demande du coordinateur)** : ajouté à `alacod-assets` (`1beb8f5`, avec `death_player.ogg` mis à jour d'après b1 `522ebdc5`) ; le dépôt compte maintenant 169 fichiers (zombies 80, throne 89 : les « 168 » et « 79 » ci-dessus sont les chiffres du premier push `d3d89c6`). `assets-prives.patterns` n'a rien à changer (`*.local.ron` le couvre, donc le drapeau le suit au build web) et `.gitignore` l'ignore déjà (`ui/*.local.ron`) : `scripts/assets-prives.sh` l'a copié sans refus (« 2 copiés, 167 déjà à jour »). `throne/ui/feedback.local.ron` et `throne/sounds/sonniss/` (S2-11) suivront le même chemin sans changement d'outil.

## Lint, `make check`

- **Lint de contenu, avec et sans surcharge** (`alacod lint games/<jeu>`, surcharge présente puis supprimée) : zombies « aucune erreur (6 personnages, 4 armes, 6 armes de corps à corps, 1 vagues, 6 cartes) » et throne « aucune erreur (14 personnages, 13 armes, 4 armes de corps à corps, 0 vagues, 2 cartes) » dans les deux états ;
  la surcharge est ensuite restaurée par `scripts/assets-prives.sh` (168 copiés).
- Retouche : `make format check_forbidden lint` verts (rc 0, commit de retouche) ; seuls des commentaires de `hud.rs` (D167→D169) et des fichiers YAML/docs changent, pas de `make test`.
- `make test_tools` sur le Mac : 48 tests, **3 échecs préexistants** de `test_claude_say.py` (le faux `say` ; les mêmes trois échecs sur la base, `git stash`) ; les 10 nouveaux tests passent.
- Test unitaire D169 : `cargo test -p game --no-default-features --lib the_local_overlay` : 2 passed (`the_local_overlay_is_looked_up_on_the_web_only_in_a_private_assets_build`, `the_local_overlay_replaces_wall_art_and_merges_icons_by_id`). Compile wasm de `zombies` avec `ALACOD_WEB_PRIVATE_ASSETS=1` : `Finished`.
- **`make check`** (profil `headless`, `CARGO_TARGET_DIR=.target-mac`, `ALACOD_MUTE=1`, commit `6ef58211`) : `make -k format check_forbidden check_rollback_registration check_sim_boundary test_crates lint` → code 0 (fmt propre ; « OK: aucun nouveau motif interdit ; 51 occurrences auditées » ; « OK : aucun appel direct à rollback_component_with* » ; « OK : frontière simulation/présentation respectée par 13 crates ; 2 en dette D69 » ; tous les `test result: ok` de `test_crates` ; lint de zombies, testbed, throne, gungeon et isaac sans erreur) ; **`make test_tools` : 3 échecs préexistants de `test_claude_say.py` sur le Mac** (écart annoncé : le faux `say`), les 48 − 3 autres tests dont mes 10 passent ; le `make check` d'un seul tenant s'arrête donc à `test_tools` sur le Mac, d'où les cibles lancées une à une.
- **Scénarios, aucune trace ne bouge** : `make test_scenarios_par SHARDS=3` (la suite de `test_scenarios` en tranches, sans `BLESS`) : `tranche 0/3 : ok`, `1/3 : ok`, `2/3 : ok`, `métriques : 811 scénarios fusionnés`, `suite en 3 tranches : 761 s`, code 0 ; `git status` sans fichier `.trace` modifié ; la surcharge privée était présente sur le disque pendant la suite.

## À faire au merge (hors de ma base)

- `games/throne/assets/assets.yaml` : `aymen-b-zombiepack` et `ittai-zombie-apocalypse` : `statut: dépôt privé alacod-assets (throne/…, docs/conventions.md § 96 et § 99)` ; `games/zombies/assets/assets.yaml` : `sound-sonniss-gdc-2026` : `statut: dépôt privé alacod-assets (zombies/sounds/sonniss/, docs/conventions.md § 99)` (remplace « EN ATTENTE de la décision de William »).
  Les statuts de l'entrée `thelazystone-postapocalypse` des deux jeux touchent la dernière ligne de l'entrée : conflit trivial avec les entrées ajoutées juste après par `polissage-throne-visuel`.
- `.gitignore` : mes deux lignes `sounds/sonniss/` sont celles, à l'octet, de `polissage-zombies-sons` (fusion propre) ; leur commentaire « en attente de la décision de William » est périmé (décision du 29 oct : dépôt privé).
- `docs/conventions.md` : § 99 en fin de fichier comme les autres sections (§ 94, § 96, § 97, § 98, § 100 des autres voies : conflits de fin de fichier à prévoir).
- `feedback.local.ron` de b1 (S1-9 étape 2, § 98) : chemin non vu ici ; couvert par `*.local.ron` de `assets-prives.patterns`, mais `assets-prives.sh` le refusera tant que `.gitignore` ne l'ignore pas (garde-fou voulu) ; si le fichier est lu par le web, appliquer le même `option_env!("ALACOD_WEB_PRIVATE_ASSETS")` que `hud_overlay_wanted`.
- Le dépôt privé n'a pas encore de `throne/sounds/sonniss/` (S2-11 de b1, le script copie ce qui existe).

## Reste, avec estimation

| Reste | Estimation |
|---|---|
| Vrai run de `main.yaml` (protocole ci-dessus) et vérification dans Chrome du site avec la surcharge | 0,25 j, geste de William (runner en ligne) |
| `statut` des trois entrées de b1 dans `assets.yaml` | 5 min au merge |
| Date et montant d'achat d'Aymen B, date de téléchargement de Sonniss dans `licences/*/LICENCE.md` (non documentés dans les dépôts : rien d'inventé) | 5 min, William |
| Confirmation écrite de TheLazyStone avant tout build web public avec la surcharge | hors lot |

## Disque et nettoyage

`df -h /` : 69 Go libres avant (début du lot), 65 Go après les deux builds wasm (`target_wasm` 21 Go), 41 Go avec le `cargo check` wasm, 56 Go après `rm -rf target_wasm`, **33 Go en fin de lot** (le `.target-mac` partagé et les autres voies ont grossi pendant la suite de scénarios) ; builds natifs dans `~/orca/workspaces/alacod/.target-mac`, `target_wasm` absent. Aucun processus laissé. Aucun son (`ALACOD_MUTE=1`, aucun navigateur lancé).
