//! P-1d (`docs/conventions.md` § 122) : la durée d'un clip lue dans le manifeste (`AnimeManifest::duration`, `fin / fps`) est le
//! temps de la **dernière clé** du clip dans son `.glb`, ce que three.js (banc) et bevy jouent. Garde sur le modèle du testbed
//! (`games/testbed/assets/models/pantin`, écrit par `scripts/gen-pantin.py`) : un générateur qui déclarerait `[1, 1 + d × fps]`
//! pour des clés de 0 à `d` allongerait chaque clip d'une image.

use std::collections::BTreeMap;
use std::path::PathBuf;

use content::models::AnimeManifest;

/// Le temps de la dernière clé de chaque animation d'un `.glb` (maximum des entrées de ses échantillonneurs).
fn last_key_times(glb: &[u8]) -> BTreeMap<String, f64> {
    let json_len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    assert_eq!(&glb[16..20], b"JSON", "premier bloc du glb");
    let gltf: serde_json::Value = serde_json::from_slice(&glb[20..20 + json_len]).unwrap();
    let accessors = gltf["accessors"].as_array().unwrap();
    gltf["animations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|animation| {
            let last = animation["samplers"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| {
                    accessors[s["input"].as_u64().unwrap() as usize]["max"][0]
                        .as_f64()
                        .unwrap()
                })
                .fold(0.0, f64::max);
            (animation["name"].as_str().unwrap().to_string(), last)
        })
        .collect()
}

#[test]
fn the_testbed_model_declares_each_clip_up_to_its_last_key() {
    let dir = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../games/testbed/assets/models/pantin"
    ));
    let manifest: AnimeManifest =
        serde_json::from_str(&std::fs::read_to_string(dir.join("anime.json")).unwrap()).unwrap();
    let keys = last_key_times(&std::fs::read(dir.join("anime.glb")).unwrap());
    assert!(!keys.is_empty());
    let mut wrong = Vec::new();
    for (clip, last) in &keys {
        let duration = manifest
            .duration(clip)
            .unwrap_or_else(|| panic!("clip `{clip}` absent du manifeste"));
        if (f64::from(duration) - last).abs() > 1e-4 {
            wrong.push(format!(
                "{clip} : manifeste {duration} s, dernière clé {last} s"
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}
