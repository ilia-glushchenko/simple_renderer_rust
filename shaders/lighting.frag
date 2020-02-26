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
layout (binding = 6, location = 35) uniform samplerCube uSkyboxSamplerCube;

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

float HeavisideStepFunction(float s)
{
    return s <= 0.0 ? 0.0 : 1.0;
}

float SchlickFresnel(float f0, float f90, float u)
{
    return f0 + (f90 - f0) * pow(1. - u, 5.);
}

//Moving Frostbite to Physically Based Rendering 3.0 (page 10)
float DisneyDiceDiffuse(vec3 n, vec3 l, vec3 v, vec3 h, float roughness, float f0)
{
	float lin_roughness = 1 - pow(1 - 0.7 * roughness, 4);

	float NdotV = abs(dot(n, v)) + EPSILON;
	float LdotH = clamp(dot(l, h), 0., 1.);
	float NdotL = clamp(dot(n, l), 0., 1.);

    float energyBias = mix(0.0, 0.5, lin_roughness);
    float energyFactor = mix(1.0, 1.0 / 1.51, lin_roughness);
    float fd90 = energyBias + 2.0 * LdotH * LdotH * lin_roughness;
    float lightScatter = SchlickFresnel(f0, fd90, NdotL);
    float viewScatter = SchlickFresnel(f0, fd90, NdotV);

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

void lighting()
{
    vec3 pointLightColor = vec3(1, 0.8, 0.5);
    //const vec3 pointLightPositionWorld = vec3(0, 1000, -150);
    const vec3 pointLightPositionWorld = vec3(50, 50, 50);

    //vec4 albedo = texture(uAlbedoMapSampler2D, uv, 0);
    //float roughness = texture(uRoughnessSampler2D, uv, 0).r;

    vec4 albedo = vec4(vec3(1, 1, 1), 1);
    float roughness = 0;//uScalarRoughnessVec1f;

	// // derivations of the fragment position
	// vec3 pos_dx = dFdx( positionWorld );
	// vec3 pos_dy = dFdy( positionWorld );
	// // derivations of the texture coordinate
	// vec2 texC_dx = dFdx( uv );
	// vec2 texC_dy = dFdy( uv );
	// // tangent vector and binormal vector
	// vec3 t = texC_dy.y * pos_dx - texC_dx.y * pos_dy;
	// vec3 b = texC_dx.x * pos_dy - texC_dy.x * pos_dx;

    // const mat3 TBN = transpose(
    //     mat3(normalize(t), normalize(b), normalize(normalWorld)));
    // vec3 n = normalize(texture(uNormalMapSampler2D, uv, 0).rgb * 2 - 1.0);
    vec3 n = normalize(normalWorld);
    vec3 l = normalize(pointLightPositionWorld - positionWorld);
    vec3 v = normalize(cameraPositionWorld - positionWorld);
    vec3 h = normalize(l + v);
    float r = length(pointLightPositionWorld - positionWorld);
    vec3 F0 = vec3(0.562, 0.565, 0.578); //Iron
    //float radiance = ClampPunctualLightRadiance(r, 1);

    //Disney diffuse
    //vec3 diffuse_radiance = albedo.xyz * radiance * DisneyDiceDiffuse(n, l, v, h, roughness, F0);
    //Shirley diffuse
    //vec3 diffuse_radiance = albedo.xyz * radiance * ShirleyDiffuse(n, l, v, roughness, F0);
    //Hammon diffuse
    vec3 diffuse_radiance = HammonDiffuse(n, l, v, h, roughness, F0, albedo.xyz) * 1;
    vec3 specular_radiance = pointLightColor * 1
        * CookTorrance(n, l, v, h, roughness, F0);
    vec3 outgoing_radiance = (diffuse_radiance + specular_radiance) * max(0, dot(n, l));

    outColor = vec4(outgoing_radiance, 1);
    //outColor = vec4(uScalarAlbedoVec3f, 1);
}

void main()
{
    vec3 albedo = uScalarAlbedoVec3f;
    float roughness = 1; uScalarRoughnessVec1f;
    float metalness = 1; uScalarMetalnessVec1f;

    vec3 n = normalize(normalWorld);
    vec3 v = normalize(cameraPositionWorld - positionWorld);
    vec3 r = normalize(reflect(v, n));

    //vec3 l = r;
    vec3 l = normalize(vec3(0, 0, 1));
    vec3 h = normalize(l + v);
    vec3 F0 = vec3(0.562, 0.565, 0.578); //Iron;

    vec3 specular = texture(uSkyboxSamplerCube, r, 0).rgb;

    //Lambert
	// float lin_roughness = 1 - pow(1 - 0.7 * roughness, 4);
	// float NdotV = abs(dot(n, v)) + EPSILON;
	// float LdotH = clamp(dot(l, h), 0., 1.);
	// float NdotL = clamp(dot(n, l), 0., 1.);
    // float energyBias = mix(0.0, 0.5, lin_roughness);
    // float energyFactor = mix(1.0, 1.0 / 1.51, lin_roughness);
    // float fd90 = energyBias + 2.0 * LdotH * LdotH * lin_roughness;
    // float lightScatter = SchlickFresnel(F0, fd90, NdotL);
    // vec3 diffuse_radiance = (1 - lightScatter) * albedo / M_PI;

    //Disney diffuse
    //vec3 diffuse_radiance = albedo * DisneyDiceDiffuse(n, l, v, h, roughness, F0);

    //Shirley diffuse
    //vec3 diffuse_radiance = albedo * ShirleyDiffuse(n, l, v, roughness, F0);

    //Hammon diffuse
    // vec3 diffuse_radiance = HammonDiffuse(n, l, v, h, roughness, F0, albedo);
    vec3 diffuse_radiance = HammonDiffuse(n, l, v, h, roughness, F0, albedo);

    // vec3 specular_radiance = specular * CookTorrance(n, l, v, h, roughness, F0);
    vec3 specular_radiance = specular * CookTorrance(n, r, v, n, roughness, F0);

    outColor = vec4((diffuse_radiance + specular_radiance) * max(0, dot(n, l)), 1);

    vec3 kS = F0;
    vec3 kD = 1.0 - kS;
    kD *= 1.0 - metalness;

    vec3 color = (kD * diffuse_radiance  + specular_radiance);
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));
    outColor = vec4(color, 1);
}