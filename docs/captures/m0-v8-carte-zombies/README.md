# Captures — m0-v8, carte avant_poste

Commandes de génération (voir le rapport `docs/taches/rapports/m0-v8-carte-zombies.md` ;
`map_preview` étant cassé de manière pré-existante, le rendu homologué est
`play_scenario --capture`, même rendu 960×540) :

```bash
cd <worktree>/alacod && source ../env.sh
../target/headless/play_scenario tests/scenarios/avant_poste_demo.ron \
    --capture /tmp/m0-v8/work/caps5 --every 40
```

Les 5 PNG ci-dessous sont un sous-ensemble des 90 captures de `caps5`
(une toutes les 40 frames, f0..f3560), chacun < 100 Ko.

| Fichier            | Frame | Ce que montre events.json à cet instant                                  |
|--------------------|-------|--------------------------------------------------------------------------|
| `frame_00000.png`  | 0     | Salle de départ (Depart) vide, joueur au sud, avant la montée au nord.   |
| `frame_00400.png`  | 400   | Vague 1 terminée (f330, 2 zombies tués), vague 2 en préparation ; le joueur tient la position nord et tire vers la fenêtre ouest (cassée f~200). |
| `frame_01600.png`  | 1600  | Vague 3 en cours : 3 zombies tués (f1448, f1467, f1593), le 4e converge vers la fenêtre ouest. |
| `frame_02400.png`  | 2400  | Vague 5 : zombies arrivés (f2386) dans la salle, premiers coups tirés ; la victoire est déjà déclarée (f2236, entrée en vague 5 = max_wave). |
| `frame_03200.png`  | 3200  | Fin de partie : victoire, vague 5 nettoyée (14e kill à f2556), joueur debout. |
