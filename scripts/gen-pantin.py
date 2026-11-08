#!/usr/bin/env python3
"""Pantin neutre du testbed (R-1, docs/conventions.md § 111) : un modèle animé synthétique au format de
l'étape `banc` de la pipeline d'assets (`anime.glb` + `anime.json`), sans dépendance (Python seul).

Un corps de boîtes (une boîte par os, poids 1), un squelette (`root`, `hips`, `spine`, `chest`, `neck`,
`head`, bras et jambes `.L`/`.R`), une arme (`gun_hand`, tenue par `hand.R`, bouche `gun_muzzle`) et des
clips : allures en quatre directions (`walk`, `gun-walk`, `aim-walk`, `reload-walk` : avant, avant-gauche,
gauche, arrière-gauche ; l'engine joue l'opposé à l'envers), gestes en marchant dans les huit
(`shoot-walk`, `strike-walk`), gestes à l'arrêt (`shoot`, `shoulder`, `aim`, `reload`, `strike`), repos et
esquive (`roll`). Texture : une palette de couleurs en aplats.

Usage : scripts/gen-pantin.py [dossier]   (défaut : games/testbed/assets/models/pantin)
"""
import json
import math
import struct
import sys
import zlib
from pathlib import Path

FPS = 24
LEG = 0.87  # cuisse + jambe (m)
STRIDE_DEG = 30.0  # amplitude de la cuisse
CYCLE_S = 0.8  # deux pas
WALK_SPEED = 2 * 2 * LEG * math.sin(math.radians(STRIDE_DEG)) / CYCLE_S  # m/s, pieds sans glisser

# Os : nom -> (parent, position de repos relative au parent)
BONES = [
    ("root", None, (0.0, 0.0, 0.0)),
    ("hips", "root", (0.0, 0.95, 0.0)),
    ("spine", "hips", (0.0, 0.12, 0.0)),
    ("chest", "spine", (0.0, 0.25, 0.0)),
    ("neck", "chest", (0.0, 0.22, 0.0)),
    ("head", "neck", (0.0, 0.08, 0.0)),
    ("upper_arm.L", "chest", (0.21, 0.17, 0.0)),
    ("forearm.L", "upper_arm.L", (0.0, -0.28, 0.0)),
    ("hand.L", "forearm.L", (0.0, -0.25, 0.0)),
    ("upper_arm.R", "chest", (-0.21, 0.17, 0.0)),
    ("forearm.R", "upper_arm.R", (0.0, -0.28, 0.0)),
    ("hand.R", "forearm.R", (0.0, -0.25, 0.0)),
    ("thigh.L", "hips", (0.1, -0.02, 0.0)),
    ("shin.L", "thigh.L", (0.0, -0.45, 0.0)),
    ("foot.L", "shin.L", (0.0, -0.42, 0.0)),
    ("thigh.R", "hips", (-0.1, -0.02, 0.0)),
    ("shin.R", "thigh.R", (0.0, -0.45, 0.0)),
    ("foot.R", "shin.R", (0.0, -0.42, 0.0)),
]
INDEX = {name: i for i, (name, _, _) in enumerate(BONES)}

# Palette (une case de 4 × 4 pixels par couleur).
PALETTE = {
    "veste": (86, 104, 122),
    "peau": (214, 178, 142),
    "pantalon": (110, 92, 70),
    "bottes": (52, 46, 40),
    "metal": (70, 76, 84),
    "bois": (122, 84, 52),
}
CELLS = list(PALETTE)

# Boîtes : os, centre relatif à l'os, taille, couleur.
BOXES = [
    ("hips", (0.0, 0.03, 0.0), (0.32, 0.16, 0.2), "pantalon"),
    ("spine", (0.0, 0.13, 0.0), (0.34, 0.26, 0.2), "veste"),
    ("chest", (0.0, 0.11, 0.0), (0.4, 0.24, 0.22), "veste"),
    ("neck", (0.0, 0.03, 0.0), (0.09, 0.08, 0.09), "peau"),
    ("head", (0.0, 0.12, 0.01), (0.2, 0.24, 0.22), "peau"),
    ("upper_arm.L", (0.0, -0.14, 0.0), (0.09, 0.28, 0.09), "veste"),
    ("forearm.L", (0.0, -0.12, 0.0), (0.08, 0.25, 0.08), "veste"),
    ("hand.L", (0.0, -0.05, 0.0), (0.07, 0.1, 0.05), "peau"),
    ("upper_arm.R", (0.0, -0.14, 0.0), (0.09, 0.28, 0.09), "veste"),
    ("forearm.R", (0.0, -0.12, 0.0), (0.08, 0.25, 0.08), "veste"),
    ("hand.R", (0.0, -0.05, 0.0), (0.07, 0.1, 0.05), "peau"),
    ("thigh.L", (0.0, -0.22, 0.0), (0.13, 0.45, 0.13), "pantalon"),
    ("shin.L", (0.0, -0.21, 0.0), (0.11, 0.42, 0.11), "pantalon"),
    ("foot.L", (0.0, -0.03, 0.05), (0.1, 0.06, 0.24), "bottes"),
    ("thigh.R", (0.0, -0.22, 0.0), (0.13, 0.45, 0.13), "pantalon"),
    ("shin.R", (0.0, -0.21, 0.0), (0.11, 0.42, 0.11), "pantalon"),
    ("foot.R", (0.0, -0.03, 0.05), (0.1, 0.06, 0.24), "bottes"),
]
# L'arme, dans le repère de la main droite : le long de -Y (l'axe de l'avant-bras).
GUN_BOXES = [
    ((0.0, -0.3, 0.0), (0.05, 0.5, 0.06), "metal"),
    ((0.0, -0.08, 0.03), (0.05, 0.18, 0.08), "bois"),
]
GUN_MUZZLE = (0.0, -0.57, 0.0)


# --- Quaternions -------------------------------------------------------------------------------
def quat(axis, angle):
    x, y, z = axis
    n = math.sqrt(x * x + y * y + z * z) or 1.0
    s = math.sin(angle / 2) / n
    return (x * s, y * s, z * s, math.cos(angle / 2))


def qmul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return (
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    )


ID = (0.0, 0.0, 0.0, 1.0)


def swing(direction_deg, angle_deg):
    """Le membre (vers -Y au repos) part vers la direction `direction_deg` du corps (0 : devant, +Z ;
    90 : sa gauche, +X) d'un angle `angle_deg`."""
    d = math.radians(direction_deg)
    return quat((-math.cos(d), 0.0, math.sin(d)), math.radians(angle_deg))


def twist(angle_deg):
    return quat((0.0, 1.0, 0.0), math.radians(angle_deg))


def smooth(u):
    u = min(1.0, max(0.0, u))
    return u * u * (3 - 2 * u)


# --- Poses : dictionnaires os -> quaternion (os absents : repos) ; "hips_y" : hauteur du bassin ---
def legs_walk(direction, phase):
    s = math.sin(2 * math.pi * phase)
    c = math.cos(2 * math.pi * phase)
    a = STRIDE_DEG
    return {
        "thigh.L": swing(direction, a * s),
        "thigh.R": swing(direction, -a * s),
        "shin.L": swing(direction, -40 * max(0.0, -c)),
        "shin.R": swing(direction, -40 * max(0.0, c)),
        "hips_y": 0.95 - 0.03 * abs(c),
    }


def arms_swing(phase):
    s = math.sin(2 * math.pi * phase)
    return {
        "upper_arm.L": swing(0, -20 * s),
        "upper_arm.R": swing(0, 20 * s),
        "forearm.L": swing(0, 15),
        "forearm.R": swing(0, 15),
    }


def arms_carry():
    return {
        "upper_arm.R": swing(0, 20),
        "forearm.R": swing(0, 55),
        "upper_arm.L": swing(-20, 35),
        "forearm.L": swing(0, 50),
    }


def arms_aim(raise_deg=0.0):
    return {
        "upper_arm.R": swing(0, 88 + raise_deg),
        "upper_arm.L": qmul(swing(-30, 75), twist(-10)),
        "forearm.L": swing(0, 25),
        "chest": twist(8),
    }


def blend_pose(a, b, u):
    out = {}
    for key in set(a) | set(b):
        if key == "hips_y":
            out[key] = a.get(key, 0.95) * (1 - u) + b.get(key, 0.95) * u
        else:
            out[key] = slerp(a.get(key, ID), b.get(key, ID), u)
    return out


def slerp(p, q, u):
    dot = sum(x * y for x, y in zip(p, q))
    if dot < 0:
        q, dot = tuple(-x for x in q), -dot
    if dot > 0.9995:
        r = tuple(x + (y - x) * u for x, y in zip(p, q))
    else:
        th = math.acos(dot)
        s = math.sin(th)
        r = tuple((math.sin((1 - u) * th) * x + math.sin(u * th) * y) / s for x, y in zip(p, q))
    n = math.sqrt(sum(x * x for x in r))
    return tuple(x / n for x in r)


def upper_idle(t):
    return {"chest": swing(0, 2 * math.sin(2 * math.pi * t / 2.0))}


def upper_shoot(u):
    kick = 18 * math.exp(-10 * u) * min(1.0, u * 12)
    pose = arms_aim(kick)
    pose["chest"] = qmul(twist(8), swing(180, 6 * math.exp(-8 * u) * min(1.0, u * 12)))
    return pose


def upper_reload(u):
    # Arme baissée, main gauche qui va chercher sous l'arme puis revient.
    reach = math.sin(math.pi * smooth(u))
    return {
        "upper_arm.R": swing(0, 45),
        "forearm.R": swing(0, 40 + 20 * reach),
        "upper_arm.L": qmul(swing(-35, 30 + 30 * reach), twist(-20)),
        "forearm.L": swing(0, 40 + 50 * reach),
        "chest": swing(0, 6 * reach),
        "head": swing(0, 15 * reach),
    }


def upper_strike(u):
    # Armé derrière, frappe en travers vers l'avant.
    wind = smooth(u / 0.4)
    hit = smooth((u - 0.4) / 0.35)
    angle = -50 * wind + (50 + 140) * hit
    return {
        "upper_arm.R": swing(-30, angle),
        "forearm.R": swing(0, 30 * (1 - hit)),
        "chest": twist(-25 * wind + 45 * hit),
        "upper_arm.L": swing(0, 25),
    }


def pose_roll(u):
    s = smooth(u)
    tuck = math.sin(math.pi * s)
    return {
        "hips": swing(0, -360 * s) if s < 1 else ID,
        "hips_y": 0.95 - 0.45 * tuck,
        "thigh.L": swing(0, 100 * tuck),
        "thigh.R": swing(0, 100 * tuck),
        "shin.L": swing(0, -130 * tuck),
        "shin.R": swing(0, -130 * tuck),
        "upper_arm.L": swing(0, 60 * tuck),
        "upper_arm.R": swing(0, 60 * tuck),
        "forearm.L": swing(0, 90 * tuck),
        "forearm.R": swing(0, 90 * tuck),
        "neck": swing(0, 30 * tuck),
    }


DIRS4 = [(0, ""), (45, "-fl"), (90, "-l"), (135, "-bl")]
DIRS8 = DIRS4 + [(180, "-b"), (-135, "-br"), (-90, "-r"), (-45, "-fr")]


def clips():
    """Nom -> (durée s, fonction t -> pose, moment)."""
    out = {}
    out["idle"] = (2.0, upper_idle, {})
    out["gun-idle"] = (2.0, lambda t: {**arms_carry(), **upper_idle(t)}, {})
    out["aim"] = (2.0, lambda t: {**arms_aim(), "chest": qmul(twist(8), swing(0, 1.5 * math.sin(math.pi * t)))}, {})
    out["shoot"] = (0.3, lambda t: upper_shoot(t / 0.3), {})
    out["shoulder"] = (0.15, lambda t: blend_pose(arms_carry(), arms_aim(), smooth(t / 0.15)), {})
    out["reload"] = (1.2, lambda t: upper_reload(t / 1.2), {})
    # M-6 (§ 116) : la frappe du coup tombe entre 45 % et 65 % du clip (`moments.strike.frappe`, fractions).
    out["strike"] = (0.35, lambda t: upper_strike(t / 0.35), {"frappe": [0.45, 0.65]})
    out["roll"] = (0.4, lambda t: pose_roll(t / 0.4), {})
    moving = {"vitesse": round(WALK_SPEED, 3)}
    for d, suffix in DIRS4:
        out["walk" + suffix] = (CYCLE_S, lambda t, d=d: {**legs_walk(d, t / CYCLE_S), **arms_swing(t / CYCLE_S)}, {**moving, "direction": d})
        out["gun-walk" + suffix] = (CYCLE_S, lambda t, d=d: {**legs_walk(d, t / CYCLE_S), **arms_carry()}, {**moving, "direction": d})
        out["aim-walk" + suffix] = (CYCLE_S, lambda t, d=d: {**legs_walk(d, t / CYCLE_S), **arms_aim()}, {**moving, "direction": d})
        out["reload-walk" + suffix] = (CYCLE_S, lambda t, d=d: {**legs_walk(d, t / CYCLE_S), **upper_reload((t / CYCLE_S) % 1.0)}, {**moving, "direction": d})
    for d, suffix in DIRS8:
        out["shoot-walk" + suffix] = (0.3, lambda t, d=d: {**legs_walk(d, t / CYCLE_S), **upper_shoot(t / 0.3)}, {**moving, "direction": d})
        out["strike-walk" + suffix] = (0.35, lambda t, d=d: {**legs_walk(d, t / CYCLE_S), **upper_strike(t / 0.35)}, {**moving, "direction": d})
    return out


# --- glTF -------------------------------------------------------------------------------------------
class Buffer:
    def __init__(self):
        self.data = bytearray()
        self.views = []
        self.accessors = []

    def add(self, raw, target=None):
        while len(self.data) % 4:
            self.data.append(0)
        view = {"buffer": 0, "byteOffset": len(self.data), "byteLength": len(raw)}
        if target:
            view["target"] = target
        self.data.extend(raw)
        self.views.append(view)
        return len(self.views) - 1

    def accessor(self, values, kind, component, fmt, target=None, minmax=False):
        flat = [x for v in values for x in (v if isinstance(v, (tuple, list)) else (v,))]
        raw = struct.pack("<" + fmt * len(flat), *flat)
        acc = {"bufferView": self.add(raw, target), "componentType": component, "count": len(values), "type": kind}
        if minmax:
            width = len(values[0]) if isinstance(values[0], (tuple, list)) else 1
            cols = [[v[i] if width > 1 else v for v in values] for i in range(width)]
            acc["min"] = [min(c) for c in cols]
            acc["max"] = [max(c) for c in cols]
        self.accessors.append(acc)
        return len(self.accessors) - 1


def rest_global():
    out = {}
    for name, parent, pos in BONES:
        base = out[parent] if parent else (0.0, 0.0, 0.0)
        out[name] = tuple(b + p for b, p in zip(base, pos))
    return out


def uv_of(color):
    i = CELLS.index(color)
    return ((i * 4 + 2) / (len(CELLS) * 4), 0.5)


FACES = [  # normale, deux axes du plan
    ((1, 0, 0), (0, 1, 0), (0, 0, 1)),
    ((-1, 0, 0), (0, 0, 1), (0, 1, 0)),
    ((0, 1, 0), (0, 0, 1), (1, 0, 0)),
    ((0, -1, 0), (1, 0, 0), (0, 0, 1)),
    ((0, 0, 1), (1, 0, 0), (0, 1, 0)),
    ((0, 0, -1), (0, 1, 0), (1, 0, 0)),
]


def box(center, size, color, joint=None):
    verts = []
    idx = []
    for n, a, b in FACES:
        base = len(verts)
        for sa, sb in [(-1, -1), (1, -1), (1, 1), (-1, 1)]:
            p = tuple(
                center[k] + (n[k] * 0.5 + a[k] * sa * 0.5 + b[k] * sb * 0.5) * size[k] for k in range(3)
            )
            verts.append((p, n, uv_of(color), joint))
        idx += [base, base + 1, base + 2, base, base + 2, base + 3]
    return verts, idx


def png(width, height, rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rows)

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def main():
    out_dir = Path(sys.argv[1] if len(sys.argv) > 1 else "games/testbed/assets/models/pantin")
    out_dir.mkdir(parents=True, exist_ok=True)
    rest = rest_global()
    buf = Buffer()

    # Corps : boîtes en pose de repos (repère du modèle), une articulation par boîte.
    verts, idx = [], []
    for bone, center, size, color in BOXES:
        c = tuple(r + o for r, o in zip(rest[bone], center))
        v, i = box(c, size, color, INDEX[bone])
        idx += [k + len(verts) for k in i]
        verts += v
    body = {
        "POSITION": buf.accessor([v[0] for v in verts], "VEC3", 5126, "f", 34962, minmax=True),
        "NORMAL": buf.accessor([v[1] for v in verts], "VEC3", 5126, "f", 34962),
        "TEXCOORD_0": buf.accessor([v[2] for v in verts], "VEC2", 5126, "f", 34962),
        "JOINTS_0": buf.accessor([(v[3], 0, 0, 0) for v in verts], "VEC4", 5123, "H", 34962),
        "WEIGHTS_0": buf.accessor([(1.0, 0.0, 0.0, 0.0) for _ in verts], "VEC4", 5126, "f", 34962),
    }
    body_idx = buf.accessor(idx, "SCALAR", 5123, "H", 34963)

    gverts, gidx = [], []
    for center, size, color in GUN_BOXES:
        v, i = box(center, size, color)
        gidx += [k + len(gverts) for k in i]
        gverts += v
    gun = {
        "POSITION": buf.accessor([v[0] for v in gverts], "VEC3", 5126, "f", 34962, minmax=True),
        "NORMAL": buf.accessor([v[1] for v in gverts], "VEC3", 5126, "f", 34962),
        "TEXCOORD_0": buf.accessor([v[2] for v in gverts], "VEC2", 5126, "f", 34962),
    }
    gun_idx = buf.accessor(gidx, "SCALAR", 5123, "H", 34963)

    inverse_bind = []
    for name, _, _ in BONES:
        x, y, z = rest[name]
        inverse_bind.append((1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, -x, -y, -z, 1))
    ibm = buf.accessor(inverse_bind, "MAT4", 5126, "f")

    # Texture : la palette en aplats.
    rows = []
    for _ in range(4):
        row = []
        for color in CELLS:
            row += list(PALETTE[color]) * 4
        rows.append(row)
    image = png(len(CELLS) * 4, 4, rows)
    image_view = buf.add(image)

    # Nœuds : os (0..n-1), arme, bouche, maillage du corps, armature.
    nodes = []
    for name, parent, pos in BONES:
        nodes.append({"name": name, "translation": list(pos), "children": []})
    for i, (name, parent, _) in enumerate(BONES):
        if parent:
            nodes[INDEX[parent]]["children"].append(i)
    gun_node = len(nodes)
    nodes.append({"name": "gun_hand", "mesh": 1, "translation": [0.0, -0.05, 0.0], "children": [gun_node + 1]})
    nodes.append({"name": "gun_muzzle", "translation": list(GUN_MUZZLE)})
    nodes[INDEX["hand.R"]]["children"].append(gun_node)
    body_node = len(nodes)
    nodes.append({"name": "body", "mesh": 0, "skin": 0})
    rig = len(nodes)
    nodes.append({"name": "Rig", "children": [INDEX["root"], body_node]})
    for n in nodes:
        if not n.get("children"):
            n.pop("children", None)

    # Animations : rotations de chaque os, hauteur du bassin, à 24 images/s.
    animations = []
    manifest_clips, moments = {}, {}
    gun_clips = []
    for name, (duration, pose_fn, moment) in clips().items():
        frames = max(2, round(duration * FPS) + 1)
        times = [min(duration, k / FPS) for k in range(frames)]
        times[-1] = duration
        poses = [pose_fn(t) for t in times]
        t_acc = buf.accessor(times, "SCALAR", 5126, "f", minmax=True)
        samplers, channels = [], []
        for bone, _, pos in BONES:
            rot = [p.get(bone, ID) for p in poses]
            samplers.append({"input": t_acc, "output": buf.accessor(rot, "VEC4", 5126, "f"), "interpolation": "LINEAR"})
            channels.append({"sampler": len(samplers) - 1, "target": {"node": INDEX[bone], "path": "rotation"}})
        hips = [(0.0, p.get("hips_y", 0.95), 0.0) for p in poses]
        samplers.append({"input": t_acc, "output": buf.accessor(hips, "VEC3", 5126, "f"), "interpolation": "LINEAR"})
        channels.append({"sampler": len(samplers) - 1, "target": {"node": INDEX["hips"], "path": "translation"}})
        animations.append({"name": name, "samplers": samplers, "channels": channels})
        manifest_clips[name] = [1.0, 1.0 + duration * FPS]
        if moment:
            moments[name] = moment
        if not name.startswith(("idle", "walk", "strike", "roll")):
            gun_clips.append(name)

    gltf = {
        "asset": {"version": "2.0", "generator": "alacod scripts/gen-pantin.py"},
        "scene": 0,
        "scenes": [{"nodes": [rig]}],
        "nodes": nodes,
        "meshes": [
            {"name": "body", "primitives": [{"attributes": body, "indices": body_idx, "material": 0}]},
            {"name": "gun", "primitives": [{"attributes": gun, "indices": gun_idx, "material": 0}]},
        ],
        "skins": [{"joints": list(range(len(BONES))), "inverseBindMatrices": ibm, "skeleton": INDEX["root"]}],
        "materials": [{"name": "palette", "pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0.0}}],
        "textures": [{"source": 0, "sampler": 0}],
        "samplers": [{"magFilter": 9728, "minFilter": 9728}],
        "images": [{"bufferView": image_view, "mimeType": "image/png"}],
        "animations": animations,
        "accessors": buf.accessors,
        "bufferViews": buf.views,
        "buffers": [{"byteLength": len(buf.data)}],
    }
    text = json.dumps(gltf, separators=(",", ":")).encode()
    text += b" " * (-len(text) % 4)
    while len(buf.data) % 4:
        buf.data.append(0)
    glb = (
        struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(text) + 8 + len(buf.data))
        + struct.pack("<II", len(text), 0x4E4F534A)
        + text
        + struct.pack("<II", len(buf.data), 0x004E4942)
        + bytes(buf.data)
    )
    (out_dir / "anime.glb").write_bytes(glb)

    manifest = {
        "fps": FPS,
        "actions": manifest_clips,
        "armes": {
            "gun": {
                "objets": {"main": "gun_hand"},
                "animations": gun_clips,
                "reperes": {"main": {"muzzle": "gun_muzzle"}},
            }
        },
        "prises": {name: "une-main" for name in manifest_clips},
        "moments": moments,
    }
    (out_dir / "anime.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n")
    print(f"{out_dir}/anime.glb : {len(glb)} octets, {len(animations)} clips ; marche {WALK_SPEED:.2f} m/s")


if __name__ == "__main__":
    main()
