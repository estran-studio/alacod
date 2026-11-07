// Rendu « façon sprite » d'un modèle 3D (R-1, docs/conventions.md § 111), porté du shader du banc
// (`facon-sprite.js` de la pipeline d'assets) : couleur de la texture lue en sRGB × une rampe continue
// de l'ombre au reflet, `0,5 + 0,5 × N·L`, lumière fixe dans le repère de la caméra, normale (lissée
// par position au chargement) tournée vers la caméra. Le sommet est celui de Bevy (squelette compris).
#import bevy_pbr::{forward_io::VertexOutput, mesh_view_bindings::view}

struct FaconSprite {
    // rgb : couleur sans texture ; a : 1 si la texture est présente.
    color: vec4<f32>,
    // rgb : émission ajoutée (flash d'un coup), dans l'espace sRGB comme au banc.
    emissive: vec4<f32>,
    // xyz : direction de la lumière, repère de la vue (x droite, y haut, z vers le spectateur).
    light: vec4<f32>,
    // x : multiplicateur à l'ombre ; y : au reflet.
    ramp: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: FaconSprite;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var base_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var base_sampler: sampler;

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(max(c, vec3(0.0031308)), vec3(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3(0.0031308));
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((max(c, vec3(0.04045)) + 0.055) / 1.055, vec3(2.4));
    return select(hi, lo, c <= vec3(0.04045));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var c = material.color.rgb;
#ifdef VERTEX_UVS_A
    if material.color.a > 0.5 {
        c = textureSample(base_texture, base_sampler, in.uv).rgb;
    }
#endif
    // La texture est décodée en linéaire par l'échantillonneur : le banc travaille sur ses valeurs sRGB.
    let albedo = to_srgb(c);
    var n = normalize((view.view_from_world * vec4(in.world_normal, 0.0)).xyz);
    // Tournée vers la caméra : c'est la face qu'on voit (le sens des triangles ne dit rien).
    if n.z < 0.0 {
        n = -n;
    }
    let t = clamp(0.5 + 0.5 * dot(n, material.light.xyz), 0.0, 1.0);
    let v = clamp(albedo * mix(material.ramp.x, material.ramp.y, t) + material.emissive.rgb, vec3(0.0), vec3(1.0));
    // L'image cible est en sRGB : écrire la valeur linéaire de `v` y range exactement `v`. Alpha 1 :
    // le corps (l'ombre écrit 0,5, le fond 0 ; la passe de pose les distingue).
    return vec4(to_linear(v), 1.0);
}
