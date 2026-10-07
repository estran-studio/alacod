# Session de test manuelle enregistrée — `zombies` et `throne` (William)

But : valider à la main M0 (`zombies`) et M1 (`throne`) après la fusion de `revue-m0-suite` (movement-feel, R4, R5)
dans `main`. Tu joues en te filmant avec OBS **et en parlant à voix haute** ; la transcription de ta voix, les
fichiers d'enregistrement du jeu et les logs me suffisent ensuite pour tout analyser et ouvrir les tâches.
Durée : 1 h 30 à 2 h, en blocs. Tu peux t'arrêter entre deux blocs.

## 0. Avant de commencer (10 min)

1. Attendre mon message « main prêt pour la session » (fusion de `revue-m0-suite` vérifiée), puis :
   ```bash
   git fetch origin && git checkout main && git pull
   git log -1 --format='%h %s'          # dis ce sha à voix haute au début de la vidéo
   make zombies ARGS="--profile headless"   # première compilation (long à froid), puis quitte
   ```
2. **OBS** : capture de la fenêtre du jeu + micro. 1080p, 60 i/s, une scène. Un fichier par bloc, nommé
   `bloc-A.mkv`, `bloc-B.mkv`, etc.
3. **Transcription** : celle que tu préfères (Whisper local, MacWhisper, OBS + plugin). Garde les horodatages
   (format SRT ou « [mm:ss] texte »). C'est ce fichier qui compte le plus.
4. Chaque partie se lance **avec enregistrement des inputs** (le jeu écrit un scénario rejouable à la fermeture).
   C'est ce qui me permet de rejouer exactement ce que tu as vu :
   ```bash
   ALACOD_RECORD=$PWD/tests/scenarios/manuel_<bloc>_<n>.ron make zombies ARGS="--profile headless"
   ALACOD_RECORD=$PWD/tests/scenarios/manuel_<bloc>_<n>.ron make throne  ARGS="--profile headless"
   ```
   Ferme la fenêtre normalement à la fin de chaque partie (pas Ctrl-C) : c'est là que le fichier s'écrit.

## 1. Comment parler pendant que tu joues

La transcription sera lue par moi : quelques mots-clés suffisent à la rendre exploitable. Dis-les **en début de
phrase**, puis décris librement.

| Mot-clé | Quand | Exemple |
|---|---|---|
| **« Début bloc A »** / **« fin du bloc »** | début et fin de chaque bloc | « Début bloc B, throne solo, sha 4f2a91c. » |
| **« Bug »** | quelque chose de faux (règle, collision, affichage incohérent) | « Bug : le zombie passe à travers la porte fermée, en bas à gauche. » |
| **« Sensation »** | ce n'est pas faux, mais ça ne te plaît pas (rythme, dégâts, vitesse, prix) | « Sensation : le dash est trop court, j'ai l'impression de ne pas bouger. » |
| **« Bien »** | ce qui marche et qu'il faut garder | « Bien : le son du fusil à pompe. » |
| **« Pas compris »** | tu ne sais pas ce qui se passe ou quoi faire | « Pas compris : pourquoi le portail n'apparaît pas. » |
| **« Idée »** | une proposition | « Idée : afficher le prix au-dessus de la porte. » |
| **« Frame »** + le nombre | juste après un bug, lis le compteur de frame en bas à droite | « Frame 4 812. » |
| **« Note »** + chiffre | à chaque question notée de la fin d'un bloc | « Note : 3. » |

Pense à voix haute le reste du temps (« je vais acheter… », « je ne vois pas d'où ça tire »). Un silence ne
m'apprend rien ; une hésitation dite, oui.

## 2. Touches (clavier)

| Action | Touche |
|---|---|
| Se déplacer | **W A S D** (les flèches bougent la caméra ; flèche droite bouge aussi le joueur : R6, connu) |
| Viser / tirer | souris / **clic gauche** |
| Sprint | **Shift gauche** |
| Dash | **C** |
| Interagir (acheter, ouvrir, réparer, réanimer) | **H** |
| Recharger | **R** (ne doit plus relancer la partie : R5) |
| Arme suivante / mode de tir | **Tab** / **Z** |
| Mêlée / lâcher l'arme | **F** / **G** |
| Mutation (throne) | **1 2 3**, ou flèches + **Entrée** |
| Caméra verrouillée / libre | **P** / **O** |

Manette : croix directionnelle, Nord = interagir, Ouest = recharger, Est = mêlée, Sud = valider.

## 3. Les blocs

### Bloc A — `zombies` solo, prise en main (20 min, 2 parties)

Lancer : `ALACOD_RECORD=$PWD/tests/scenarios/manuel_A_1.ron make zombies ARGS="--profile headless"`

À faire exprès, en le disant :
1. **Déplacement** (movement-feel, jamais testé par un humain) : courir, sprinter, changer de direction, dasher
   contre un mur, dasher à travers un zombie (i-frames).
2. Acheter une arme au mur, un perk (Juggernog), ouvrir une porte : prix affiché, prompt, disparition après achat.
3. Laisser un zombie casser une fenêtre, puis la réparer (**H**).
4. Ramasser chaque power-up qui tombe et dire lequel.
5. Recharger avec **R** en pleine partie (ne doit pas relancer).
6. Mourir (ou abandonner), puis **Rejouer** : les murs doivent toujours bloquer (R4).

Fin du bloc, une note de 1 à 5 chacune, dite à voix haute : déplacement · tir · lisibilité de ce qui m'attaque ·
économie (assez d'argent ?) · envie de rejouer. Puis : « la chose à changer en premier serait… ».

### Bloc B — `zombies` solo, partie longue (20 min, 1 partie)

Lancer avec `manuel_B_1.ron`. Joue « pour de vrai », le plus loin possible. Dis à chaque vague : numéro, ce qui
change, si c'est trop facile ou trop dur. Reviens sur tes notes d'avant (S1 « le gameplay est vraiment mauvais »,
S2 « la carte ne fait aucun sens ») : **qu'est-ce qui est mauvais, précisément ?** (tir, rythme, zombies, carte,
feedback, sons…).

Fin du bloc : mêmes cinq notes, plus « vague atteinte » et « pourquoi je suis mort ».

### Bloc C — `throne` solo, découverte (25 min, 2 runs)

Lancer : `ALACOD_RECORD=$PWD/tests/scenarios/manuel_C_1.ron make throne ARGS="--profile headless"`

1. Première caverne : lis le HUD à voix haute (vie, niveau, rads, munitions par type, étage) et dis ce que tu
   ne comprends pas.
2. Armes : ramasse et essaie **chaque** arme qui tombe ; dis pour chacune si elle a un caractère (D51 a changé la
   précision de toutes les armes).
3. Ennemis : nomme ce que tu vois (« le carré bleu tire en éventail »…) et dis lesquels sont lisibles avant
   d'attaquer (télégraphes au sol).
4. Écran de mutation : lis les trois cartes, choisis, dis pourquoi.
5. Portail : dis quand tu le vois et si la transition est claire.
6. Roche destructible : essaie de la casser (explosions).
7. **Troisième étage, le boss** : comment tu le gagnes ou le perds, et si c'est trop facile (les bots le passent
   198 fois sur 200 depuis D51).

Fin du bloc, notes de 1 à 5 : déplacement · armes · lisibilité des ennemis · difficulté du 3e étage (1 = trop
facile, 3 = juste, 5 = trop dur) · envie de rejouer. Puis : « la chose à changer en premier serait… ».

### Bloc D — `throne` solo, run sérieuse (15 min, 1 run)

`manuel_D_1.ron`. Va le plus loin possible, y compris après le boss (la run boucle). Dis l'étage atteint et la cause
de la mort.

### Bloc E — à deux (optionnel, 20 min)

Seulement si ton ami est disponible : recette `docs/jouer-a-deux.md` (même commit des deux côtés). `zombies` d'abord
(à terre et réanimation, power-ups partagés, fin de partie et retour au lobby, **Rejouer** en ligne), puis `throne`
si ça tient. Dis à voix haute chaque « lag », saccade ou désaccord entre vos deux écrans, avec le numéro de frame.

## 4. Après la session

Dépose, dans un dossier `docs/digests/session-manuelle-<date>/` d'une branche `session-manuelle-<date>` (ou
envoie-les-moi autrement) :
- la **transcription** de chaque bloc (SRT ou texte horodaté) — indispensable ;
- les fichiers `tests/scenarios/manuel_*.ron` écrits par le jeu ;
- les logs de partie (`game_run_*`, chemin affiché au lancement) ;
- les vidéos OBS seulement si elles font moins de quelques centaines de Mo ; sinon garde-les, je te demanderai un
  passage précis par son horodatage.

Puis dis-moi « session déposée ». Je te rends sous un jour :
1. le carnet `docs/digests/revue-m0.md` et un carnet M1 remplis (une ligne par constat, comme R1–R9) ;
2. pour chaque **bug** : un scénario rejoué depuis ton enregistrement qui le reproduit, et une tâche ;
3. pour chaque **sensation** : une proposition de réglage chiffrée, que tu valides ou non ;
4. tes notes 1–5 en tableau, zombies contre throne ;
5. la liste des décisions qui restent à toi (graine fixe R8, carte S2, rééquilibrage du 3e étage…).
