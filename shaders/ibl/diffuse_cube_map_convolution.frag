#version 460 core

layout (binding = 0, location = 30) uniform samplerCube uSkyboxSamplerCube;
layout (location = 0) in vec3 texCoord;
layout (location = 0) out vec4 outColor;

#define M_PI 3.1415926535897932384626433832795

void main()
{
    vec3 normal = normalize(-texCoord);
    vec3 up = vec3(0.0, 1.0, 0.0);
    vec3 right = normalize(cross(up, normal));
    up = normalize(cross(normal, right));

    vec3 irradiance = vec3(0.0);
    float sampleDelta = 0.015;
    int nrSamples = 0;

    for(float phi = 0.0; phi < 2.0 * M_PI; phi += sampleDelta)
    {
        for(float theta = 0.0; theta < 0.5 * M_PI; theta += sampleDelta)
        {
            // spherical to cartesian (in tangent space)
            vec3 tangentSample = normalize(
                vec3(sin(theta) * cos(phi),  sin(theta) * sin(phi), cos(theta)));
            // tangent space to world
            vec3 sampleVec = tangentSample.x * right + tangentSample.y * up + tangentSample.z * normal;

            irradiance += texture(uSkyboxSamplerCube, sampleVec).rgb * cos(theta) * sin(theta);
            nrSamples++;
        }
    }

    irradiance = M_PI * irradiance / float(nrSamples);
    outColor = vec4(irradiance, 1);
}