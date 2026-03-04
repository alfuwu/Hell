pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "#version 460
layout(set = 0, binding = 0) uniform Camera {
    mat4 view_proj;
} camera;
layout(push_constant) uniform PushConstants {
    mat4 model;
} push;
layout(set = 0, binding = 2) readonly buffer BoneMatrices {
    mat4 bones[];
} skin;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec2 v_uv;

layout(location = 3) in uvec4 bone_indices;
layout(location = 4) in vec4 bone_weights;

void main() {
    mat4 skin_matrix = mat4(0.0);
    float total_weight = 0.0;

    for (int i = 0; i < 4; i++) {
        if (bone_indices[i] < 0xFFFFFFFFu && bone_weights[i] > 0.0) {
            skin_matrix += bone_weights[i] * skin.bones[bone_indices[i]];
            total_weight += bone_weights[i];
        }
    }

    if (total_weight < 0.0001) {
        skin_matrix = mat4(1.0);
    }
    else if (abs(total_weight - 1.0) > 0.0001) {
        skin_matrix /= total_weight;
    }

    vec4 skinned_pos = skin_matrix * vec4(position, 1.0);
    vec4 world_pos = push.model * skinned_pos;

    mat3 normal_skin = mat3(skin_matrix);
    mat3 normal_model = mat3(push.model);

    v_normal = normalize(normal_model * normal_skin * normal);
    v_world_pos = world_pos.xyz;
    v_uv = uv;

    gl_Position = camera.view_proj * world_pos;
}"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "#version 460
layout(set = 0, binding = 1) uniform sampler2D tex;
//layout(set = 0, binding = 2) uniform sampler2D shadow_map;
//layout(set = 0, binding = 3) uniform Light {
//    mat4 light_view_proj;
//} light;

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

void main() {
    vec3 light_dir = normalize(vec3(1.0, 1.0, 1.0));

    vec3 normal = normalize(v_normal);
    float diff = max(dot(normal, light_dir), 0.0);

    vec3 tex_color = texture(tex, v_uv).rgb;

    vec3 ambient = 0.2 * tex_color;
    vec3 diffuse = diff * tex_color;

    /*vec4 light_space = light.light_view_proj * vec4(v_world_pos, 1.0);
    vec3 proj = light_space.xyz / light_space.w;
    vec2 shadow_uv = proj.xy * 0.5 + 0.5;
    float closest_depth = texture(shadow_map, shadow_uv).r;
    float current_depth = proj.z;
    float bias = 0.005;
    float in_shadow = current_depth - bias > closest_depth ? 0.3 : 1.0;*/

    f_color = vec4((ambient + diffuse) /** in_shadow*/, 1.0);
}"
    }
}

pub mod shadow_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "#version 460
layout(set = 0, binding = 0) uniform Light {
    mat4 light_view_proj;
} light;
layout(push_constant) uniform PushConstants {
    mat4 model;
} push;

layout(location = 0) in vec3 position;

void main() {
    gl_Position = light.light_view_proj * push.model * vec4(position, 1.0);
}"
    }
}