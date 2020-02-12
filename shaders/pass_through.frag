#version 460

layout (location = 10) uniform mat4 uModelMat4;
layout (location = 11) uniform mat4 uViewMat4;
layout (location = 12) uniform mat4 uProjMat4;
layout (location = 13) uniform vec3 uCameraPosVec3;

layout (binding = 0, location = 30) uniform sampler2D uAlbedoMapSampler2D;
layout (binding = 1, location = 31) uniform sampler2D uNormalMapSampler2D;
layout (binding = 2, location = 32) uniform sampler2D uBumpMapSampler2D;
layout (binding = 3, location = 33) uniform sampler2D uMetallicSampler2D;
layout (binding = 4, location = 34) uniform sampler2D uRoughnessSampler2D;

layout (location = 0) in vec3 normal;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 normalView;
layout (location = 4) in vec3 posView;

layout (location = 0) out vec4 outColor;

vec3 pointLightWindowing(vec3 l, vec3 pointLightColor)
{
    float r0 = 1;
    float r_min = 50;
    float r_max = 100;
    float r = length(l);

    float windowed_r = max(0, pow(1 - r / r_max, 2));
    vec3 attenuated_color = pointLightColor * pow( r0 / max(windowed_r, r_min), 2);

    return attenuated_color;
}

void main()
{
    vec3 pointLightColor = vec3(1, 1, 1);
    const vec3 pointLightPosWorld = vec3(0, 1000, 0);
    const vec3 pointLightPosView = (uViewMat4 * vec4(pointLightPosWorld, 1)).xyz;

    vec4 albedo = texture(uAlbedoMapSampler2D, uv, 0);

    vec3 l = pointLightPosView - posView;
    vec3 attenuated_color = pointLightWindowing(l, pointLightColor);

    outColor = vec4(attenuated_color * max(0, dot(normalView, l)), 1);
}