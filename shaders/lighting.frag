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
layout (binding = 5, location = 35) uniform samplerCube uDiffuseSamplerCube;
layout (binding = 6, location = 36) uniform sampler2D uBrdfLUTSampler2D;
layout (binding = 7, location = 37) uniform samplerCube uEnvMapSamplerCube;

layout (location = 0) in vec2 uv;
layout (location = 1) in vec3 normalWorld;
layout (location = 2) in vec3 positionWorld;
layout (location = 3) in vec3 cameraPositionWorld;

layout (location = 0) out vec4 outColor;

#define M_PI 3.1415926535897932384626433832795
#define EPSILON 1e-5

float ClampPunctualLightRadiance(float r, float radiance)
{
    float r0 = 2;
    float r_min = 1;
    float r_max = 100;

    float win = pow(max(0, 1 - pow(r / r_max, 4)), 2);
    float attenuated_radiance = radiance * (r0 * r0) / pow(max(r, r_min), 2) * win;

    return attenuated_radiance;
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
float HeavisideStepFunction(float s)
{
    return s <= 0.0 ? 0.0 : 1.0;
}

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

// Unreal CookTorrance PBR (from Learn OpenGL)
vec3 FresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

vec3 FresnelSchlickRoughness(float cosTheta, vec3 F0, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(1.0 - cosTheta, 5.0);
}

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

vec3 UnrealCookTorrance(vec3 n, vec3 l, vec3 v, vec3 h, float roughness, vec3 F0)
{
    float NDF = DistributionGGX(n, h, roughness);
    float G = GeometrySmith(n, v, l, roughness);
    vec3 F = FresnelSchlick(max(dot(h, v), 0.0), F0);

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(n, v), 0.0) * max(dot(n, l), 0.0);
    vec3 specular = numerator / max(denominator, 0.001);

    return specular;
}

// Frostbite CookTorrance PBR

float F_Schlick(float f0, float f90, float u)
{
    return f0 + (f90 - f0) * pow(1. - u, 5.);
}

vec3 F_Schlick3(in vec3 f0, in float f90, in float u)
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
    float lightScatter = F_Schlick3(F0, fd90, NdotL).r;
    float viewScatter = F_Schlick3(F0, fd90, NdotV).r;

    return lightScatter * viewScatter * energyFactor;
}

float V_SmithGGXCorrelated(float NdotL, float NdotV, float alphaG)
{
    float alphaG2 = alphaG * alphaG;
    float Lambda_GGXV = NdotL * sqrt((-NdotV * alphaG2 + NdotV) * NdotV + alphaG2);
    float Lambda_GGXL = NdotV * sqrt((-NdotL * alphaG2 + NdotL) * NdotL + alphaG2);

    return 0.5f / (Lambda_GGXV + Lambda_GGXL);
}

float D_GGX(float NdotH, float m)
{
    float m2 = m * m;
    float f = (NdotH * m2 - NdotH) * NdotH + 1;
    return m2 / (f * f);
}

vec3 ForstbiteCookTorrance(vec3 n, vec3 l, vec3 v, vec3 h, float alpha, vec3 F0)
{
    float NdotV = abs(dot(n, v)) + EPSILON;
    float LdotH = clamp(dot(l, h), 0.f, 1.f);
    float NdotH = clamp(dot(n, h), 0.f, 1.f);
    float NdotL = clamp(dot(n, l), 0.f, 1.f);

    float f90 = 0.5f + pow(max(dot(h, v), 0.0), 2) * alpha;
    vec3 F = F_Schlick3(F0, f90, LdotH);
    float Vis = V_SmithGGXCorrelated(NdotL, NdotV, alpha);
    float D = D_GGX(NdotH, alpha);
    vec3 Fr = F * D * Vis / M_PI;

    return Fr;
}

mat3 CalculateTBNMatrix( vec3 N, vec3 p, vec2 pUV )
{
    // get edge vectors of the pixel triangle
    vec3 dp1 = dFdx( p );
    vec3 dp2 = dFdy( p );
    vec2 duv1 = dFdx( pUV );
    vec2 duv2 = dFdy( pUV );

    // solve the linear system
    vec3 dp2perp = cross( dp2, N );
    vec3 dp1perp = cross( N, dp1 );
    vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
    vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;

    // construct a scale-invariant frame
    float invmax = inversesqrt( max( dot(T,T), dot(B,B) ) );
    return mat3( T * invmax, B * invmax, N );
}

void main()
{
    mat3 TBN = CalculateTBNMatrix(normalWorld, positionWorld, uv);

    vec3 pointLightsTBN[] = {
        TBN * vec3(-50.0f,  50.0f, 50.0f),
        TBN * vec3( 50.0f,  50.0f, 50.0f),
        TBN * vec3(-50.0f, -50.0f, 50.0f),
        TBN * vec3( 50.0f, -50.0f, 50.0f),
        TBN * vec3(  0.0f,   0.0f, 50.0f),
    };
    vec3 pointLightColors[] = {
        vec3(1.f, 1.f, 1.f),
        vec3(1.f, 1.f, 1.f),
        vec3(1.f, 1.f, 1.f),
        vec3(1.f, 1.f, 1.f),
        vec3(1.f, 1.f, 1.f)
    };
    float pointLightRadiance[] = {
        1000., 1000., 1000., 1000., 1000.
    };
    vec3 positionTBN = TBN * positionWorld;
    vec3 cameraPositionTBN = TBN * cameraPositionWorld;

    vec3 albedo = texture(uAlbedoMapSampler2D, uv).rgb;
    float metalness = 0; //texture(uMetallicSampler2D, uv).r;
    float roughness = clamp(texture(uRoughnessSampler2D, uv).r, 0.04f, 1.f);

    vec3 n = normalize(texture(uNormalMapSampler2D, uv).rgb);
    n = normalize(n * 2.0 - 1.0);
    // vec3 n = TBN * normalWorld;
    vec3 v = normalize(cameraPositionTBN - positionTBN);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metalness);

    vec3 Lo = vec3(0);
    for (int i = 0; i < 5 ; ++i)
    {
        vec3 l = normalize(pointLightsTBN[i] - positionTBN);
        vec3 h = normalize(l + v);

        vec3 kS = FresnelSchlick(max(dot(h, v), 0.0), F0);
        vec3 kD = (1.0 - kS) * (1.0 - metalness);

        vec3 radiance = pointLightColors[i] * ClampPunctualLightRadiance(
            length(pointLightsTBN[i] - positionTBN), pointLightRadiance[i]);

        Lo += (
            kD * HammonDiffuse(n, l, v, h, roughness, F0, albedo)
            + ForstbiteCookTorrance(n, l, v, h, roughness * roughness, F0)
        ) * radiance * max(0, dot(n, l));
    }

    vec3 kS = FresnelSchlickRoughness(max(dot(n, v), 0.0), F0, roughness);
    vec3 kD = 1.0 - kS;
    kD *= 1.0 - metalness;
    vec3 irradiance = texture(uDiffuseSamplerCube, -n).rgb;
    vec3 diffuse = irradiance * albedo;

    const float MAX_REFLECTION_LOD = 7.0;
    vec3 worldR = normalize(reflect(cameraPositionWorld - positionWorld, normalWorld));
    vec3 prefilteredColor = textureLod(uEnvMapSamplerCube, worldR, roughness * MAX_REFLECTION_LOD).rgb;
    vec3 F        = FresnelSchlickRoughness(max(dot(n, v), 0.0), F0, roughness);
    vec2 envBRDF  = texture(uBrdfLUTSampler2D, vec2(max(dot(n, v), 0.0), roughness)).rg;
    vec3 specular = prefilteredColor * (F * envBRDF.x + envBRDF.y);

    vec3 ambient = (kD * diffuse + specular);
    vec3 color = ambient + Lo;

    outColor = vec4(color, 1);
}