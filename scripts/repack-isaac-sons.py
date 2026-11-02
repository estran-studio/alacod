#!/usr/bin/env python3
"""scripts/repack-isaac-sons.py (S2-5, docs/conventions.md § 106). Usage : python3 scripts/repack-isaac-sons.py [nom ...]
Script frère de scripts/repack-throne-sons.py (même mécanique, `process`/`extract`/`TARGET` de scripts/repack-zombies-sons.py), mêmes archives de
~/Downloads (Snake's, Kenney, Sonniss), mêmes règles : un fichier extrait à la fois sous ~/orca/workspaces/alacod/packs/, jamais un bundle entier ; mono
44,1 kHz, rognage, fondu, niveau RMS cible, crête <= -1 dBFS, ogg vorbis q5.
Les .ogg de Snake's et Kenney vont sous games/isaac/assets/sounds/ (commités, repli public) ; ceux de Sonniss sous sounds/sonniss/ (gitignoré, jamais
publiés : surcharge locale ui/feedback.local.ron, dépôt privé d'assets, `isaac/`). Licences dans games/isaac/assets/assets.yaml.
Choix faits sur le nom et la durée, sans écoute (aucun son sur le Mac) : à juger à l'oreille par William, remplaçables un par un."""
import importlib.util, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("repack_zombies_sons", HERE / "repack-zombies-sons.py")
z = importlib.util.module_from_spec(spec)
spec.loader.exec_module(z)
z.OUT = HERE.parent / "games/isaac/assets/sounds"
S1, K, SB = z.S1, z.K, z.SB

SN1 = "Snake's Authentic Gun Sounds/"
# Explosion (bombe, rochers, barils) : un tir très grave de Snake's, comme throne (même fichier, même traitement).
SNAKE = [("explosion", S1, SN1 + "Full Sound/7.62x54R/WAV/762x54r Single WAV.wav", "snake", 1.1, 0.35, -1200)]
# Événements propres à isaac, replis publics (Kenney impact, CC0).
KENNEY = [
    ("bomb", "Audio/impactTin_medium_001.ogg", 0.18, 0.05),            # mèche amorcée
    ("shell", "Audio/impactMetal_light_000.ogg", 0.16, 0.05),          # sou ramassé
    ("active", "Audio/impactBell_heavy_002.ogg", 0.7, 0.2),            # objet actif utilisé
    ("pickup_key", "Audio/impactMetal_medium_000.ogg", 0.25, 0.08),    # clé
    ("pickup_bomb", "Audio/impactMetal_heavy_000.ogg", 0.22, 0.07),    # bombe
    ("pickup_coeur", "Audio/impactSoft_medium_000.ogg", 0.25, 0.08),   # cœur
    ("pickup_coeur_ame", "Audio/impactGlass_light_000.ogg", 0.3, 0.1), # cœur d'âme
    ("boss_phase", "Audio/impactMining_001.ogg", 0.7, 0.2),
    ("floor", "Audio/impactPlate_medium_003.ogg", 0.65, 0.2),
]
# Mêmes événements que zombies et throne (même fichier, même traitement) : repris de scripts/repack-zombies-sons.py.
SHARED_PUBLIC = ["death_player", "boss_spawn", "boss_death", "room_cleared", "purchase", "pickup", "menu_navigate", "menu_confirm", "menu_back"]
SHARED_SONNISS = ["death_player", "room_cleared", "purchase", "pickup", "menu_navigate", "menu_confirm", "menu_back"]
# Sonniss propres à isaac : (nom, bundle, membre, durée, fondu[, secondes de silence initial coupées]). Mêmes fichiers que throne pour les boss,
# l'étage et l'explosion ; clés, cœurs et bombe de ramassage : pas de fichier Sonniss convaincant, ils gardent le repli Kenney.
SONNISS = [
    ("boss_spawn", 1, "344 Audio - Dinosaurs Vol. 1/ANMLRept_T Rex_344 Audio_Dinosaurs.wav", 2.6, 0.5),
    ("boss_death", 2, "Epic Stock Media - Halloween Game - Haunted House and Horror Audio Scare Kit/CREABeast_Creature Werewolf Growl Menacing Monstrous 06_ESM_HALG.wav", 2.4, 0.6),
    ("boss_phase", 2, "Epic Stock Media - Halloween Game - Haunted House and Horror Audio Scare Kit/DSGNEthr_Jumpscare Vocal Aggressive Whisper Harsh Distortion 02_ESM_HALG.wav", 1.2, 0.3),
    ("floor", 2, "Cinematic Sound Design - Ultra Transitions & Impacts/Transition Braam Slow Dark Creepy.wav", 2.0, 0.6),
    ("explosion", 2, "Epic Stock Media - Anime Game/EXPLDsgn_Explosion Small Blast Enemy Death Crunchy Boom Cartoon Noisy Crash Impact Delay 03_ESM_AG.wav", 1.4, 0.4),
    ("bomb", 2, "Cinematic Sound Design - Interface & Infographics/Interface Pop High Short.wav", 0.3, 0.08),
    ("shell", 2, "Cinematic Sound Design - Hybrid Game & UI Elements/Foley Coin Flip Single Fast.wav", 0.35, 0.1),
    ("active", 2, "Epic Stock Media - Anime Game/DSGNStngr_Power Up Bright Positive Successful Light Saturation Crash Shimmer 05_ESM_AG.wav", 1.2, 0.4),
    # Mort par famille : petites bêtes (araignée, mouche, follet), humains (rampant, suiveur, cracheur), spectres, brutes (colosse, gardien), machines.
    ("death_bete", 1, "344 Audio - Dinosaurs Vol. 1/ANMLRept_Hatchling Calling Out_344 Audio_Dinosaurs.wav", 1.0, 0.3),
    ("death_humain", 2, "Epic Stock Media - Humanoid Creatures Vol 4 - Monstrous and Undead Creature Vocalization Sound Sets/VOXReac_Construction Kit Male Flutter Death Vocal Stuttered Long 05_ESM_HC4.wav", 1.6, 0.5),
    ("death_spectre", 5, "SoundBits - Vox Bestiae - Source Elements/CREAEthr_Ethereal Entity Grim Pain Long 4_SNDBTS_VB-SE.wav", 1.4, 0.4),
    ("death_brute", 2, "Epic Stock Media - Humanoid Creatures Vol 4 - Monstrous and Undead Creature Vocalization Sound Sets/CREAHmn_Designed Orc Male Attack Long Heavy Hit Charged Up 03_ESM_HC4.wav", 1.6, 0.5),
    ("death_machine", 2, "Epic Stock Media - Tower Defense Game/ROBTMvmt_Tower Deploy Hitech Robot Motor Dark Thump Servo Whine 04_ESM_TDG.wav", 1.2, 0.4),
]
z.TARGET.update({
    "explosion": -18, "boss_phase": -20, "floor": -20, "shell": -24, "active": -22, "bomb": -22,
    "pickup_key": -24, "pickup_bomb": -22, "pickup_coeur": -24, "pickup_coeur_ame": -24,
    "death_bete": -21, "death_humain": -21, "death_spectre": -21, "death_brute": -21, "death_machine": -22,
})

if __name__ == "__main__":
    only = set(sys.argv[1:])
    for name, zp, member, pack, dur, fade, pitch in SNAKE:
        if not only or name in only:
            print(name, z.process(z.extract(zp, member, pack), name, dur, fade, pitch=pitch), "s", member.split("/")[-1])
    for name, member, dur, fade in KENNEY:
        if not only or name in only:
            print(name, z.process(z.extract(K, member, "kenney-impact"), name, dur, fade), "s", member.split("/")[-1])
    for name, zp, member, pack, dur, fade in z.ITEMS:
        if name in SHARED_PUBLIC and (not only or name in only):
            print(name, z.process(z.extract(zp, member, pack), name, dur, fade), "s", member.split("/")[-1])
    for name, bundle, member, dur, fade in z.SONNISS:
        if name in SHARED_SONNISS and (not only or name in only):
            print(name, z.process(z.extract(SB(bundle), member, "sonniss"), name, dur, fade, z.OUT / "sonniss"), "s", member.split("/")[-1][:70])
    for name, bundle, member, dur, fade, *skip in SONNISS:
        if not only or name in only:
            print(name, z.process(z.extract(SB(bundle), member, "sonniss"), name, dur, fade, z.OUT / "sonniss", skip=skip[0] if skip else 0), "s", member.split("/")[-1][:70])
