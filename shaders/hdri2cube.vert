//https://learnopengl.com/PBR/IBL/Diffuse-irradiance
#version 460 core

layout (location = 10) uniform mat4 uProjMat4;
layout (location = 11) uniform mat4 uViewMat4;

layout (location = 0) in vec3 aPosition;
layout (location = 0) out vec3 position;

void main()
{
    position = aPosition;
    gl_Position =  uProjMat4 * uViewMat4 * vec4(aPosition, 1.0);
}