// Ombre portée d'un modèle « façon sprite » (R-1, docs/conventions.md § 111) : le maillage (squelette
// compris, comme le sommet de Bevy `mesh.wgsl`) projeté sur le sol (y = 0) le long de la lumière, en
// noir d'alpha 0,5. La passe de pose reconnaît cet alpha et y pose l'ombre à son opacité, sous le corps.
#import bevy_pbr::{
    mesh_functions,
    skinning,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

struct ModelShadow {
    // xyz : direction vers la lumière, repère du monde, normalisée, y > 0.
    direction: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> shadow: ModelShadow;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
#ifdef SKINNED
    let world_from_local = skinning::skin_model(vertex.joint_indices, vertex.joint_weights, vertex.instance_index);
#else
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
#endif
    let world = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    let l = shadow.direction.xyz;
    let ground = world.xyz - l * (world.y / l.y);
    // Un peu sous le sol : le pied, posé à y = 0, passe devant son ombre.
    out.world_position = vec4<f32>(ground.x, -0.01, ground.z, 1.0);
    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);
#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.5);
}
