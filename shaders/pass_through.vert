#version 460

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aUV;
layout (location = 3) in uint aMaterial;

layout (location = 0) out vec4 color;
layout (location = 1) out vec3 normal;
layout (location = 2) out vec2 uv;
layout (location = 3) out uint material;

void main()
{
    gl_Position = vec4(aPosition, 1);
    color = vec4(0.5, 0.5, 0.5, 1);
    normal = aNormal;
    uv = aUV;
    material = aMaterial;
}