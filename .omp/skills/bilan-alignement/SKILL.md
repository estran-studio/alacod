---
name: bilan-alignement
description: Bilan pour William (stakeholder, architecte/designer) de tout ce qui a été développé depuis la dernière invocation — tâches accomplies, validations faites, alignement avec le plan et les objectifs finaux, points de désalignement et décisions à prendre. Invoquer par /skill:bilan-alignement ; met à jour le repère de dernière invocation.
---

# Bilan d'alignement

Tu rends compte à William, qui est le **stakeholder** (il paie, il décide) et l'**architecte/designer**
(le plan et les conventions sont les siens). Le but du bilan n'est pas de lister du travail : c'est de
vérifier que **tout le monde reste aligné** — l'orchestrateur, les workers et lui — sur les objectifs
du projet. Écris en français, concis, factuel ; chaque affirmation vient d'une source nommée (commit,
rapport, trace, journal). Aucun enthousiasme, aucune reformulation marketing.

## 1. Période

Le repère de la dernière invocation est `.omp/skills/bilan-alignement/dernier-bilan` (une ligne :
`<sha de main> <date ISO>`). Lis-le ; s'il n'existe pas, prends comme borne le premier commit du jour
précédent. Période = de ce sha (exclu) à `origin/main` (inclus). À la fin du bilan, **réécris le fichier**
avec le sha courant de `origin/main` et l'heure UTC, et commite-le (`bilan : repère <date>`).

Commandes de départ :

```bash
git fetch -q origin
git log --oneline --first-parent <sha-précédent>..origin/main          # merges et commits orch
git log --format='%h %s' --first-parent <sha-précédent>..origin/main | grep -i '^.\{8\} Merge'
git diff --stat <sha-précédent>..origin/main -- docs/taches.md docs/taches/dettes.md docs/conventions.md
```

Pour chaque merge, lis **le message de merge** (il contient les vérifications et le verdict de la
revue) et le **rapport** de la tâche (`docs/taches/rapports/<branche>.md`, sections « Preuves »,
« Traces bougées », « Dettes », « Questions »). Lis aussi les lignes de journal ajoutées à
`docs/taches.md` §10 et les dettes ouvertes/fermées dans `docs/taches/dettes.md`.

## 2. Sources des objectifs (ce à quoi on compare)

- `docs/plan-engine.md` : vision, architecture cible, jalons par clone, frontière simulation /
  présentation (et portage console).
- `docs/taches.md` : jalons M0…M5, critères de sortie d'un jalon (« Reste à faire pour sortir de M2 »,
  « Atteignabilité » de M3), réponses de William aux questions (tableau Q), règles du §1 (quatre voies).
- `docs/etat-engine.md` : contrat de sortie d'un jalon (p2p, rollbacks mesurés, nightly).
- `CLAUDE.md` : règles non négociables (déterminisme GGRS, traces, présentation dérivée de l'état).
- `docs/conventions.md` : les vocabulaires (un § par tâche) — ce que le moteur promet.

## 3. Structure du bilan (fixe, dans cet ordre)

### A. En une phrase
Où en est le jalon courant par rapport à sa sortie (ex. « M2 : il reste D86 et ta revue humaine ;
M3 : 5 tâches sur 18 restantes »). Donne les chiffres : merges sur la période, lignes, traces bougées
avec preuve, dettes ouvertes/fermées.

### B. Tâches accomplies
Tableau : tâche | ce que ça apporte **au joueur ou à l'engine** (pas au code) | worker/modèle | preuve
de validation (suite N scénarios, test_crates, p2p rollbacks réels, revue indépendante : verdicts et
nombre de passages) | traces bougées (jeu, cause §5). Une ligne par merge. Les correctifs d'infra
(scripts, outillage) à part, en une ligne.

### C. Validations : ce qui est prouvé, ce qui ne l'est pas
- Ce que **chaque** livraison a réellement exercé (suite, p2p avec frames resimulées > 0, fuzz,
  campagnes de bots, captures/vidéos, revue humaine).
- Ce qui n'est **que déclaré** par un worker (le reviewer le signale : « affirmations non prouvées »).
- Ce qui attend William : vidéos à regarder, sons à écouter, revue manuelle, décisions.
- Tout **désaccord entre un worker et le reviewer** et comment il a été tranché.

### D. Alignement avec les objectifs
Pour chaque axe, un verdict ✅ / ⚠️ / ❌ et une ligne de justification :
- **Déterminisme et rollback** (CLAUDE.md) : traces bougées sans preuve ? p2p réel ? desync ?
- **Data-driven** : la mécanique est-elle en données + behaviors génériques, ou spécifique à un jeu
  dans le moteur ?
- **Frontière simulation / présentation** : rien de visuel dans `GgrsSchedule`, présentation dérivée
  de l'état ? `check_sim_boundary` ?
- **Atteignabilité** (M3) : la mécanique est-elle rencontrée par une partie ordinaire (lint
  `MechanicUnmet`, graines `alacod-sim`, campagnes) ?
- **Critères de sortie du jalon** : lesquels ont avancé, lesquels régressent.
- **Décisions de William appliquées** (tableau Q, décisions datées) : lesquelles ont été appliquées
  telles quelles, lesquelles ont été interprétées (et comment), lesquelles attendent.
- **Règles d'orchestration** (quatre voies, revue avant merge, BLESS justifié) : respectées ?

### E. Désalignements et dérives
Ce qui s'éloigne du plan ou du design, même petit : choix pris par un worker ou l'orchestrateur sans
décision de William (les nommer, avec l'alternative), contournements (« préexistant » non prouvé,
budget relâché, trace rebénie sans cause), dettes qui s'accumulent sur un même sujet, écarts de
périmètre (fiche demandait X, livré Y), écarts de qualité visibles par le joueur (lisibilité,
équilibrage, feeling). Pour chacun : impact, et ce que tu proposes (garder / corriger / demander).

### F. Distance aux buts finaux
Rappelle les buts finaux (engine data-driven multi-clones, rollback p2p solide, console possible) et
dis **concrètement** ce que la période a rapproché : quelle(s) mécanique(s) de la liste du plan
existent désormais en données, quel clone est jouable de bout en bout par un bot / un humain / en
p2p, quels risques structurels restent (RNG, perf, dettes moteur).

### G. Décisions attendues de William
Liste numérotée, chaque item en une phrase avec l'option par défaut que l'orchestrateur prendra
s'il n'a pas de réponse. Rien d'autre : pas de questions rhétoriques.

### H. Prochaine période
Les 3 à 6 prochaines tâches, dans l'ordre, avec leur raison (chemin critique, prérequis, risque),
et ce qui est déjà en cours (worker, branche, durée).

## 4. Règles

- Longueur : 1 à 2 écrans. Si la période couvre plus de 10 merges, regroupe par thème dans B et
  garde le tableau en annexe courte.
- Pas de tableau de métriques en mermaid/svg : tableaux markdown seulement.
- Un fait non vérifié se dit `[INFERENCE]` ou « déclaré par le worker, non rejoué ».
- Si une source manque (rapport absent, preuve §5 absente), dis-le dans C : c'est un signal.
- N'invente jamais un état de validation. Quand tu ne sais pas si William a regardé une vidéo, pose
  la question dans G.
- Termine par la ligne : `Repère mis à jour : <sha> <date>` après avoir écrit et commité le fichier
  `dernier-bilan`.
