#version 460

layout (location = 10) uniform mat4 uModelMat4;
layout (location = 11) uniform mat4 uViewMat4;
layout (location = 12) uniform mat4 uProjMat4;
layout (location = 13) uniform vec3 uCameraPosVec3;
layout (location = 14) uniform vec3 uScalarAlbedoVec3f;
layout (location = 15) uniform float uScalarRoughnessVec1f;
layout (location = 16) uniform float uScalarMetalnessVec1f;

layout (binding = 0, location = 30) uniform sampler2D uAlbedoMapSampler2D;
layout (binding = 1, location = 31) uniform sampler2D uNormalMapSampler2D;
layout (binding = 2, location = 32) uniform sampler2D uBumpMapSampler2D;
layout (binding = 3, location = 33) uniform sampler2D uMetallicSampler2D;
layout (binding = 4, location = 34) uniform sampler2D uRoughnessSampler2D;
layout (binding = 5, location = 35) uniform samplerCube uSpecularSamplerCube;
layout (binding = 6, location = 36) uniform samplerCube uDiffuseSamplerCube;

layout (location = 0) in vec3 normalModel;
layout (location = 1) in vec3 tangentModel;
layout (location = 2) in vec3 bitangentModel;
layout (location = 3) in vec2 uv;
layout (location = 4) in vec3 normalWorld;
layout (location = 5) in vec3 tangentWorld;
layout (location = 6) in vec3 bitangentWorld;
layout (location = 7) in vec3 positionWorld;
layout (location = 8) in vec3 cameraPositionWorld;
layout (location = 9) in vec3 posView;

layout (location = 0) out vec4 outColor;

#define M_PI 3.1415926535897932384626433832795
#define EPSILON 1e-5

float ClampPunctualLightRadiance(float r, float radiance)
{
    float r0 = 250;
    float r_min = 10;
    float r_max = 10000;

    float win = pow(max(0, 1 - pow(r / r_max, 4)), 2);
    float attenuated_radiance = radiance * (r0 * r0) / pow(max(r, r_min), 2) * win;

    return attenuated_radiance;
}

// Diffuce BRDF components

float HeavisideStepFunction(float s)
{
    return s <= 0.0 ? 0.0 : 1.0;
}

float SchlickFresnel(float f0, float f90, float u)
{
    return f0 + (f90 - f0) * pow(1. - u, 5.);
}

vec3 SchlickFresnel3(in vec3 f0, in float f90, in float u)
{
    return f0 + (f90 - f0) * pow(1.f - u, 5.f);
}

//Moving Frostbite to Physically Based Rendering 3.0 (page 10)
float DisneyDiceDiffuse(vec3 n, vec3 l, vec3 v, vec3 h, float lin_roughness, vec3 F0)
{
	float NdotV = clamp(abs(dot(n, v)) + EPSILON, 0., 1.);
	float LdotH = clamp(dot(l, h), 0., 1.);
	float NdotL = clamp(dot(n, l), 0., 1.);

    float energyBias = mix(0.0, 0.5, lin_roughness);
    float energyFactor = mix(1.0, 1.0 / 1.51, lin_roughness);
    float fd90 = energyBias + 2.0 * LdotH * LdotH * lin_roughness;
    float lightScatter = SchlickFresnel3(F0, fd90, NdotL).r;
    float viewScatter = SchlickFresnel3(F0, fd90, NdotV).r;

    return lightScatter * viewScatter * energyFactor;
}

//See Real-Time Rendering (page 351)
//  "This can only be applied to surfaces where the specular reflectane is that
//  of a perfect Fresnel mirror."
//So I assume it is not going to work right with Microfacet BRDFs.
float ShirleyDiffuse(vec3 n, vec3 l, vec3 v, float roughness, float F0)
{
    float lightScatter = 1 - pow(1 - max(0, dot(n, l)), 5);
    float viewScatter = 1 - pow(1 - max(0, dot(n, v)), 5);
    float fresnelFactor = 21.0 / (20.0 * M_PI) * (1 - F0);

    return  fresnelFactor * roughness * lightScatter * viewScatter;
}

//See Real-Time Rendering (page 355)
vec3 HammonDiffuse(vec3 n, vec3 l, vec3 v, vec3 h, float roughness, vec3 F0, vec3 albedo)
{
	float NdotV = abs(dot(n, v)) + EPSILON;
	float NdotH = clamp(dot(l, h), 0., 1.);
	float NdotL = clamp(dot(n, l), 0., 1.) + EPSILON;
    float LdotV = clamp(dot(l, v), 0., 1.);

    float lightScatter = 1 - pow(1 - NdotL, 5);
    float viewScatter = 1 - pow(1 - NdotV, 5);
    float k_facing = 0.5 + 0.5 * LdotV;

    vec3 f_smooth = 21./20 * (1 - F0) * lightScatter * viewScatter;
    float f_rough = k_facing * (0.9 - 0.4 * k_facing) * ((0.5 + NdotH) / NdotH);
    float f_multi = 0.3641 * roughness;

    return HeavisideStepFunction(NdotL) * HeavisideStepFunction(NdotV) *
        albedo / M_PI *
        ((1. - roughness) * f_smooth + roughness * f_rough + albedo * f_multi);
}

// Cook Torrance BRDF
// Specular BRDF components

float GGX(vec3 n, vec3 h, float roughness)
{
    float NoH = dot(n, h);
    float lambda = max(0, NoH);
    float roughnessSq = roughness * roughness;

    return (lambda * roughnessSq) /
        M_PI * (1 + pow(NoH, 2) * pow(roughnessSq - 1, 2));
}

float CombinedSmithGGXMaskingShadowingFunction(vec3 n, vec3 l, vec3 v, float roughness)
{
    float u0 = max(EPSILON, dot(n, l));
    float ui = max(EPSILON, dot(n, v));
    float roughnessSq = pow(roughness, 2);

    return 0.5 / (
        u0 * sqrt(roughnessSq + ui*(ui - roughnessSq * ui)) +
        ui * sqrt(roughnessSq + u0*(u0 - roughnessSq * u0))
    );
}

vec3 SchlickFresnel(vec3 n, vec3 l, vec3 F0)
{
    return F0 + (1 - F0) * pow(1 - max(0, dot(n, l)), 5);
}

vec3 CookTorrance(vec3 n, vec3 l, vec3 v, vec3 h, float roughness, vec3 F0)
{
    return SchlickFresnel(n, l, F0) *
        CombinedSmithGGXMaskingShadowingFunction(n, l, v, roughness) *
        GGX(n, h, roughness);
}

//Learn OpenGL PBR

float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
	
    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = M_PI * denom * denom;
	
    return num / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return num / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
	
    return ggx1 * ggx2;
} 

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}   

void main()
{
    vec3 point_lights[] = {
        vec3(-10.0f,  10.0f, 10.0f),
        vec3( 10.0f,  10.0f, 10.0f),
        vec3(-10.0f, -10.0f, 10.0f),
        vec3( 10.0f, -10.0f, 10.0f),
    };
    vec3 light_colors[] = {
        vec3(300.0f, 300.0f, 300.0f),
        vec3(300.0f, 300.0f, 300.0f),
        vec3(300.0f, 300.0f, 300.0f),
        vec3(300.0f, 300.0f, 300.0f)
    };

    vec3 albedo = uScalarAlbedoVec3f;
    float roughness = uScalarRoughnessVec1f + 0.01;
    float metalness = uScalarMetalnessVec1f + 0.01;

    vec3 n = normalize(normalWorld);
    vec3 v = normalize(cameraPositionWorld - positionWorld);
    vec3 r = normalize(reflect(v, n));

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metalness);
    
    vec3 Lo = vec3(0);
    for (int i = 0; i < 4 ; ++i)
    {
        vec3 l = normalize(point_lights[i] - positionWorld);
        vec3 h = normalize(l + v);

        vec3 radiance = light_colors[i] / pow(length(point_lights[i] - positionWorld), 2);

        vec3 kS = SchlickFresnel(n, l, F0);
        vec3 kD = 1.0 - kS;
        kD *= 1.0 - metalness;

        float NDF = DistributionGGX(n, h, roughness);        
        float G = GeometrySmith(n, v, l, roughness);      
        vec3 F = fresnelSchlick(max(dot(h, v), 0.0), F0);  

        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(n, v), 0.0) * max(dot(n, l), 0.0);
        vec3 specular = numerator / max(denominator, 0.001); 

        Lo += (kD * albedo / M_PI + specular) * radiance * max(0, dot(n, l));
    }

    outColor = vec4(Lo, 1);
}