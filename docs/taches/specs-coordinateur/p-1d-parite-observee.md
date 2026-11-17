# Tâche : P-1d « Parité observée : visée au curseur et rechargement des fantassins » (Anthropic Sonnet) — jalon **S1 dim 15 nov**, avant tout le reste

Lis `~/orca/runs/alpha/commun.md`, `CLAUDE.md`, la ligne P-1d du § 6 de `docs/taches/plan-1837-cli.md` (**sa preuve fait foi**), les rapports `docs/taches/rapports/` de P-1, P-1b et R-1.
Branche `wquintal/p-1d-parite-observee`, worktree `~/orca/workspaces/alacod/p-1d-parite-observee` (partie d'`origin/main` `d54ab899`).

## Le verdict (William, 14 nov, sur 1837 `origin/main` `7d78906`, engine `e054e0a9`) : « on est encore loin de la parité avec les démos du labo »
William testait **à la souris**.
1. **Son personnage ne suit pas le curseur : il ne peut pas viser.** Banc (`~/Project/bascanada/1867_lore/tools/site/armes.js`, lecture seule) : `regard: "visee"` (A:27) ;
   A:786-788 : `if (R.regard === "visee" || a) J.cap = J.visee; J.yaw = arrondir(J.cap);` (le corps prend le cap de la visée, arrondi aux 8 directions).
   `1837/docs/parite-labo.md` l. 74 dit « identique » d'après une **lecture du code** (`FacingDirection` depuis la visée dès |pan| > 5) : personne ne l'a observé à l'écran.
2. **Les fantassins n'ont pas leur animation de rechargement complète.** Banc A:695-696 : `const d = clip("rechargement"); etatEnnemi(e, "recharge", R.eRecharge, "rechargement",
   { unefois: true, vitesse: d ? d / R.eRecharge : 1, fondu: 0.1 });` : le clip (48 images à 24 i/s = 2 s) est joué **une fois**, étiré sur 3 s (vitesse 2/3), fondu 0,1 s.

## Méthode imposée
- **La cause d'abord, sans la supposer.** Reproduis dans le jeu 1837 réel : copie jetable de 1837 `origin/main` (ex. `git worktree add /tmp/p1d-1837 origin/main` depuis
  `~/Project/bascanada/1837`, **jamais** de changement dans ce checkout ni de commit dans 1837), engine pointé sur ton worktree (`alacod engine local` ou `[patch]` dans la copie),
  build **release** natif, `ALACOD_MUTE=1`. Visée par une **vraie position de curseur dans la fenêtre** (déplacement réel du pointeur, ex. `cliclick`/`osascript`, ou événements
  de fenêtre réels), **pas** `PlayerScript::aim` ni une visée injectée par scénario. Trace toute la chaîne : curseur → `CursorPosition` → pan de l'input → visée → `FacingDirection` →
  orientation du modèle de R-1 (présentation). Pour le fantassin : quel clip, quelle vitesse, combien de lectures, quel fondu pendant les 180 frames de rechargement.
  **La fenêtre du jeu et le pointeur du Mac sont libres pendant l'absence de William (sam 14, à partir de 10 h 55 NST)** ; ferme le jeu dès qu'une capture est faite.
- Corrige dans l'engine ce qui est générique ; si une part relève de la **donnée de 1837** (RON, manifeste), décris-la précisément dans ton rapport et dans un `status`
  (le coordinateur la transmet au coordinateur 1837) ; ne la commite pas.
- **Preuves** pour chaque défaut : un test qui **échoue avant et passe après**, au plus près de la cause ; une **capture côte à côte banc / jeu** du même geste (curseur à un point fixe ;
  fantassin du tir à la fin du rechargement). Banc : page `1867_lore/site/labo/armes/` ou `tools/site/armes.js` servie en local, **une** capture ponctuelle par état (pas de boucle
  d'automatisation de Chrome). Captures sous `~/orca/runs/alpha/logs/p-1d-captures/`.
- Aucune trace existante modifiée sans BLESS justifié par écrit ; `make check_rapide` (journal `~/orca/runs/alpha/logs/p-1d-parite-observee-check-rapide-<sha>.log`) ;
  pas de suite complète sur le Mac. § 122, D218, rapport `docs/taches/rapports/p-1d-parite-observee.md`, LIVRÉ.

## Ordre (studio, sam 14, 10 h 55 NST)
1. **Les deux défauts nommés d'abord.** Dès qu'une tranche est verte (test avant/après, capture, `check_rapide`) : `status` « E prêt <sha> » **tout de suite** ; le coordinateur
   merge et donne le SHA au coordinateur 1837 pour que `./jouer.sh` l'apporte à William à son retour. Ne pas attendre la suite pour signaler.
2. **Ensuite, dans le même lot** (c'est la correction du livrable de P-1) : **la revue observée de la salle.** Pour chaque état du banc, joueur et fantassin
   (`armes.js`, `gigue.js`, `allures.js` : attente, marche, épauler, viser, tir, rechargement, coup de hache, gigue, touché, mort) :
   - noter ce que le banc affiche : quel clip, une fois ou en boucle, étiré sur quelle durée, quel fondu, corps tourné vers quoi, arme en main ;
   - l'observer dans le jeu réel : capture côte à côte banc / jeu du même état ;
   - lister chaque écart (tableau dans le rapport) ; corriger ceux de l'engine (une tranche verte = un « E prêt ») ; décrire ceux qui relèvent des données de 1837.
   - Rédige pour 1837 le **nouveau texte des lignes de `docs/parite-labo.md`** : chaque ligne « identique » devient « observé le <date> » (avec la capture) ou « écart » (avec
     sa description) ; plus aucun « identique » d'après une lecture de code. Ce texte va dans ton rapport (section dédiée) : le coordinateur 1837 l'applique dans son dépôt.

Règles : jamais de profil debug ; un seul cargo ; sweep après ; **commite et pousse à chaque étape** ; ne touche ni `~/Project/bascanada/alacod` ni
`~/Project/bascanada/1837` (copie jetable seulement) ; `status` « cause <défaut> : … » dès qu'une cause est établie, « E prêt <sha> » par tranche verte, un seul `worker_done` à la toute fin.
