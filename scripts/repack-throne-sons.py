#!/usr/bin/env python3
"""scripts/repack-throne-sons.py (S2-11, docs/conventions.md § 98). Usage : python3 scripts/repack-throne-sons.py [nom ...]
Script frère de scripts/repack-zombies-sons.py (importé : `process`, `extract`, `TARGET`, `OUT`), mêmes archives de ~/Downloads (Snake's, Kenney,
Sonniss), mêmes règles : un fichier extrait à la fois sous ~/orca/workspaces/alacod/packs/, jamais un bundle entier ; mono 44,1 kHz, rognage, fondu,
niveau RMS cible, crête <= -1 dBFS, ogg vorbis q5.
Les .ogg de Snake's et Kenney vont sous games/throne/assets/sounds/ (commités, repli public) ; ceux de Sonniss sous sounds/sonniss/ (gitignoré,
jamais publiés : surcharge locale ui/feedback.local.ron, dépôt privé d'assets). Licences dans games/throne/assets/assets.yaml.
Les armes sans son libre dédié (lanceurs, énergie) reprennent un tir Snake's changé de hauteur (`pitch`, centièmes de ton) : repli public choisi sans écoute."""
import importlib.util, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("repack_zombies_sons", HERE / "repack-zombies-sons.py")
z = importlib.util.module_from_spec(spec)
spec.loader.exec_module(z)
z.OUT = HERE.parent / "games/throne/assets/sounds"
S1, S2, K, SB = z.S1, z.S2, z.K, z.SB

SN1 = "Snake's Authentic Gun Sounds/"
SN2 = "Snake's SECOND Authentic Gun Sounds/"
RELOADS = SN1 + "Reloads, Cycling & More/WAV/"
# (nom de sortie, zip, membre, pack, durée max s, fondu s, hauteur en centièmes de ton)
SHOTS = [
    # Balles : revolver, mitraillette, fusil à pompe, canon à ricochets.
    ("shot_revolver", S2, SN2 + "Full Sounds/9mm/WAV/9mm Single.wav", "snake", 0.6, 0.15, -300),
    ("shot_mitraillette", S1, SN1 + "Full Sound/5.56/WAV/556 Single WAV.wav", "snake", 0.45, 0.12, 0),
    ("shot_fusil_a_pompe", S2, SN2 + "Full Sounds/20 Gauge/WAV/20 Gauge Single.wav", "snake", 0.8, 0.2, 0),
    ("shot_canon_ricochet", S1, SN1 + "Full Sound/7.62x39/WAV/762x39 Single WAV.wav", "snake", 0.7, 0.18, 0),
    # Obus : un tir de fusil changé de hauteur (plus grave : plus lourd).
    ("shot_lance_grenades", S2, SN2 + "Full Sounds/20 Gauge/WAV/20 Gauge Single.wav", "snake", 0.7, 0.2, -800),
    ("shot_roquette", S1, SN1 + "Full Sound/7.62x54R/WAV/762x54r Single WAV.wav", "snake", 0.9, 0.25, -500),
    ("shot_mortier", S1, SN1 + "Full Sound/7.62x54R/WAV/762x54r Single WAV.wav", "snake", 1.0, 0.3, -1100),
    # Énergie : un .22 LR plus aigu (laser) ou plus gras (plasma), une rafale double (traqueur).
    ("shot_laser", S1, SN1 + "Full Sound/.22LR/WAV/22LR Single WAV.wav", "snake", 0.3, 0.1, 1000),
    ("shot_plasma", S1, SN1 + "Full Sound/.22LR/WAV/22LR Single WAV.wav", "snake", 0.5, 0.15, -400),
    ("shot_traqueur", S1, SN1 + "Full Sound/.22LR/WAV/22LR Double Tap WAV.wav", "snake", 0.5, 0.12, 400),
    # Lames : impacts métalliques de Kenney.
    ("shot_lance_lames", K, "Audio/impactMetal_light_001.ogg", "kenney-impact", 0.25, 0.08, 0),
    ("shot_disque", K, "Audio/impactPlate_light_003.ogg", "kenney-impact", 0.5, 0.12, 0),
    # Explosion : un tir très grave.
    ("explosion", S1, SN1 + "Full Sound/7.62x54R/WAV/762x54r Single WAV.wav", "snake", 1.1, 0.35, -1200),
]
RELOAD_ITEMS = [
    ("reload_revolver", S2, SN2 + "& More/Revolver/44 Magnum/WAV/44 Magnum Single Load.wav", "snake", 1.2, 0.1, 0),
    ("reload_mitraillette", S1, RELOADS + "AR Reload Full WAV.wav", "snake", 2.4, 0.15, 0),
    ("reload_canon", S1, RELOADS + "AK Reload Full WAV.wav", "snake", 2.4, 0.15, 0),
    ("reload_fusil_a_pompe", S1, RELOADS + "Pump Reload Full Fast WAV.wav", "snake", 1.8, 0.15, 0),
    ("reload_lanceur", S1, RELOADS + "Single Shot Reload Full WAV.wav", "snake", 1.8, 0.15, 0),
    ("reload_energie", S1, RELOADS + "AR Charging Handle WAV.wav", "snake", 0.8, 0.1, 0),
    ("reload_lames", S1, RELOADS + "Lever Reload WAV.wav", "snake", 1.8, 0.15, 0),
]
# Événements, replis publics (Kenney impact, CC0) : choisis sur le nom et la durée, sans écoute.
KENNEY = [
    ("death_machine", "Audio/impactMetal_heavy_003.ogg", 0.3, 0.1),
    ("boss_phase", "Audio/impactMining_001.ogg", 0.7, 0.2),
    ("floor", "Audio/impactPlate_medium_003.ogg", 0.65, 0.2),
    ("bomb", "Audio/impactTin_medium_001.ogg", 0.18, 0.05),
    ("blank", "Audio/impactGlass_heavy_000.ogg", 0.25, 0.08),
]
# Mêmes événements qu'à zombies (même fichier, même traitement) : repris de scripts/repack-zombies-sons.py.
SHARED_PUBLIC = ["death_player", "boss_spawn", "boss_death", "room_cleared", "purchase", "pickup", "menu_navigate", "menu_confirm", "menu_back"]
SHARED_SONNISS = ["death_player", "room_cleared", "purchase", "pickup", "menu_navigate", "menu_confirm", "menu_back"]
# Sonniss propres à throne : (nom, bundle, membre, durée, fondu).
SONNISS = [
    ("boss_spawn", 1, "344 Audio - Dinosaurs Vol. 1/ANMLRept_T Rex_344 Audio_Dinosaurs.wav", 2.6, 0.5),
    ("boss_death", 2, "Epic Stock Media - Halloween Game - Haunted House and Horror Audio Scare Kit/CREABeast_Creature Werewolf Growl Menacing Monstrous 06_ESM_HALG.wav", 2.4, 0.6),
    ("boss_phase", 2, "Epic Stock Media - Halloween Game - Haunted House and Horror Audio Scare Kit/DSGNEthr_Jumpscare Vocal Aggressive Whisper Harsh Distortion 02_ESM_HALG.wav", 1.2, 0.3),
    ("floor", 2, "Cinematic Sound Design - Ultra Transitions & Impacts/Transition Braam Slow Dark Creepy.wav", 2.0, 0.6),
    ("explosion", 2, "Epic Stock Media - Anime Game/EXPLDsgn_Explosion Small Blast Enemy Death Crunchy Boom Cartoon Noisy Crash Impact Delay 03_ESM_AG.wav", 1.4, 0.4),
    ("bomb", 2, "Cinematic Sound Design - Interface & Infographics/Interface Pop High Short.wav", 0.3, 0.08),
    ("blank", 3, "InMotionAudio - Arc/ELECArc_ArcPowerUpDesign04_InMotionAudio_Arc.wav", 1.0, 0.3),
    # Mort par famille : rongeurs (rat, roi_rat), bêtes (chien, buffle), humains (pillard, franc_tireur, rodeur, cracheur), brute, machines.
    ("death_rongeur", 1, "344 Audio - Dinosaurs Vol. 1/ANMLRept_Hatchling Calling Out_344 Audio_Dinosaurs.wav", 1.0, 0.3),
    ("death_bete", 1, "344 Audio - Dinosaurs Vol. 2/ANMLRept_Large Herbivore Roar 01_344 Audio_Dinosaurs Vol 2.wav", 1.6, 0.5),
    ("death_humain", 2, "Epic Stock Media - Humanoid Creatures Vol 4 - Monstrous and Undead Creature Vocalization Sound Sets/VOXReac_Construction Kit Male Flutter Death Vocal Stuttered Long 05_ESM_HC4.wav", 1.6, 0.5),
    ("death_brute", 2, "Epic Stock Media - Humanoid Creatures Vol 4 - Monstrous and Undead Creature Vocalization Sound Sets/CREAHmn_Designed Orc Male Attack Long Heavy Hit Charged Up 03_ESM_HC4.wav", 1.6, 0.5),
    ("death_machine", 2, "Epic Stock Media - Tower Defense Game/ROBTMvmt_Tower Deploy Hitech Robot Motor Dark Thump Servo Whine 04_ESM_TDG.wav", 1.2, 0.4),
]
# RMS cibles : lanceurs un peu plus forts (le volume d'un tir est celui de la clé de sa famille, `shot_pistol`), énergie un peu plus bas.
z.TARGET.update({
    "shot_lance_grenades": -20, "shot_roquette": -20, "shot_mortier": -19, "explosion": -18,
    "shot_laser": -24, "shot_plasma": -23, "shot_traqueur": -24,
    "reload_revolver": -27, "reload_mitraillette": -27, "reload_canon": -27, "reload_fusil_a_pompe": -27,
    "reload_lanceur": -27, "reload_energie": -27, "reload_lames": -27,
    "death_rongeur": -21, "death_bete": -21, "death_humain": -21, "death_brute": -21, "death_machine": -22,
    "boss_phase": -20, "floor": -20, "blank": -22,
})

if __name__ == "__main__":
    only = set(sys.argv[1:])
    for name, zp, member, pack, dur, fade, pitch in SHOTS + RELOAD_ITEMS:
        if only and name not in only:
            continue
        print(name, z.process(z.extract(zp, member, pack), name, dur, fade, pitch=pitch), "s", member.split("/")[-1])
    for name, member, dur, fade in KENNEY:
        if only and name not in only:
            continue
        print(name, z.process(z.extract(K, member, "kenney-impact"), name, dur, fade), "s", member.split("/")[-1])
    for name, zp, member, pack, dur, fade in z.ITEMS:
        if name in SHARED_PUBLIC and (not only or name in only):
            print(name, z.process(z.extract(zp, member, pack), name, dur, fade), "s", member.split("/")[-1])
    for name, bundle, member, dur, fade in z.SONNISS:
        if name in SHARED_SONNISS and (not only or name in only):
            print(name, z.process(z.extract(SB(bundle), member, "sonniss"), name, dur, fade, z.OUT / "sonniss"), "s", member.split("/")[-1][:70])
    for name, bundle, member, dur, fade in SONNISS:
        if not only or name in only:
            print(name, z.process(z.extract(SB(bundle), member, "sonniss"), name, dur, fade, z.OUT / "sonniss"), "s", member.split("/")[-1][:70])
