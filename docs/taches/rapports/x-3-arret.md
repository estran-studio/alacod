# X-3 « Verbes D1 » — arrêt (2026-10-27, décision de William : reporté après X-5)

- **Branche poussée, non fusionnée** : `x-3-verbes-d1` @ `0e57f6f3`, base `be732707`, en retard sur `main` (avant SB-6/SB-7 et X-4a fusionnés). L'en-cours non commité est annulé (`git checkout -- . && git clean -fd`).
  Seul un remplissage du rapport était en cours ; aucun code n'a été perdu.
- **`35d94c5e`** : condition `Timing` (horloge, début de dash, début de coup), renvoi de projectiles par une hitbox de mêlée (`ProjectileReflected` neutre) et
  `WeaponConfig::move_speed_mult` (poids). Selon le rapport du worker (non rejoué ici), la suite complète passait en 4 tranches sans BLESS et sans trace existante modifiée.
  `schedule_order_diff` : seulement le dump à targets séparés sur le Mac, avec un type rollback de plus avant la clôture. Il y a aussi une capture Mac du renvoi.
- **Revue indépendante de `35d94c5e` : un P1 et trois P2.** P1 : `TimingSource` exige `Clock`. P2 : renvoi ordonné avant la création ou l'expiration des hitbox ; horloge
  d'étage lue avant sa remise à zéro ; peau de balle renvoyée mal restaurée. Avec ces correctifs, la livraison restait partielle.
- **`0e57f6f3`** (commit du worker stoppé par quota) déclare corriger P1/P2 et livrer : parade mêlée contre mêlée (`MeleeParried`), notes `TimingJudged` en
  `FrameEvents`, hit stop « cible seule », musique calée sur l'horloge (`game::music_sync`, piste CC0 du testbed). Il ajoute aussi cinq scénarios neufs avec leur trace.
  **Non vérifié** : le rapport committé dit « À REMPLIR » pour les tests unitaires et la suite. Je n'ai lu aucun résultat de test de ce SHA ; aucune revue.
- **Dettes** (sur la branche) : D149, D150 et D151 marquées livrées par `0e57f6f3`, sans preuve ; D152 ouverte (repères « début de rechargement » et « coup qui arrive »).
- **Reprise (après X-5)** : sur `0e57f6f3`, jouer les unitaires (`combat`, `effects`, `content`, `game`), `make test_scenarios` sans BLESS, `check-forbidden` et
  `check-rollback-registration`. Puis rebaser sur `main`, lancer `make schedule_order_diff` et une revue indépendante. Enfin, voir sur le Mac le hit stop et la musique.
  Ne garder que les verbes dont 1837 aura montré le besoin en X-5.
