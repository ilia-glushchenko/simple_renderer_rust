#version 460

layout (binding = 0, location = 30) uniform samplerCube uSkyboxSamplerCube;
layout (location = 0) in vec3 texCoord;
layout (location = 0) out vec4 outColor;

void main()
{
    outColor = texture(uSkyboxSamplerCube, texCoord, 0);
}