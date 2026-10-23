#!/usr/bin/env bash
# Voix Apple françaises à installer : Thomas ou Amélie (Réglages > Accessibilité > Contenu énoncé).
set +e
if [[ "${ALACOD_VOICE:-}" == "stop" ]]; then
  pkill say 2>/dev/null || true
  exit 0
fi
[[ "${ALACOD_VOICE:-}" == "1" ]] || exit 0

payload=$(cat)
stop_active=$(printf '%s' "$payload" | python3 -c 'import json,sys
try: print(json.load(sys.stdin).get("stop_hook_active", False))
except Exception: print(False)')
[[ "$stop_active" == "True" ]] && exit 0

transcript=$(printf '%s' "$payload" | python3 -c 'import json,sys
try: print(json.load(sys.stdin).get("transcript_path", ""))
except Exception: print("")')
[[ -r "$transcript" ]] || exit 0
text=$(python3 - "$transcript" <<'PY'
import json, sys, re
path = sys.argv[1]
last = ""
try:
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                item = json.loads(line)
            except Exception:
                continue
            if item.get("type") != "assistant":
                continue
            content = item.get("message", {}).get("content", [])
            last = " ".join(block.get("text", "") for block in content
                            if isinstance(block, dict) and block.get("type") == "text")
except Exception:
    pass
# Remplacer les blocs de code avant les autres marqueurs Markdown.
last = re.sub(r"```[\s\S]*?```", " bloc de code ", last)
last = re.sub(r"`[^`]*`", " ", last)
last = re.sub(r"!?\[([^\]]+)\]\([^)]*\)", r"\1", last)
last = re.sub(r"[#*]", "", last)
last = re.sub(r"\s+", " ", last).strip()
try:
    limit = max(1, int(__import__("os").environ.get("ALACOD_VOICE_MAX", "600")))
except ValueError:
    limit = 600
if len(last) > limit:
    cut = last[:limit]
    end = max(cut.rfind("."), cut.rfind("!"), cut.rfind("?"))
    last = cut[:end + 1] if end >= limit // 2 else cut.rstrip() + "…"
print(last)
PY
)
[[ -n "$text" ]] || exit 0

if command -v say >/dev/null 2>&1; then
  voice_args=()
  voice="${ALACOD_VOICE_NAME:-}"
  if [[ -n "$voice" ]] && say -v '?' 2>&1 | awk '{print $1}' | grep -qx "$voice"; then
    voice_args=(-v "$voice")
  fi
  ( nohup say "${voice_args[@]}" "$text" >/dev/null 2>&1 & )
elif command -v espeak-ng >/dev/null 2>&1; then
  ( nohup espeak-ng -v fr "$text" >/dev/null 2>&1 & )
elif command -v espeak >/dev/null 2>&1; then
  ( nohup espeak -v fr "$text" >/dev/null 2>&1 & )
elif command -v piper >/dev/null 2>&1 && [[ -n "${ALACOD_VOICE_PIPER_MODEL:-}" ]]; then
  ( printf '%s\n' "$text" | nohup piper --model "$ALACOD_VOICE_PIPER_MODEL" >/dev/null 2>&1 & )
else
  printf '%s\n' "$text" > "${TMPDIR:-/tmp}/alacod-voice.txt"
fi
exit 0
