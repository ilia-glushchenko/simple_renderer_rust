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
layout (location = 2) in vec4 normalView;
layout (location = 3) in vec4 lightView;
layout (location = 4) in vec4 posView;

layout (location = 0) out vec4 outColor;

void main()
{
    vec3 light_direction = -normalize(vec3(1, 1, 1));
    vec4 albedo = texture(uAlbedoMapSampler2D, uv, 0);
    float diffuse = dot(normal, light_direction) * 0.5;

    vec3 positionView = posView.xyz / posView.w;

    float spec = max(0, dot(reflect(lightView.xyz, normalView.xyz), -normalize(positionView)));

    outColor = vec4(albedo.rgb * spec, albedo.a);
    outColor = albedo;
}