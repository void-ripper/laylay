struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Material {
    @location(9) ambient: vec3<f32>, 
    @location(10) diffuse: vec3<f32>,
    @location(11) specular: vec3<f32>,    
    @location(12) shininess: f32,
    @location(13) opacity: f32,
}

struct Light {
    enabled: u32,
    kind: u32,
    position: vec3<f32>,
    ambient: vec3<f32>,
    diffuse: vec3<f32>,
    specular: vec3<f32>,
    cut_off: f32,
    size: f32,
    attenutation: vec3<f32>,
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
    @location(5) transform: mat4x4<f32>,
    material: Material,
) -> VertexOutput {
    var out: VertexOutput;
    // let x = f32(1 - i32(in_vertex_index)) * 0.5;
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    // out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = model.color;
    out.clip_position = camera.view_proj * transform * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out = vec4(0.0, 0.0, 0.0, 0.0);
    var shadow = 0.0;
    var light = lights[0];

    // directional light
    if (light.kind == 0) {
        var L = normalize(-light.dir); // direction
        var E = normalize(light.pos - in.clip_position); // we are in Eye Coordinates, so EyePos is (0,0,0)
        var R = reflect(L, norm);

        var Iamb = light.ambient * material.ambient;
        var Idiff = light.diffuse * (material.diffuse * max(dot(norm, L), 0.0));
        var Ispec = light.specular * (material.specular * pow(max(dot(E, R), 0.0), material.shininess));

        out = vec4(Iamb +  (Idiff + Ispec) * (1.0 - shadow), 1.0);
    }
    // spot light
    else if (light.kind == 1) {
        var L = normalize(-light.direction);
        var E = normalize(light.position - in.clip_position);
        var R = reflect(L, norm);
        var theta: f32 = dot(-E, L);

        var color = light.ambient * material.ambient;

        if (theta > light.cutOff) {
            var Idiff = light.diffuse * (material.diffuse * max(dot(norm, L), 0.0));
            var Ispec = light.specular * (material.specular * pow(max(dot(E, R), 0.0), material.shininess));

            color += theta * (1.0 - shadow) * (Idiff + Ispec);
        }

        out = vec4(color , 1.0);
    }
    // point light
    else {
        var dis: f32 = length(light.position - in.clip_position);
        var attenutation: f32 = 1.0 / (light.attenutation.x + light.attenutation.y * dis + light.attenutation.z * dis * dis);

        var Iamb = light.ambient * material.ambient * attenutation;
        var Idiff = light.diffuse * material.diffuse * attenutation;
        var Ispec = light.specular * material.specular * attenutation;

        out = vec4(Iamb + (1.0 - shadow) * (Idiff + Ispec), 1.0);
    }    
  
    return vec4<f32>(in.color, 1.0);
}
