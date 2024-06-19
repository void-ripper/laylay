struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    kind: u32,
    cut_off: f32,
    position: vec3<f32>,
    direction: vec3<f32>,
    ambient: vec3<f32>,
    diffuse: vec3<f32>,
    specular: vec3<f32>,
    attenutation: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> lights: array<Light, 10>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct Material {
    @location(6) ambient: vec3<f32>, 
    @location(7) diffuse: vec3<f32>,
    @location(8) specular: vec3<f32>,    
    @location(9) shininess: f32,
    @location(10) opacity: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) ambient: vec3<f32>, 
    @location(2) diffuse: vec3<f32>,
    @location(3) specular: vec3<f32>,    
    @location(4) shininess: f32,
    @location(5) opacity: f32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    model: VertexInput,
    @location(2) t0: vec4<f32>,
    @location(3) t1: vec4<f32>,
    @location(4) t2: vec4<f32>,
    @location(5) t3: vec4<f32>,
    material: Material,
) -> VertexOutput {
    var out: VertexOutput;
    var transform = mat4x4<f32>(t0, t1, t2, t3);

    out.ambient = material.ambient;
    out.diffuse = material.diffuse;
    out.specular = material.specular;
    out.shininess = material.shininess;
    out.opacity = material.opacity;
    out.clip_position = camera.view_proj * transform * vec4<f32>(model.position, 1.0);
    out.normal = (transform * vec4<f32>(model.normal, 1.0)).xyz;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out = vec4(0.0, 0.0, 0.0, 0.0);
    var shadow = 0.0;
    var light = lights[0];

    // directional light
    if (light.kind == 1) {
        var L = normalize(-light.direction); // direction
        var E = normalize(light.position - in.clip_position.xyz); // we are in Eye Coordinates, so EyePos is (0,0,0)
        var R = reflect(L, in.normal);

        var Iamb = light.ambient * in.ambient;
        var Idiff = light.diffuse * (in.diffuse * max(dot(in.normal, L), 0.0));
        var Ispec = light.specular * (in.specular * pow(max(dot(E, R), 0.0), in.shininess));

        out = vec4(Iamb +  (Idiff + Ispec) * (1.0 - shadow), 1.0);
    }
    // spot light
    else if (light.kind == 2) {
        var L = normalize(-light.direction);
        var E = normalize(light.position - in.clip_position.xyz);
        var R = reflect(L, in.normal);
        var theta: f32 = dot(-E, L);

        var color = light.ambient * in.ambient;

        if (theta > light.cut_off) {
            var Idiff = light.diffuse * (in.diffuse * max(dot(in.normal, L), 0.0));
            var Ispec = light.specular * (in.specular * pow(max(dot(E, R), 0.0), in.shininess));

            color += theta * (1.0 - shadow) * (Idiff + Ispec);
        }

        out = vec4(color , 1.0);
    }
    // point light
    else if (light.kind == 3) {
        var dis: f32 = length(light.position - in.clip_position.xyz);
        var attenutation: f32 = 1.0 / (light.attenutation.x + light.attenutation.y * dis + light.attenutation.z * dis * dis);

        var Iamb = light.ambient * in.ambient * attenutation;
        var Idiff = light.diffuse * in.diffuse * attenutation;
        var Ispec = light.specular * in.specular * attenutation;

        out = vec4(Iamb + (1.0 - shadow) * (Idiff + Ispec), 1.0);
    }
  
    // return vec4<f32>(in.color, 1.0);
    return out;
}
