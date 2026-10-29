#!/usr/bin/env python3
"""scripts/repack-zombies-sons.py (S1-9). Usage : python3 scripts/repack-zombies-sons.py [nom ...]
Dépend de sox, oggenc et des archives de ~/Downloads (Snake's, Kenney, Sonniss) ; les extraits vont sous
~/orca/workspaces/alacod/packs/. Les .ogg de Snake's et Kenney sont commités ; ceux de Sonniss vont sous sounds/sonniss/ (gitignoré, en attente de la décision de William : dépôt public ou dépôt privé d'assets). Licences dans games/zombies/assets/assets.yaml.
S1-9 : extrait UN fichier à la fois des packs (sous ~/orca/workspaces/alacod/packs/), puis le traite
(mono 44,1 kHz, rognage, fondu de sortie, niveau RMS cible avec crête <= -1 dBFS, ogg vorbis q5) vers games/zombies/assets/sounds/."""
import subprocess, sys, tempfile, zipfile
from pathlib import Path
TMP=Path(tempfile.mkdtemp(prefix="zombies-sons-")); HOME=Path.home(); DL=HOME/"Downloads"; PACKS=HOME/"orca/workspaces/alacod/packs"
OUT=Path(__file__).resolve().parent.parent/"games/zombies/assets/sounds"
S1=DL/"Snake's Authentic Gun Sounds And More.zip"; S2=DL/"Snake's SECOND Authentic Gun Sounds.zip"
K=DL/"kenney_impact-sounds.zip"
SB=lambda n: DL/f"Sonniss.com-GDC2026-GameAudioBundle{n}of5.zip"
# (nom de sortie, zip, membre, pack (dossier), durée max s, fondu s)
ITEMS=[
 ("shot_pistol", S2, "Snake's SECOND Authentic Gun Sounds/Full Sounds/9mm/WAV/9mm Single.wav","snake",0.55,0.15),
 ("shot_smg",    S1, "Snake's Authentic Gun Sounds/Full Sound/5.56/WAV/556 Single WAV.wav","snake",0.45,0.12),
 ("shot_shotgun",S2, "Snake's SECOND Authentic Gun Sounds/Full Sounds/20 Gauge/WAV/20 Gauge Single.wav","snake",0.8,0.2),
 ("shot_rifle",  S2, "Snake's SECOND Authentic Gun Sounds/Full Sounds/.308 (7.62x51)/WAV/308 Single.wav","snake",0.8,0.2),
 ("reload_pistol",S2,"Snake's SECOND Authentic Gun Sounds/& More/Pistol/WAV/9mm Pistol Reload 1.wav","snake",1.6,0.1),
 ("reload_smg",  S1, "Snake's Authentic Gun Sounds/Reloads, Cycling & More/WAV/AR Reload Full WAV.wav","snake",2.4,0.15),
 ("reload_shotgun",S1,"Snake's Authentic Gun Sounds/Reloads, Cycling & More/WAV/Pump Reload Full Fast WAV.wav","snake",1.8,0.15),
 ("reload_rifle",S1, "Snake's Authentic Gun Sounds/Reloads, Cycling & More/WAV/308 Magazine Full WAV.wav","snake",2.2,0.15),
 ("window_break",K,  "Audio/impactPlank_medium_000.ogg","kenney-impact",1.2,0.1),
 ("window_repair",K, "Audio/impactWood_light_000.ogg","kenney-impact",0.8,0.1),
]
SONNISS=[
 ("death_zombie_1",2,"Epic Stock Media - Humanoid Creatures Vol 4 - Monstrous and Undead Creature Vocalization Sound Sets/VOXReac_Construction Kit Male Flutter Death Vocal Stuttered Long 05_ESM_HC4.wav",1.6,0.5),
 ("death_zombie_2",5,"SoundBits - Vox Bestiae - Source Elements/CREAEthr_Ethereal Entity Grim Pain Long 4_SNDBTS_VB-SE.wav",1.4,0.4),
 ("death_zombie_full",2,"Epic Stock Media - Humanoid Creatures Vol 4 - Monstrous and Undead Creature Vocalization Sound Sets/CREAMnstr_Designed Sea Beast Creature Pain Intense Yell Long 04_ESM_HC4.wav",2.0,0.6),
 ("boss_spawn",1,"344 Audio - Dinosaurs Vol. 1/ANMLRept_T Rex_344 Audio_Dinosaurs.wav",2.6,0.5),
 ("boss_death",2,"Epic Stock Media - Halloween Game - Haunted House and Horror Audio Scare Kit/CREABeast_Creature Werewolf Growl Menacing Monstrous 06_ESM_HALG.wav",2.4,0.6),
 ("death_player",2,"Epic Stock Media - AAA Game Character British Female Detective/VOXReac_British Detective Pain Vocal Female Dying High Grunt Breath_ESM_AAAGCBFD.wav",1.6,0.4),
 ("wave_start",3,"Jake Fielding - Cinematic Horn Braams/DSGNBram____Cinematic Horn Braam, Epic, Cinematic, Dark, Instrument, Huge-32.wav",3.0,1.0),
 ("wave_end",2,"Cinematic Sound Design - User Interface/Button Arp Twinkle.wav",1.2,0.3),
 ("room_cleared",2,"Cinematic Sound Design - User Interface/Interface Plucks Happy.wav",0.9,0.2),
 ("purchase",2,"Cinematic Sound Design - UI Interaction Elements/Ting Coins.wav",0.9,0.25),
 ("pickup",2,"Cinematic Sound Design - Hybrid Game & UI Elements/Game Entry Happy Short.wav",0.6,0.15),
 ("menu_navigate",2,"Cinematic Sound Design - Interface & Infographics/Interface Percussion Snap.wav",0.3,0.08),
 ("menu_confirm",2,"Cinematic Sound Design - UI Interaction Elements/Accept Boing Crunch.wav",0.5,0.12),
 ("menu_back",2,"Cinematic Sound Design - UI Interaction Elements/Deny Muted.wav",0.4,0.1),
]
def extract(zippath, member, pack):
    dest=PACKS/pack; dest.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(zippath) as z:
        z.extract(member, dest)
    return dest/member
TARGET={"reload_pistol":-27,"reload_smg":-27,"reload_shotgun":-27,"reload_rifle":-27,
        "death_zombie_1":-21,"death_zombie_2":-21,"death_zombie_full":-21,"boss_spawn":-18,"boss_death":-18,
        "wave_start":-20,"window_break":-24,"window_repair":-24}
def process(src, name, dur, fade, out=None):
    import wave, array, math
    out = out or OUT
    out.mkdir(parents=True,exist_ok=True)
    tmp=TMP/f"{name}.wav"; raw=TMP/f"{name}.raw.wav"
    src_len=float(subprocess.run(["soxi","-D",str(src)],capture_output=True,text=True).stdout)
    dur=min(dur,src_len); fade=min(fade,dur/2)
    subprocess.run(["sox",str(src),"-c","1","-r","44100","-b","16",str(raw),"trim","0",str(dur),"fade","t","0",str(dur),str(fade)],check=True)
    w=wave.open(str(raw)); a=array.array('h',w.readframes(w.getnframes()))
    rms=20*math.log10(math.sqrt(sum(v*v for v in a)/len(a))/32768+1e-9); peak=20*math.log10(max(map(abs,a))/32768+1e-9)
    gain=min(TARGET.get(name,-22)-rms, -1.0-peak)
    subprocess.run(["sox",str(raw),str(tmp),"gain",f"{gain:.2f}"],check=True)
    subprocess.run(["oggenc","-q","5","-Q","-o",str(out/f"{name}.ogg"),str(tmp)],check=True)
    w=wave.open(str(tmp)); a=array.array('h',w.readframes(w.getnframes())); sr=w.getframerate()
    lead=next((i for i,v in enumerate(a) if abs(v)>600),len(a))/sr
    r2=20*math.log10(math.sqrt(sum(v*v for v in a)/len(a))/32768+1e-9)
    return f"{len(a)/sr:.2f}s lead={lead*1000:.0f}ms rms={r2:.1f} peak={20*math.log10(max(map(abs,a))/32768+1e-9):.1f} gain={gain:+.1f}"

only=set(sys.argv[1:])
for name,zp,member,pack,dur,fade in ITEMS:
    if only and name not in only: continue
    print(name, process(extract(zp,member,pack),name,dur,fade),"s", member.split('/')[-1])
for name,n,member,dur,fade in SONNISS:
    if only and name not in only: continue
    print(name, process(extract(SB(n),member,"sonniss"),name,dur,fade,OUT/"sonniss"),"s", member.split('/')[-1][:70])
