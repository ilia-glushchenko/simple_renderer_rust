#version 460

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec2 aUV;

layout (location = 0) out vec2 uv;

void main()
{
    uv = aUV;
    gl_Position = vec4(aPosition, 1);
}
