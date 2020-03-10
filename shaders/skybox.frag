#version 460 core

layout (binding = 0, location = 30) uniform samplerCube uSkyboxSamplerCube;
layout (location = 0) in vec3 texCoord;
layout (location = 0) out vec4 outColor;

void main()
{
    vec3 color = texture(uSkyboxSamplerCube, -texCoord, 0).rgb;
    outColor = vec4(color, 1);
}