#version 460

layout (location = 0) in vec4 color;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 uv;
layout (location = 3) flat in uint material;
layout (location = 4) in float depth;

layout (binding = 0, location = 30) uniform sampler2D uAlbedoMapSampler2D;
layout (binding = 1, location = 31) uniform sampler2D uNormalMapSampler2D;
layout (binding = 2, location = 32) uniform sampler2D uBumpMapSampler2D;
layout (binding = 3, location = 33) uniform sampler2D uMetallicSampler2D;
layout (binding = 4, location = 34) uniform sampler2D uRoughnessSampler2D;

layout (location = 0) out vec4 outColor;

void main()
{
    // outColor = color + vec4(normal, 0) + vec4(uv, 0, 0) + material.xxxx;
    //outColor = vec4((normal + 1) / 2, 1);
    outColor = texture(uAlbedoMapSampler2D, uv, 0);

    //outColor = vec4(pow(depth, 10).xxx, 1);
}