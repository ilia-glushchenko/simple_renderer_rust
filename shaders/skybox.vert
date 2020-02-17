#version 460

layout (location = 10) uniform mat4 uModelMat4;
layout (location = 11) uniform mat4 uViewMat4;
layout (location = 12) uniform mat4 uProjMat4;

layout (location = 0) in vec3 aPosition;

layout (location = 0) out vec3 texCoord;

void main()
{
    texCoord = aPosition;
    gl_Position = uProjMat4 * uViewMat4 * uModelMat4 * vec4(aPosition, 1);
}