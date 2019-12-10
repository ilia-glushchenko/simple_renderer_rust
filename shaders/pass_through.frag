#version 460

layout (location = 0) in vec4 color;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 uv;
layout (location = 3) flat in uint material;

layout (location = 0) out vec4 outColor;

void main()
{
    outColor = color + vec4(normal, 0) + vec4(uv, 0, 0) + material.xxxx;
}