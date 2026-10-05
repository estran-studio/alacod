# m1-analyse-200-throne — pourquoi les bots perdent sur `throne` (200 graines)

## État en cours

- Fait : branche depuis origin/main `a88339a`. Lecture des journaux des 120 premières graines à
  2 bots (`../m1-200-throne/throne-2-*.log`, binaire `alacod-sim-61ac539`) : 86 run finies,
  27 défaites, 7 soft-locks, 0 desync. **Toutes les défaites au niveau d'index 2 (`niveau_3`,
  étage du boss `roi_rat`)**, 26 sur 27 avec un mort sur deux ; soft-locks : 2 au niveau 1,
  5 au niveau 2.
- En attente : JSON et journaux complets (« données prêtes » d'orch), feu vert pour les rejeux.
