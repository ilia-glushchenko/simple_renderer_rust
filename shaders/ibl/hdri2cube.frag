//https://learnopengl.com/PBR/IBL/Diffuse-irradiance
#version 460 core

layout (binding = 0, location = 30) uniform sampler2D uHdriSampler2D;
layout (location = 0) in vec3 position;
layout (location = 0) out vec4 outColor;


vec2 SampleSphericalMap(vec3 v)
{
    const vec2 invAtan = vec2(0.1591, 0.3183);

    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;

    return uv;
}

void main()
{
    vec2 uv = SampleSphericalMap(normalize(position));
    vec3 color = texture(uHdriSampler2D, uv).rgb;

    outColor = vec4(color, 1.0);
}
