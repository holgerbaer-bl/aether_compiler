struct GlobalUniform {
    view_proj: mat4x4<f32>,
    cam_pos: vec4<f32>,
    sky_color: vec4<f32>,
};
@group(0) @binding(0) var<uniform> globals: GlobalUniform;

@group(1) @binding(0) var t_atlas: texture_2d<f32>;
@group(1) @binding(1) var s_atlas: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct InstanceInput {
    @location(3) instance_pos_and_id: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = model.position + instance.instance_pos_and_id.xyz;
    out.clip_position = globals.view_proj * vec4<f32>(world_position, 1.0);
    out.world_position = world_position;
    out.normal = model.normal;

    // UV Atlas Calculation (4x4 tiles)
    var block_id = u32(instance.instance_pos_and_id.w);
    let atlas_size_tiles = 4.0;
    
    // Row 0: Grass (1), Stone (2) -> IDs are 1-based in engine mostly
    // We map: 
    // ID 1 (Grass): Top=0, Side=1, Bottom=2 (Dirt)
    // ID 2 (Stone): All=3
    // ID 3 (Sand): All=4
    // ID 4 (Water): All=5
    // ID 5 (Wood): Top/Bottom=7, Side=6
    // ID 6 (Leaves): All=8
    
    var tile_index: f32 = 0.0;
    
    if (block_id == 1u) { // Grass
        if (model.normal.y > 0.5) {
            tile_index = 0.0; // Grass Top
        } else if (model.normal.y < -0.5) {
            tile_index = 2.0; // Dirt
        } else {
            tile_index = 1.0; // Grass Side
        }
    } else if (block_id == 2u) { // Stone
        tile_index = 3.0;
    } else if (block_id == 3u) { // Sand
        tile_index = 4.0;
    } else if (block_id == 4u) { // Water
        tile_index = 5.0;
    } else if (block_id == 5u) { // Wood
        if (abs(model.normal.y) > 0.5) {
            tile_index = 7.0; // Wood Top
        } else {
            tile_index = 6.0; // Wood Side
        }
    } else if (block_id == 6u) { // Leaves
        tile_index = 8.0;
    } else {
        tile_index = f32(block_id);
    }
    
    let tile_x = tile_index % atlas_size_tiles;
    let tile_y = floor(tile_index / atlas_size_tiles);
    
    let uv_offset = vec2<f32>(tile_x, tile_y) / atlas_size_tiles;
    let scaled_uv = model.uv / atlas_size_tiles;

    out.tex_coords = scaled_uv + uv_offset;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    
    let tex_color = textureSample(t_atlas, s_atlas, in.tex_coords);

    if (tex_color.a < 0.1) {
        discard;
    }

    // Directional Lighting (Sun)
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));
    let diffuse = max(dot(in.normal, light_dir), 0.0);
    let ambient = 0.35;
    let light_factor = diffuse + ambient;
    
    var final_color = tex_color.rgb * light_factor;
    
    // Distance Fog
    let view_dist = distance(globals.cam_pos.xyz, in.world_position);
    let fog_start = 30.0;
    let fog_end = 65.0;
    let fog_factor = clamp((view_dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    
    final_color = mix(final_color, globals.sky_color.rgb, fog_factor);
    
    return vec4<f32>(final_color, tex_color.a);
}
