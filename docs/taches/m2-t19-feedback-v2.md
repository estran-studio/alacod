# m2-t19-feedback-v2 — feedback v2 : télégraphes de patterns, flash du blank, fin de boss (M2-T19, V4)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Simulation et présentation »),
`docs/conventions.md` §7 et §31 (feedback v1 : présentation seule, `FeedbackLog`, preuve sans
écran), §20 et §41 (patterns, émetteurs, blank), §37 et §45 (boss, action `Telegraph`). Branche
`m2-t19-feedback-v2` depuis `origin/main`. Estimation : 2 j. Section de conventions réservée :
**§55**.

## Contexte

Le feedback v1 (§31) donne :
- le flash et le chiffre de dégâts sur un coup ;
- la secousse et le hit stop ;
- un télégraphe au sol en cercle (ruée, émetteur).

En `gungeon`, un pattern de boss (éventail, spirale) n'a pas de télégraphe lisible : le cercle
du v1 ne dit pas la forme du tir. Le blank (§41) efface les balles sans effet visible, et la mort
d'un boss passe sans marque.

## Décisions (l'agent confirme ou amende en dix lignes au rapport avant de coder)

1. **Tout reste présentation** : rien de rollback, rien dans le checksum ni dans les traces. Le
   « ralenti » de fin de boss **ne ralentit jamais la simulation** (p2p : ggrs avance sur
   `Time<Virtual>`, qu'on ne touche pas). C'est un gel de rendu long, comme le hit stop (§31,
   `AnimationFreeze`), avec zoom de caméra et flash plein écran, dérivé de l'état.
2. **Télégraphe de pattern** : pendant la phase `Telegraph` d'un émetteur (et le `Telegraph(n)` d'une
   frise de boss, §45), dessiner la **forme** du prochain pattern à partir de ses données :
   - les rayons d'un `Fan` (angle, nombre) ;
   - l'amorce d'une `Spiral` ;
   - le cône d'un `Burst`.
   Ce sont des gizmos, comme le v1. La forme se calcule dans une fonction pure, testable sans
   écran.
3. **Flash du blank** : dérivé de l'état. Quand les charges de blank d'un joueur baissent, un
   anneau qui s'étend sur le rayon du blank, plus un flash léger. Configurable dans
   `ui/feedback.ron` (champs optionnels, défauts sûrs).
4. **Fin de boss** : quand la salle d'un boss passe `Cleared` après la mort du boss (état, pas
   événement), gel de rendu de `boss_end.freeze_frames`, zoom sur la position du boss, flash. Le
   tout est configurable et désactivable.

## À faire

1. Les trois effets ci-dessus, avec leurs champs optionnels dans `FeedbackSettings` et le lint
   correspondant (frames > 0, rayon ≥ 0, zoom dans des bornes raisonnables) ; fixtures.
2. **Preuve sans écran** : chaque effet écrit son indice dans `FeedbackLog`. `scenario::events` en
   fait un moment clé `feedback` (« télégraphe fan 31 (5 rayons) », « blank 0 (rayon 160) »,
   « fin de boss 40 »). Attentes `Event(kind: "feedback", label_contains: …)` ajoutées à des
   scénarios existants (`boss_phase_shoot`, un scénario de blank de M2-T1, `gungeon_boss_bot`).
   Ces attentes ne touchent pas aux traces.
3. Contenu : `games/gungeon/assets/ui/feedback.ron` règle les trois effets.
4. Tests unitaires des fonctions pures (formes de télégraphe, rayon de l'anneau au fil des frames).

## Règles

**Aucune trace ne doit bouger** : tout changement de trace est un défaut de cette tâche, pas un
bless. Rien dans `GgrsSchedule`. Les crates de simulation n'importent rien de rendu.
`cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Sons (lot A de `docs/plan-assets.md`), particules, écran de mort, HUD (M2-T18).

## Session cloud

Tâche prévue pour une session Claude Code cloud (pas la machine `orca`) : ignorer les consignes
`orca` du README (§1 : `make sweep`, ports de signaling) ; build sans `game/dev-dylib` si
`./scripts/dev-features.sh` ne le propose pas. La session ne peut pas écrire à l'orchestrateur :
**pousser la branche régulièrement** (commits WIP acceptés ; le conteneur redémarre environ toutes
les deux heures et tue les processus d'arrière-plan : lancer la suite au premier plan) et finir par
un commit `LIVRÉ m2-t19-feedback-v2` qui contient le rapport. Pas de GPU dans le conteneur : la
preuve passe par `FeedbackLog` et les attentes `Event` ; dire au rapport quelles vidéos regarder
(`make videos SCENARIO=…`, frames), William les regardera sur un poste avec GPU.

## Vérification

README §4 (`make test_scenarios` complet sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`./scripts/check-forbidden.sh`, `./scripts/check-rollback-registration.sh`, `make gen` ×4) ; rapport
`docs/taches/rapports/m2-t19-feedback-v2.md`, conventions §55.
