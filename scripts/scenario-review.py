#!/usr/bin/env python3
"""Génère target/videos/index.html : page de revue des vidéos de scénarios.

Appelé par scripts/scenario-video après chaque rendu. Liste les commits rendus
(target/videos/<commit>/), leurs vidéos et montage, et les comparaisons
(target/videos/compare/). La page est statique : les données sont intégrées en JSON,
les vidéos référencées par chemin relatif.

`--serve [port] [--bind <adresse>]` : sert target/videos (défaut 8766, sur 127.0.0.1) avec
les requêtes HTTP Range, sans lesquelles le navigateur ne peut pas se positionner dans une
vidéo (`make review_videos`, `TAILSCALE=1` pour l'IP Tailscale de la machine).
"""
import functools
import http.server
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VIDEOS = ROOT / "target" / "videos"
METRICS = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "metrics"
SCENARIOS = ROOT / "tests" / "scenarios"


def git(*args):
    try:
        return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True).stdout.strip()
    except subprocess.CalledProcessError:
        return ""


def scenario_info(name):
    """Description et « À regarder » (commentaires en tête du .ron), attentes du scénario."""
    path = SCENARIOS / f"{name}.ron"
    if not path.exists():
        return {"description": "", "watch": "", "expect": [], "frames": None}
    text = path.read_text()
    description, watch = [], []
    for line in text.splitlines():
        if not line.startswith("//"):
            if line.strip():
                break
            continue
        content = line.lstrip("/").strip()
        if content.startswith("À regarder"):
            watch.append(content.split(":", 1)[1].strip() if ":" in content else "")
        elif watch:
            watch.append(content)
        elif content:
            description.append(content)
    expect = re.findall(r"^\s*((?:PlayerAlive|PlayerDead|WaveAtLeast|KillsAtLeast|WindowsBrokenAtLeast|ActiveWeapon|Ammo|DoorsOpenAtLeast|PlayerPosition|WindowHealth|BulletsInside)\([^)]*\))", text, re.M)
    frames = re.search(r"frames:\s*(\d+)", text)
    return {
        "description": " ".join(description),
        "watch": " ".join(w for w in watch if w),
        "expect": expect,
        "frames": int(frames.group(1)) if frames else None,
    }


def load_events(path):
    """Moments clés écrits par la capture (play_scenario), s'ils existent."""
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError):
        return []


def commit_info(dirname):
    sha = dirname.removesuffix("-dirty")
    subject = git("log", "-1", "--format=%s", sha)
    date = git("log", "-1", "--format=%cs", sha)
    return {"subject": subject, "date": date, "dirty": dirname.endswith("-dirty")}


def load_metrics(path):
    """Charge les métriques depuis un fichier JSON."""
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError):
        return {}


def main():
    commits = []
    for d in sorted((p for p in VIDEOS.iterdir() if p.is_dir()), key=lambda p: p.stat().st_mtime, reverse=True):
        if d.name in ("compare", "worktrees", "test") or d.name.startswith("frames"):
            continue
        videos = sorted(v.stem for v in d.glob("*.mp4") if v.stem != "montage")
        if not videos:
            continue

        # Load metrics for this commit
        metrics_path = METRICS / d.name / "metrics.json"
        metrics_data = load_metrics(metrics_path)

        commits.append({
            "id": d.name,
            **commit_info(d.name),
            "montage": (d / "montage.mp4").exists(),
            "metrics": metrics_data,
            "videos": [
                {"name": v, **scenario_info(v), "events": load_events(d / f"{v}.events.json")}
                for v in videos
            ],
        })

    compares = []
    compare_dir = VIDEOS / "compare"
    if compare_dir.exists():
        for v in sorted(compare_dir.glob("*_vs_*.mp4"), key=lambda p: p.stat().st_mtime, reverse=True):
            m = re.match(r"(.+)_([0-9a-f]+)_vs_([0-9a-f]+)$", v.stem)
            if not m:
                continue
            name, base, head = m.groups()
            compares.append({
                "file": f"compare/{v.name}",
                "scenario": name,
                "base": base,
                "head": head,
                "base_subject": git("log", "-1", "--format=%s", base),
                "head_subject": git("log", "-1", "--format=%s", head),
                **scenario_info(name),
                "base_events": load_events(v.with_suffix(".base.events.json")),
                "head_events": load_events(v.with_suffix(".head.events.json")),
            })

    data = {"commits": commits, "compares": compares, "branch": git("rev-parse", "--abbrev-ref", "HEAD")}
    template = (Path(__file__).resolve().parent / "scenario-review.html").read_text()
    html = template.replace("/*__DATA__*/null", json.dumps(data, ensure_ascii=False))
    (VIDEOS / "index.html").write_text(html)
    print(f"page de revue : {VIDEOS / 'index.html'}")


class RangeHandler(http.server.SimpleHTTPRequestHandler):
    """Fichiers statiques avec support de `Range: bytes=a-b` (lecture et positionnement vidéo)."""

    def send_head(self):
        range_header = self.headers.get("Range")
        path = Path(self.translate_path(self.path))
        if not range_header or not path.is_file():
            return super().send_head()
        match = re.match(r"bytes=(\d*)-(\d*)$", range_header.strip())
        size = path.stat().st_size
        if not match:
            self.send_error(416)
            return None
        start = int(match.group(1)) if match.group(1) else max(size - int(match.group(2) or 0), 0)
        end = int(match.group(2)) if match.group(1) and match.group(2) else size - 1
        end = min(end, size - 1)
        if start > end:
            self.send_error(416)
            return None
        f = path.open("rb")
        f.seek(start)
        self.send_response(206)
        self.send_header("Content-Type", self.guess_type(str(path)))
        self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
        self.send_header("Content-Length", str(end - start + 1))
        self.end_headers()
        self._range_remaining = end - start + 1
        return f

    def copyfile(self, source, outputfile):
        remaining = getattr(self, "_range_remaining", None)
        if remaining is None:
            return super().copyfile(source, outputfile)
        while remaining > 0:
            chunk = source.read(min(64 * 1024, remaining))
            if not chunk:
                break
            outputfile.write(chunk)
            remaining -= len(chunk)
        self._range_remaining = None

    def end_headers(self):
        self.send_header("Accept-Ranges", "bytes")
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()

    def log_message(self, *args):
        pass


def serve(port, bind):
    handler = functools.partial(RangeHandler, directory=str(VIDEOS))
    with http.server.ThreadingHTTPServer((bind, port), handler) as server:
        host = "localhost" if bind == "127.0.0.1" else bind
        print(f"revue : http://{host}:{port}  (Ctrl+C pour arrêter)")
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
    args = sys.argv[1:]
    if args[:1] == ["--serve"]:
        bind = "127.0.0.1"
        if "--bind" in args:
            i = args.index("--bind")
            bind = args[i + 1]
            del args[i : i + 2]
        serve(int(args[1]) if len(args) > 1 else 8766, bind)
