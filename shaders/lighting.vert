#version 460

layout (location = 10) uniform mat4 uModelMat4;
layout (location = 11) uniform mat4 uViewMat4;
layout (location = 12) uniform mat4 uProjMat4;
layout (location = 13) uniform vec3 uCameraPosVec3;

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec3 aTangent;
layout (location = 3) in vec3 aBitangent;
layout (location = 2) in vec2 aUV;

layout (location = 0) out vec3 normalModel;
layout (location = 1) out vec3 tangentModel;
layout (location = 2) out vec3 bitangentModel;
layout (location = 3) out vec2 uv;

layout (location = 4) out vec3 normalWorld;
layout (location = 5) out vec3 tangentWorld;
layout (location = 6) out vec3 bitangentWorld;
layout (location = 7) out vec3 positionWorld;
layout (location = 8) out vec3 cameraPositionWorld;

layout (location = 9) out vec3 posView;

void main()
{
    normalModel = aNormal;
    tangentModel = aTangent;
    bitangentModel = aBitangent;
    uv = aUV;

    normalWorld = (uModelMat4 * vec4(normalModel, 0)).xyz;
    tangentWorld = (uModelMat4 * vec4(tangentModel, 0)).xyz;
    bitangentWorld = (uModelMat4 * vec4(bitangentModel, 0)).xyz;
    positionWorld = (uModelMat4 * vec4(aPosition, 1)).xyz;
    cameraPositionWorld = uCameraPosVec3;

    posView = (uViewMat4 *  uModelMat4 * vec4(aPosition, 1)).xyz;

    gl_Position = uProjMat4 * uViewMat4 * uModelMat4 * vec4(aPosition, 1);
}