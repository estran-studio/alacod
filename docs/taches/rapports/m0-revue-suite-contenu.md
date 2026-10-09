# m0-revue-suite-contenu — préparation S5

Session de William, 2026-10-09 ; base `ae87ae0`, branche `m0-revue-suite-contenu`.

- §0 : branche existante conservée, `git fetch origin` effectué ; compilation de `zombies`
  avec rendu et profil `headless` réussie (4 min 39 s, compilation groupée avec `alacod-sim`).
- §1 : prompt, `CLAUDE.md`, README des tâches, carnet, dettes et conventions utiles lus.
- Instrumentation : `alacod-sim` rapporte, hors simulation dans `Last`, la préparation,
  le début des apparitions, la fin, les baisses de santé, les mises à terre et les morts
  par vague. Une frame déjà observée est ignorée.
- Compilation séparée d'`alacod-sim` sans la feature de rendu des tilemaps : réussie,
  59,14 s. Test du binaire : 1 réussi, 0 échec.
- `cargo fmt --all -- --check` : réussi ; contrôles forbidden : 4 avertissements
  préexistants ; contrôle rollback registration : OK.
- Aucun réglage de gameplay appliqué, aucune trace bénie, aucune intervention dans M2.
- D29 confirmé : les multiplicateurs de santé **et** de dégâts des vagues sont calculés
  et tracés, mais ne sont pas lus pour appliquer la santé ou les attaques des zombies.
- Mesure avant S5 : en cours, quatre acheteurs, graines 1..20, carte `avant_poste`,
  arrêt à l'entrée en vague 5, plafond 20 000 frames ; quatre lots indépendants de cinq.
- Prochaine étape : tableau de référence, choix chiffré par William, puis mesure après
  et partie solo. Suite complète, p2p et bench : non vérifiés ici à ce stade.

Commande de chaque lot (bornes inclusives) :

```sh
APP_VERSION=x target/headless/alacod-sim --game zombies --bots 4 \
  --profiles acheteur,acheteur,acheteur,acheteur --seeds 1..5 \
  --until-wave 5 --max-frames 20000 --json target/metrics/m0-revue-suite-contenu/s5-avant-lot1.json
```

Autres lots : `6..10`, `11..15`, `16..20`. Les durées CPU sous concurrence ne constituent
pas un benchmark. Les dégâts sont la somme des baisses de santé observées, comme le relevé
global du runner (arrondi par vague), pas une somme brute des événements de dégâts.

Propositions, non appliquées (variance conservée à 0..2, lots d'un zombie, types inchangés) :

| Réglage | Actuel | A : allègement modéré | B : départ doux, recommandé |
|---|---|---|---|
| `base_enemies` | 6 | 4 | 3 |
| `enemies_per_wave` | 4 | 3 | 2 |
| V1 | 6–8 | 4–6 | 3–5 |
| V2 | 10–12 | 7–9 | 5–7 |
| V3 | 14–16 | 10–12 | 7–9 |
| V4 | 18–20 | 13–15 | 9–11 |
| V5 | 22–24 | 16–18 | 11–13 |
| `spawn_interval_frames` | 60 (1 s) | 90 (1,5 s) | 120 (2 s) |
| `max_concurrent_enemies` | 8 | 6 | 4 |

Préparation : 180 frames ; délai après le dernier kill : 600 frames, suivies de la
préparation (environ 13 s au total). Ces deux valeurs restent inchangées dans A et B.
Les zombies des vagues 1–3 gardent 50 PV et la répartition 80 % `zombie_full` (80 px/s),
20 % `zombie_1` (120 px/s). Les multiplicateurs D29 restent inchangés, sans leur prêter
d'effet. Les deux options affectent aussi les vagues ultérieures et tous les nombres de
joueurs ; moins de kills peut retarder les achats. Il faudra mesurer la variante choisie,
puis la faire jouer en solo avant de valider S5.
