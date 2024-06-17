struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    kind: u32,
    pos: vec3<f32>,
    dir: vec3<f32>,
    color: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> lights: array<Light, 10>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // let x = f32(1 - i32(in_vertex_index)) * 0.5;
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    // out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out: vec4 = vec4(0.0, 0.0, 0.0, 0.0);
    var shadow: f32 = 0.0;

    // directional light
    if (light.kind == 0) {
        var L: vec3 = normalize(-light.dir); // direction
        var E: vec3 = normalize(light.pos - in.clip_position); // we are in Eye Coordinates, so EyePos is (0,0,0)
        var R: vec3 = reflect(L, norm);

        var Iamb: vec3 = light.ambient * material.ambient;
        var Idiff: vec3 = light.diffuse * (material.diffuse * max(dot(norm, L), 0.0));
        var Ispec: vec3 = light.specular * (material.specular * pow(max(dot(E, R), 0.0), material.shininess));

        out = vec4(Iamb +  (Idiff + Ispec) * (1.0 - shadow), 1.0);
    }
    // spot light
    else if (light.kind == 1) {
        var L: vec3 = normalize(-light.direction);
        var E: vec3 = normalize(light.position - in.clip_position);
        var R: vec3 = reflect(L, norm);
        var theta: f32 = dot(-E, L);

        var color: vec3 = light.ambient * material.ambient;

        if (theta > light.cutOff) {
            var Idiff: vec3 = light.diffuse * (material.diffuse * max(dot(norm, L), 0.0));
            var Ispec: vec3 = light.specular * (material.specular * pow(max(dot(E, R), 0.0), material.shininess));

            color += theta * (1.0 - shadow) * (Idiff + Ispec);
        }

        out = vec4(color , 1.0);
    }
    // point light
    else {
        var dis: f32 = length(light.position - in.clip_position);
        var attenutation: f32 = 1.0 / (light.attenutation.x + light.attenutation.y * dis + light.attenutation.z * dis * dis);

        var Iamb: vec3 = light.ambient * material.ambient * attenutation;
        var Idiff: vec3 = light.diffuse * material.diffuse * attenutation;
        var Ispec: vec3 = light.specular * material.specular * attenutation;

        out = vec4(Iamb + (1.0 - shadow) * (Idiff + Ispec), 1.0);
    }    
  
    return vec4<f32>(in.color, 1.0);
}
