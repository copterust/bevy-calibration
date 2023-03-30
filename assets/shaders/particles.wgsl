#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

struct Time {
    time_since_startup: f32,
};
@group(1) @binding(0)
var<uniform> time: Time;

#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(4) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var model = mesh.model;
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.color = vertex.color;
    out.clip_position = mesh_position_world_to_clip(out.world_position);
    return out;
}

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(4) color: vec4<f32>,
};

@fragment
fn fragment(
    in: FragmentInput,
) -> @location(0) vec4<f32> {
    return vec4(in.color.rgb, 1.0);
}
