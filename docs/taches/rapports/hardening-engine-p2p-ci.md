# Durcissement P2P, CI et preuves de validation

Branche : `hardening-engine-p2p-ci`. Base : `b917c38f` (2026-10-14).
Demande de William : préparer des modifications pour la pérennité de l’engine, à revoir dans
sa session d’orchestration. Pas de merge sur main dans cette tâche.

## Changements à reviewer

1. **Détection P2P active.** Le builder commun natif/navigateur utilise `desync_interval` (10
   frames dans les arguments par défaut), refuse zéro et fixe explicitement les 60 Hz de l’engine.
   GGRS était resté sur `DesyncDetection::Off` dans ce chemin. Trois tests utilisent le builder
   de production avec deux pairs en mémoire : intervalle invalide, inputs retardés avec rollback
   sans desync, checksum divergent détecté. Aucune modification des inputs de jeu ni des types hashés.
2. **Destruction des balles ordonnée par NetId.** Remplacement du HashSet puis tri par
   `Entity.index()` par un BTreeMap indexé par `GgrsNetId.0`. Les Entity sont locales au client.
   Même politique de collision et `despawn_rollback()` ; aucune trace ne doit être bénie pour ce lot.
3. **CI complète.** `test_crates` couvre tout le workspace sans features par défaut, au profil
   headless, avec exclusion exacte de la suite `scenarios` jouée séparément. La CI utilise cette
   cible, teste les outils et le lint. L’ancien `cargo test` à la racine ne couvrait pas les tests
   unitaires de toutes les crates (dont ceux de game). `make gungeon` transmet aussi
   `DEV_FEATURES`, comme les autres jeux, pour éviter une seconde compilation de Bevy en local.
4. **Contrôle de déterminisme strict.** Imports groupés, collections Bevy, noms non qualifiés,
   RNG système et horloge sont détectés dans les crates concernées ; les nouveaux crates sont
   contrôlés par défaut. Exceptions exactes avec compte et raison, y compris les mesures de tests
   et les maps de chargement/présentation. Une exception périmée échoue aussi. Suppression d’un
   import HashSet inutilisé. Le contrôle est lexical, pas une preuve de déterminisme.
5. **Traces P2P vérifiées.** Le nightly supprime les anciens fichiers, impose des inputs neutres
   et exige 599 frames contiguës de chaque pair pour le smoke à 600 frames. Absence, fichier vide,
   trace tronquée ou divergence ne peut plus passer comme succès.
6. **Scénarios et métriques fiables.** Les identifiants des scénarios générés incluent le jeu
   (`generated/<jeu>/<nom>`). Les noms courts uniques restent acceptés. Six noms de fichiers
   étaient partagés par plusieurs jeux ; le BTreeMap des métriques pouvait les écraser. Le filtre
   refuse noms inconnus/ambigus, les partitions invalides échouent, le rendu actif ne transforme
   plus la suite en succès sans exécution. Les métriques shardées exigent toutes les parts du run,
   refusent les doublons et n’écrasent pas latest.json après échec. Le script transmet aussi
   `DYLD_LIBRARY_PATH` au binaire de test direct sur macOS.
7. **Performance explicite.** `make perf_review PERF_BASELINE=...` compare des mesures choisies,
   avec contrôle des frames/joueurs, refus des données absentes/invalides et seuil de régression
   (10 % par défaut). Objectif absolu optionnel, indépendant des budgets historiques. Aucun seuil
   historique relevé, aucune revendication de gain de vitesse.
8. **Documentation d’état et de sortie.** `docs/etat-engine.md`, README, ancien milestone,
   préambule du plan et consignes de validation : distinguer livré, accessible, testé et validé
   humainement. Suite proposée : D59, boucle M2, calibration/profilage, frontières des dépendances.

## Vérification

Toutes les commandes ci-dessous ont été exécutées localement ; aucune référence `.trace` réécrite.

- Trois tests P2P ciblés : verts (inputs variables, retards, rollback, divergence injectée).
- Treize tests Python des garde-fous et comparateurs : verts.
- Filtres inconnus/ambigus, slice hors bornes, shard malformé : quatre exécutions négatives
  du runner retournent 101. Script sharding avec zéro tranche : retour 2.
- Deux tests ciblés de sélection/partition : verts.
- `make format`, `make check_forbidden`, `make check_rollback_registration` : verts.
  Contrôle lexical : 36 occurrences historiques explicitement justifiées, aucune nouvelle implicite.
  `bash -n` sur les trois scripts shell touchés et `git diff --check` : verts.
- `ALACOD_DYLIB=0 scripts/test-scenarios-sharded.sh 4` : **255 scénarios verts**, sans bless,
  529 s au total, quatre tranches ; **255 métriques distinctes** après fusion. Aucun `.trace` changé.
- `ALACOD_DYLIB=0 make test_crates` : **728 tests réussis, zéro échec, 10 ignorés**
  (tests et doc-tests du workspace ; suite `scenarios` exclue exactement, jouée ci-dessus).
- `ALACOD_DYLIB=0 make lint` : **zéro erreur** dans les quatre jeux ; avertissements
  zombies 1, testbed 61, throne 3, gungeon 11 (aucune logique de lint ni aucun contenu changé).
- Deux scénarios `weapon_pistol` sélectionnés par ids complets : verts, traces inchangées.
- `make -n gungeon` avec `ALACOD_DYLIB=1` puis `0` : présence puis absence de `game/dev-dylib`, comme attendu.
- Environnement : macOS aarch64, Rust `1.101.0-nightly (0abfedbc7 2026-10-02)`, builds statiques
  (`ALACOD_DYLIB=0`). Suites sous charge : les FPS de ce run ne sont pas une référence de performance.

## Limites et reprise par l’orchestrateur

- Docker est installé mais son daemon ne répond pas sur cette machine : WebRTC/allumette et le
  restart P2P réel restent à exécuter. Les tests en mémoire ne valident pas le NAT ni le navigateur.
- Pas de build wasm (target non installé), pas de validation entre architectures ni de run humaine.
  Le build dylib n’a pas été reconstruit ; vérification statique des commandes et variables seulement.
- Les suites tournent sous charge ; leurs débits ne constituent pas une calibration. Refaire les
  benches au calme sur une machine nommée avant de fixer un objectif ou de conclure sur la fluidité.
- Aucun asset, contenu de jeu ou fichier `.trace` changé. D59, D61, D62, D60 et les autres tâches
  de contenu restent dans leurs fiches ; ne pas les considérer résolues par ce lot.
- La frontière ECS seule reste un chantier : ce lot documente les dépendances de présentation
  encore présentes, sans prétendre les avoir supprimées ni avoir validé un portage console.
- Les consommateurs externes de métriques doivent utiliser les nouveaux ids des scénarios générés ;
  refaire leur référence pour comparer ces scénarios. Les ids des scénarios manuscrits restent identiques.

## Dépendances : signal remonté au push

GitHub a signalé **58 alertes de dépendances sur la branche par défaut** (24 hautes,
24 modérées, 10 faibles) lors du push. Ce compteur est une notification de l’hébergement,
pas un audit réalisé dans cette tâche. Trier les [alertes Dependabot](https://github.com/estran-studio/alacod/security/dependabot)
par composant, exposition réelle et correctif disponible dans une tâche dédiée ; ne pas assimiler
ce compteur à 58 vulnérabilités exploitables de la simulation. Aucune dépendance mise à jour ici.

Pour la revue : comparer à `b917c38f`, lancer `ALACOD_DYLIB=0 make check` (ou les étapes séparées
avec les features habituelles du poste), puis la recette P2P à 2/4 et le restart. Ne pas fusionner
sur la seule base des tests en mémoire.
