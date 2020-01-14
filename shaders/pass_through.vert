#version 460

layout (location = 10) uniform mat4 uModelMat4;
layout (location = 11) uniform mat4 uViewMat4;
layout (location = 12) uniform mat4 uProjMat4;

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aUV;
layout (location = 3) in uint aMaterial;

layout (location = 0) out vec4 color;
layout (location = 1) out vec3 normal;
layout (location = 2) out vec2 uv;
layout (location = 3) out uint material;
layout (location = 4) out float depth;

void main()
{
    gl_Position = uProjMat4 * uViewMat4 * uModelMat4 * vec4(aPosition, 1);
    color = vec4(0.5, 0.5, 0.5, 1);
    normal = aNormal;
    uv = aUV;
    material = aMaterial;

    float Fcoef = 2.0 / log2(20000 + 1.0);
    //gl_Position.z = log2(max(1e-6, 1.0 + gl_Position.w)) * Fcoef - 1.0;
    // depth = pow(gl_Position.z, 2048);
}