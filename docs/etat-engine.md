# État de l’engine et critères de sortie

Revue du 2026-10-14, base `b917c38f`. Cette page synthétise l’état connu ; les preuves détaillées
restent dans les rapports de tâches. Les tableaux de `plan-engine.md` décrivent principalement
l’écart initial du 2026-09-28 : ne pas les lire comme l’inventaire actuel.

## Avancement connu

| Partie | Livré | Validation disponible | Prochaine preuve requise |
|---|---|---|---|
| Socle | Fixed-point, rollback, RNG par flux, événements de frame, traces, invariants, lint, scénarios, bots | Suite de scénarios et campagnes documentées | Maintenir ces preuves sur chaque état fusionné |
| M0 / zombies | Boucle de vagues, achats, portes/fenêtres, perks, à terre/réanimation, restart | Campagnes de bots, traces P2P à 2 et 4 clients | Runs humaines sur plusieurs machines et validation du ressenti |
| M1 / throne | Trois cavernes, destruction, armes/munitions, mutations, portail, boss, HUD | Digest : 198/200 runs à 2 bots, 200/200 à 4, sans soft-lock ni desync sur la révision mesurée | Revue humaine finale, équilibrage, nouvelles campagnes après changements de contenu |
| M2 / gungeon | Salles et grammaires, patterns, blanks, esquives, effets, synergies, boss, props, formations, bot explorateur | Scénarios dédiés ; campagne explorateur sur une version antérieure du boss | Butin/coffres/boutique, contenu accessible, profil/hub/UI selon fiches, puis run intégrée P2P |
| M3–M6 | Plan et contrats partiels réutilisables | Pas de validation de clone complet | Terminer les critères de M2 avant d’étendre fortement le vocabulaire |

Les réussites de bots sont des résultats fonctionnels sur une configuration donnée ; elles ne
mesurent ni le plaisir, ni la difficulté pour un humain. « Zéro desync » en synctest n’équivaut
pas à une validation de traversée NAT ou de réseau dégradé.

## Contrat de sortie d’un jalon

Chaque clôture fournit un court dossier de preuve avec commit, jeu, commandes, graines, machines,
plateformes, options de build et limites. Un point non exécuté reste « non vérifié ».

1. **Boucle accessible.** Une partie lancée normalement depuis le manifeste rencontre les
   mécaniques annoncées. Les overrides de scénarios restent des tests complémentaires. Un objet
   seulement défini en RON ne compte pas comme contenu jouable. Le lint et les campagnes de bots
   couvrent atteignabilité, transitions et absence de soft-lock.
2. **Régression.** Tests du workspace, scénarios sans bless, lint et contrôles statiques passent.
   Un bless est lié à un changement voulu et à sa preuve (§10 des conventions). Le format historique
   d’un checksum peut évoluer avec une migration explicitement revue ; ne pas ajouter de conventions
   de hash complexes uniquement pour éviter un changement de référence légitime.
3. **P2P intégré.** Au moins une run complète à deux, puis quatre joueurs lorsque le clone les
   supporte, avec combat, choix/objets, boss, transitions et restart. Inclure deux machines distinctes.
   Garder des inputs actifs : le smoke de 600 frames à inputs neutres ne valide pas cette boucle.
4. **Réseau dégradé.** Mesurer et documenter les conditions, puis tester retard, jitter et pertes.
   Matrice de départ proposée : RTT 60/120/200 ms, jitter 0/20 ms, pertes 0/1/3 %, séparément puis
   quelques combinaisons représentatives. Mesurer stalls, profondeur/fréquence des rollbacks et
   récupération ; les conditions hors cible peuvent ralentir, mais ne doivent pas masquer un desync.
   Les tests en mémoire couvrent le protocole et la resimulation ; ils ne remplacent pas WebRTC/NAT.
5. **Plateformes.** Pour chaque combinaison annoncée (natif, navigateur, architectures différentes),
   comparer une même partie et ses états confirmés. Le fixed-point seul ne prouve pas la compatibilité
   des builds ni la stabilité des fonctions de hash entre plateformes/toolchains.
6. **Performance.** Sur une machine cible nommée et au calme, publier simulation, sauvegarde et
   restauration des snapshots, mémoire et coût des ticks avec resimulation. À 60 Hz, un tick dispose
   de 16,7 ms ; réserver une marge pour rejouer les frames prédites. Mesurer percentiles et pics sur
   horde, bullet hell, étage complet, explosions en chaîne et transitions. Les valeurs du harness
   headless/synctest sont des indicateurs de régression, pas une mesure directe du budget de production.
7. **Humain.** Jouer en coop, vérifier esquive/visée, lisibilité des dangers et du HUD, progression et
   redémarrage. Consigner les problèmes observés avant de déclarer le jalon terminé.

## Ordre recommandé après cette branche

1. Revue et intégration du durcissement P2P/CI ; exécuter le transport réel sur le poste
   d’orchestration (Docker opérationnel) puis entre machines.
2. D59 : choix explicite de la séquence d’étages dans le manifeste et grammaire par défaut de
   gungeon, avec transition en P2P. Garder la décision et la fiche existantes comme autorité.
3. M2-T5/T6/T14 : brancher les sources de butin et l’économie ; fermer D62, puis refaire une
   campagne sur le boss actuel et la boucle ordinaire. Terminer les autres livrables de M2.
4. Calibrer une référence de performance reproductible, profiler le coût dominant et corriger
   selon les mesures. Ne pas relever artificiellement les seuils de `tests/budgets.ron`.
5. Réduire progressivement les dépendances de présentation dans les modules de simulation :
   `combat` porte encore `animation`, les loaders RON et le type d’adresse Matchbox ; `map` et
   `utils` portent également des dépendances/features de présentation. Le headless actuel évite
   fenêtre/GPU à l’exécution ; il ne prouve pas une compilation ECS seule ni un portage console.
6. Faire assembler un petit jeu par une autre personne : une arme, un ennemi, un objet et une salle.
   Consigner le Rust nécessaire, les erreurs de contenu et les difficultés rencontrées.

## Outils de consolidation

- `make check` : format, tests des outils, garde-fous, workspace headless, scénarios, lint.
- `make test_tools` : tests rapides sans compilation de l’engine.
- Les scénarios générés ont pour id `generated/<jeu>/<nom>` : pas d’écrasement de métriques
  entre jeux. Les filtres inconnus/ambigus et les tranches invalides échouent explicitement.
  Le merge des métriques shardées refuse les parts absentes ou les collisions.
- `scripts/check-forbidden.sh` : strict par défaut, couvre aussi les imports groupés/Bevy et les
  nouveaux crates. Les exceptions sont des lignes exactes avec compte et justification dans
  `scripts/determinism-allowlist.json`. Réduire/retirer une entrée périmée ; ne pas régénérer cette
  liste pour faire passer une nouvelle violation. Ce contrôle lexical ne remplace pas une revue.
- `python3 scripts/check-p2p-traces.py --frames 599 client0.trace client1.trace` : traces complètes,
  contiguës et identiques d’un smoke démarré à froid avec `ALACOD_EXIT_AT_FRAME=600`.

Pour une comparaison de performance, copier la référence **avant** de relancer les mesures :

```bash
cp target/metrics/latest.json /chemin/persistant/reference-machine.json
# Même machine, options, instrumentation, scénarios et nombre de frames ; sans autres builds.
ALACOD_DYLIB=0 make test_scenarios SCENARIO=bench_bullets,bench_bullets_room,bench_horde,bench_cave,bench_floor
make perf_review PERF_BASELINE=/chemin/persistant/reference-machine.json
```

`perf_review` refuse mesures manquantes/non finies, charges de frames/joueurs différentes et
régressions de plus de 10 %. `PERF_MAX_REGRESSION`, `PERF_SCENARIOS`, `PERF_CURRENT` et
`PERF_MIN_SIM_FPS` permettent un contrat explicite. L’objectif absolu est optionnel et doit être
calibré ; les budgets historiques et la mesure d’un tick de production restent distincts.
Ne pas comparer des mesures issues d’une suite shardée à une suite séquentielle.
