#version 460

layout (location = 10) uniform mat4 uModelMat4;
layout (location = 11) uniform mat4 uViewMat4;
layout (location = 12) uniform mat4 uProjMat4;

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aUV;

layout (location = 0) out vec3 normal;
layout (location = 1) out vec2 uv;
layout (location = 2) out vec4 normalView;
layout (location = 3) out vec4 lightView;
layout (location = 4) out vec4 posView;

const vec3 light_direction = -normalize(vec3(1, 1, 1));

void main()
{
    normal = aNormal;
    uv = aUV;

    normalView = uViewMat4 * vec4(normal, 0);
    lightView = uViewMat4 * vec4(light_direction, 0);
    posView = uViewMat4 *  uModelMat4 * vec4(aPosition, 1);

    gl_Position = uProjMat4 * uViewMat4 * uModelMat4 * vec4(aPosition, 1);
}