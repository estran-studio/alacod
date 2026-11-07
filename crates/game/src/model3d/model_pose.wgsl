// Pose d'un personnage « façon sprite » sur le décor (R-1, docs/conventions.md § 111), portée de la
// passe `POSE` du banc (`facon-sprite.js`) : le personnage est dessiné seul dans une image au fond
// transparent (sur-échantillonnée : `S × S` texels par pixel de sortie, la moyenne fait l'anticrénelage),
// puis posé ici avec l'anneau du contour : pixels vides à moins d'un pixel de sprite (`rayon`, en pixels
// de sortie) de la silhouette, teinte locale (moyenne des voisins) × `contour`. L'image porte aussi
// l'ombre (alpha 0,5, voir `model_shadow.wgsl`), posée sous le corps à son opacité.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ModelPose {
    // xy : taille de la sortie (pixels) ; z : rayon du contour (pixels de sortie) ; w : teinte du contour.
    params: vec4<f32>,
    // x : opacité de l'ombre ; y : texels par pixel de sortie (S).
    extra: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> pose: ModelPose;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var image: texture_2d<f32>;

struct Cell {
    // Corps : couleur prémultipliée (linéaire) et couverture.
    body: vec4<f32>,
    // Part de l'ombre dans le pixel.
    shadow: f32,
}

fn cell(p: vec2<i32>) -> Cell {
    var out: Cell;
    out.body = vec4<f32>(0.0);
    out.shadow = 0.0;
    let s = i32(pose.extra.y);
    let size = vec2<i32>(textureDimensions(image));
    let n = f32(s * s);
    for (var dy = 0; dy < s; dy++) {
        for (var dx = 0; dx < s; dx++) {
            let q = p * s + vec2<i32>(dx, dy);
            if q.x < 0 || q.y < 0 || q.x >= size.x || q.y >= size.y {
                continue;
            }
            let t = textureLoad(image, q, 0);
            if t.a > 0.75 {
                out.body += vec4<f32>(t.rgb, 1.0) / n;
            } else if t.a > 0.25 {
                out.shadow += 1.0 / n;
            }
        }
    }
    return out;
}

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
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<i32>(floor(mesh.uv * pose.params.xy));
    let here = cell(p);
    var color = here.body;
    if here.body.a <= 0.5 {
        let radius = pose.params.z;
        var sum = vec4<f32>(0.0);
        for (var dy = -3; dy <= 3; dy++) {
            for (var dx = -3; dx <= 3; dx++) {
                let d = vec2<f32>(f32(dx), f32(dy));
                if length(d) <= radius + 0.01 {
                    sum += cell(p + vec2<i32>(dx, dy)).body;
                }
            }
        }
        if sum.a >= 0.5 {
            // La teinte est assombrie sur les valeurs sRGB, comme au banc.
            color = vec4<f32>(to_linear(to_srgb(sum.rgb / sum.a) * pose.params.w), 1.0);
        }
    }
    // L'ombre (noire) sous le corps et son contour.
    let a = color.a + (1.0 - color.a) * here.shadow * pose.extra.x;
    if a <= 0.0 {
        discard;
    }
    return vec4<f32>(color.rgb / a, a);
}
