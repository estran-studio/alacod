# M2-T18 seconde partie — point de travail

Branche `m2-t18-hud-gungeon`, créée depuis `origin/main`.
Décision de William : `bevy_hui` pour les écrans, `hud.ron` conservé pour le HUD.

- Implémenté : retrait du renderer maison des menus ; listes/focus génériques en Hui.
- Implémenté : HUD gungeon (objets, consommables, charge, minicarte dérivée).
- Implémenté : découverte et lint des fichiers HTML, y compris le contenu embarqué wasm.
- Écran de fin : conversion Hui, résumé et commandes `RunRequest` conservés.
- Écran de mutation : maintien du modèle de cartes chronométrées et de sa validation par bits rollback ; dette de rendu détaillée dans les conventions §56.
- Smoke effectué : `ui_inventory` avec rendu natif, captures 9/20/31 observées ; HUD, passifs, actif prêt puis vide, inventaire et focus lisibles, aucune panique.
- Dépendances téléchargées ; cible wasm32 installée. Aucune trace réécrite.
- En cours : autres captures, puis fusion `origin/main`, vérifications standard sur la fusion, rapport final et commit `LIVRÉ`.
