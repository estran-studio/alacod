# Jouer à gungeon (pour William)

Toutes les commandes se lancent depuis la racine du dépôt (`alacod/`). Elles ont été essayées sur un Mac Apple
Silicon (rapport `docs/taches/rapports/m2-salles-murs-gungeon.md`, §5 et « Suivi de revue »). Ce qui n'a pas pu être
essayé est dit comme tel.

## Préparer

1. Toolchain Rust nightly choisie par `rust-toolchain.toml` (installée par `rustup` au premier `cargo`), plus les
   prérequis système de `.github/workflows/pr.yaml`.
2. Compiler une fois (≈ 20 min à froid, quelques secondes ensuite) :

   ```bash
   APP_VERSION=dev cargo build -p gungeon --features native
   ```

   Le binaire est `target/debug/gungeon` (profil `dev` optimisé : il tourne à pleine vitesse). **Ne pas déplacer le
   dépôt après la compilation** : le binaire lit `games/gungeon` par le chemin capturé à la compilation.

Les commandes `cargo run` ci-dessous recompilent au besoin ; `make gungeon` est le raccourci du solo.

## Les modes

| Je veux… | Commande | Notes |
|---|---|---|
| **Jouer seul** | `make gungeon` | Équivaut à `APP_VERSION=dev cargo run -p gungeon --features native -- --local-port 7000 --players localhost` (le Makefile ajoute aussi les features de développement de `scripts/dev-features.sh`, d'où une compilation à part). Démarre dans le **hub** : le marchand (profil, déblocages) et la porte, qui lance la run. |
| **Jouer seul, directement dans une run, toujours la même carte** | `ALACOD_SKIP_HUB=1 make gungeon GARGS="--seed 10"` | Saute le hub. `--seed <n>` fixe la graine de la carte (deux parties avec la même graine ont les mêmes salles). |
| **Deux joueurs sur le même Mac** | `APP_VERSION=dev cargo run -p gungeon --features native -- --local-port 7000 --players localhost localhost` | Deux héros dans la partie. **Limite : les deux reçoivent les mêmes touches et la même manette** (même table de touches pour chaque joueur local, voir plus bas) : utile pour voir deux héros, pas pour jouer à deux sur un clavier. Pour jouer à deux : en ligne, ci-dessous. |
| **Jouer à deux en ligne (un Mac chacun, ou deux fenêtres)** | voir « En ligne à deux » | Une machine héberge le serveur de rencontre (signaling), les deux lancent le jeu. |

**N'utilisez pas** `make gungeon GARGS="--players localhost localhost"` : le Makefile ajoute déjà un `--players localhost`,
et `--players` s'**ajoute** au lieu de remplacer : cela donne **trois** joueurs.

## Éditer en jouant (mode éditeur)

Pour régler le jeu pendant qu'on y joue (avec Claude, à la voix ou au clavier) :

- `alacod editor games/zombies` (ou `gungeon`, `isaac`, `throne`, `testbed` ; alias `make editor GAME=zombies`) : le jeu choisi, partie en marche, clavier actif ;
- `alacod sandbox games/zombies` (alias `make sandbox GAME=zombies`) : le **bac à sable**, voir ci-dessous.

Dans les deux cas la partie tourne dès le départ, les étiquettes F6 (nom, prix, état) se voient au-dessus des entités, et le port 15702 répond :
`alacod remote discover` puis `brief` (première ligne `editor=True online=False check_distance=0`). Limite : édition en **local** seulement
(pas de partie en ligne) et à `check_distance` 0. Le mode d'emploi complet est dans `docs/editeur-vocal.md`.

### Bac à sable : essayer un personnage, une arme, un ennemi

`--sandbox [profil]` fonctionne sur **tout** jeu construit avec l'engine (les cinq clones ; un jeu créé par `alacod new` l'a aussi, sans une ligne de code : `alacod sandbox` dans son dossier (`native` : rechargement à chaud des RON) ; preuve : `make test_alacod_cli`) : l'**arène vide** du jeu (une salle de murs et le joueur, rien d'autre : `alacod new-arena`, `docs/editeur-vocal.md` §2.3 ; ni vagues ni étages), un
**profil** de départ (`sandbox/<profil>.ron` du jeu : personnage, arme, objets, argent, modificateurs, ennemis posés ; `arene` n'en pose aucun), le mode éditeur allumé. Sans nom, le profil est `arene` ; chaque jeu livre aussi `lourd`.

| Je veux | Commande |
|---|---|
| Le bac à sable d'un jeu (profil `arene`) | `alacod sandbox games/zombies` (ou `gungeon`, `isaac`, `throne`, `testbed` ; alias `make sandbox GAME=zombies`) ; `--detach` : en arrière-plan, la commande rend la main quand le jeu répond |
| Un autre profil | `alacod sandbox games/isaac --profile lourd` (`make sandbox GAME=isaac PROFIL=lourd`) |
| Une autre graine d'arène | `alacod sandbox games/zombies --seed 42` (sans `--seed`, toujours la graine 123456 : l'arène est la même à chaque lancement) |
| Sans fenêtre (VPS, CI) | `alacod sandbox games/zombies --headless` |
| Un jeu créé par `alacod new demo` | `cd demo && alacod sandbox` (profil `assets/sandbox/arene.ron` ; un autre : `alacod new-content Sandbox lourd`, puis `alacod sandbox --profile lourd`) |
| Une autre arène vide (taille, joueurs) | `alacod new-arena games/zombies --name grande --size 40x30 --use` |
| Le remplaçant de `make character_tester` | `make character_tester` (alias de `make sandbox GAME=zombies`) |

Pendant la partie, par `alacod remote` (ou à la voix, via Claude) :

```bash
alacod remote profile              # les profils du jeu, l'actif marqué *
alacod remote profile lourd        # « charge le profil lourd » : arène du profil, puis restart
alacod remote give --weapon shotgun         # « donne-moi le fusil »
alacod remote give --item bottes            # un objet (passif, actif ou consommable)
alacod remote give --set-currency 500       # « mets-moi 500 sous » (--currency 100 ajoute, -100 retire)
alacod remote spawn zombie_runner --ahead   # « spawne-moi un zombie_runner devant moi »
alacod remote modifier MoveSpeed Mul 1.2    # « un peu plus rapide » (unmodify pour retirer)
alacod remote restart                       # « reset » : joueur recréé, profil reposé, ennemis du profil reposés
```

`restart` rejoue le profil actif : ce que tu as donné (`give`, `modifier`) ou fait apparaître (`spawn`) disparaît. Modifie le RON d'un profil, puis `alacod remote reload && alacod remote restart`.
Limites : local seulement (une session en ligne est refusée au lancement), `give` n'est pas enregistré dans un scénario (`save`).

## En ligne à deux

Il faut Docker (ou OrbStack) sur **une** des deux machines, appelée A ci-dessous (celle dont l'adresse IP est connue de
l'autre ; même réseau local, ou adresse joignable).

1. Sur A, lancer le serveur de rencontre :

   ```bash
   docker compose -f docker-compose.ci.yaml up -d signaling
   ```

   Port 3536 par défaut ; `MATCHBOX_PORT=3570 docker compose -f docker-compose.ci.yaml up -d signaling` pour un autre port.
   Arrêt : `docker compose -f docker-compose.ci.yaml down`.
2. Sur **chaque** machine (le même salon, le même `--players`, un `--cid` et un `--name` différents) :

   ```bash
   # machine A
   APP_VERSION=dev target/debug/gungeon --number-player 2 --matchbox ws://<IP de A>:3536 --lobby salon \
     --players localhost remote --cid alice --name alice
   # machine B
   APP_VERSION=dev target/debug/gungeon --number-player 2 --matchbox ws://<IP de A>:3536 --lobby salon \
     --players localhost remote --cid bob --name bob
   ```

   (`localhost` = le héros de cette machine, `remote` = l'autre.) Quand les deux se voient, la partie démarre ; chacun
   joue son héros avec **sa** souris et **son** clavier. Pour une run précise : ajouter à chaque ligne
   `ALACOD_SKIP_HUB=1` devant la commande et `--seed 10` à la fin, **identiques des deux côtés**.
3. Le serveur public `wss://allumette.estran.studio` **marche, mais seulement depuis le site** (section suivante) : en
   ligne de commande, `--matchbox wss://allumette.estran.studio` est refusé (le signaling exige un ticket de session) et
   `--allumette` parle l'ancienne API de salons, pas celle du site ; il n'utilise pas non plus TURN (D165). Entre deux
   binaires natifs, utiliser le `docker compose` ci-dessus.

Si un pair est annoncé par le serveur mais que la connexion ne s'établit pas (réseau qui perd des paquets : environ 1 démarrage sur 25
à 3 % de perte), l'écran du lobby le dit (« Connexion au pair… »), puis, au bout de 15 s, rouvre une connexion (« nouvelle tentative
(2/2) ») ; si elle échoue aussi, il revient au salon (« retour au salon »). Rien à faire : les deux joueurs se retrouvent.

Si les deux parties divergent, un écran « DESYNC » s'affiche (la partie commune est perdue) : notez la graine et les
frames, c'est un défaut du moteur.

## En ligne depuis le site (Allumette, deux réseaux)

Le site `https://alacod.estran.studio/online` passe par Allumette (`allumette.estran.studio`, TURN `turn.estran.studio:3478`),
opéré par William (dépôt `estran-studio/allumette`, redéploiement par un push sur `main` ; état et procédure :
`docs/contrat-lancement-web.md`, « Service en production »). Une session dure **30 min** au plus (`SESSION_TTL`) : à
l'expiration, la page revient au salon, la partie est perdue ; jouer des parties de moins de 30 min, retour au salon entre
deux. Protocole de la recette S1-3 (deux machines avec Chrome, A sur la maison, B sur le partage de connexion d'un
téléphone ; **VPN coupé** sur les deux : derrière un VPN qui bloque l'UDP, la connexion prend ≈ 40 s de plus, D166 ; 45 min) :

1. A : ouvrir `https://alacod.estran.studio/online`, un pseudo, créer un salon **zombies**, copier le lien d'invitation.
2. B (cache vidé) : ouvrir le lien (moins de 15 min), rejoindre ; chronométrer de « Lancer » à « Connexion établie ».
3. Les deux cliquent « Je suis prêt », A lance ; sur A, ouvrir `chrome://webrtc-internals` dans un autre onglet.
4. Noter le type du candidat retenu (`host`, `srflx` ou `relay`) ; derrière le partage d'un téléphone, `relay` est attendu.
5. Jouer 5 min : déplacement, tir, une mort ; noter tout écran « DESYNC », gel ou retour au salon imprévu (avec l'heure).
6. Fin de partie, bouton « Lobby » : les deux pages reviennent au salon.
7. Relancer une seconde partie dans le même salon (nouvelle session, nouveau ticket).
8. Refaire 1 à 7 avec **throne**.
9. B ouvre une invitation depuis `https://alacod.pages.dev` ou une prévisualisation d'un autre build : attendu « Cette
   invitation utilise une autre version du jeu. »
10. Envoyer les heures, les captures de `webrtc-internals` et la console (F12) de chaque machine en cas d'erreur.

Si le chronomètre du point 2 dépasse 60 s, le ticket de signaling a expiré avant l'ouverture du socket : la page revient
au salon (ou l'écran d'attente reste jusqu'à 2 min sur le build de prod) ; relancer depuis le salon, le cache est chaud.

## Touches

Clavier + souris. Les touches sont des **positions physiques** : sur un clavier AZERTY, W A S D sont les touches Z Q S D
et la touche « Q » (blank) est la touche A.

| Action | Touche |
|---|---|
| Se déplacer | **W A S D** |
| Viser / tirer | souris / **clic gauche** |
| Interagir (ouvrir, acheter, ramasser, relever) | **H** |
| Recharger | **R** |
| Changer d'arme / de mode de tir | **Tab** / **Z** |
| Mêlée | **F** |
| Lâcher l'arme | **L** |
| Objet actif | **Espace** |
| Blank (efface les balles proches) | **Q** |
| Bombe | **E** |
| Sprint | **Maj gauche** |
| Esquive (dash) | **C** |
| Pause / inventaire | **Échap** / **I** |
| Choix de mutation | **1 / 2 / 3** |
| Étiquettes au-dessus des entités (nom, prix, état) | **F6** |
| Légende des silhouettes et couleurs | **F7** |
| Compteur de FPS (si la page est ouverte avec `?perf=1`) | **F8** |
| Débogage : champ de flux / profil / états des ennemis | **F3** / **F4** / **F5** |

### Manette

Disposition Xbox (une manette PlayStation donne : Croix = A, Rond = B, Carré = X, Triangle = Y, L1/R1 = LB/RB, L2/R2 =
LT/RT). Le schéma est celui accepté par William le 2026-10-22 ; la table du code est `character/control.rs`
(`get_input_map`), et **l'écran « Touches » de la pause** l'affiche en jeu, filtré par jeu (une bombe ou une esquive que le
personnage n'a pas n'y figure pas).

| Action | Manette |
|---|---|
| Se déplacer | **stick gauche** (huit directions, seuil 0,4 par axe) |
| Viser | **stick droit** (la dernière direction est gardée au repos ; la souris reprend la main dès qu'elle bouge) |
| Tirer | **RT** (analogique : tire à mi-course, relâche sous le tiers de la course ; LT, l'objet actif, de même) |
| Recharger / Interagir / Mêlée | **X** / **Y** / **B** |
| Arme suivante / mode de l'arme | **RB** / **R3** (stick droit enfoncé) |
| Esquive / sprint | **LB** / **L3** (stick gauche enfoncé) |
| Objet actif | **LT** |
| Lâcher l'arme / blank | croix **↓** / croix **↑** |
| Bombe | **A** (sur un écran ouvert ou un choix de mutation, A valide : la bombe ne part pas) |
| Choix de mutation | croix **← →** pour surligner, **A** pour valider |
| Pause / inventaire | **Start** / **Select** |
| Dans un menu | croix **↑ ↓** choisir, **A** valider, **B** retour (« Touches » dans la pause) |
| Écran de fin | **A** rejouer, **B** retour au lobby (après 0,8 s, pour ne pas valider d'un geste de jeu) |

La croix ne déplace plus le personnage. Sous un écran ouvert (pause, inventaire), la manette ne donne que le déplacement et le
sprint à la partie : A, B, la croix y naviguent sans lâcher de bombe ni donner de coup, et un bouton tenu pendant l'écran reste
muet jusqu'à son relâchement.

Dans le navigateur (Chrome), la manette n'est visible qu'**après un premier appui** sur l'un de ses boutons, page au premier
plan : c'est la règle de l'API Gamepad, pas du jeu. Elle est lue par `bevy_gilrs` (`gilrs` en wasm), sans code propre au
site. Débrancher puis rebrancher en cours de partie est reconnu au premier appui suivant. Preuve et limites :
`docs/taches/rapports/polissage-manette.md`.

## Variables utiles

- `ALACOD_SKIP_HUB=1` : démarrer directement dans la run.
- `--seed <n>` (argument de tous les jeux : `zombies`, `gungeon`, `isaac`, `throne`, `testbed`) : graine de la carte.
- `ALACOD_LABELS=1` : étiquettes allumées dès le départ (F6 les bascule).
- `ALACOD_MUTE=1` : aucun son.

Où regarder les parties automatiques (vidéos des bots, page de revue) : `docs/taches/rapports/m2-salles-murs-gungeon.md`, §4.
