#version 460

layout (location = 10) uniform mat4 uModelMat4;
layout (location = 11) uniform mat4 uViewMat4;
layout (location = 12) uniform mat4 uProjMat4;
layout (location = 13) uniform vec3 uCameraPosVec3;

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec3 aTangent;
layout (location = 3) in vec3 aBitangent;
layout (location = 4) in vec2 aUV;

layout (location = 0) out vec2 uv;
layout (location = 1) out vec3 normalWorld;
layout (location = 2) out vec3 positionWorld;
layout (location = 3) out vec3 cameraPositionWorld;
layout (location = 4) out mat3 mTBN;

void main()
{
    uv = aUV;
    normalWorld = normalize(vec3(uModelMat4 * vec4(aNormal, 0.0)));
    positionWorld = (uModelMat4 * vec4(aPosition, 1)).xyz;
    cameraPositionWorld = uCameraPosVec3;

    vec3 t = normalize(uModelMat4 * vec4(aTangent, 0)).rgb;
    vec3 b = normalize(uModelMat4 * vec4(aBitangent, 0)).rgb;
    mTBN = mat3(t, b, normalWorld);

    gl_Position = uProjMat4 * uViewMat4 * uModelMat4 * vec4(aPosition, 1);
}