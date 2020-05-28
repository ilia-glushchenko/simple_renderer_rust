#version 460

layout (binding = 0, location = 30) uniform sampler2D uColorSampler2D;
layout (location = 0) in vec2 uv;
layout (location = 0) out vec4 outColor;

// void main()
// {
//     vec3 color = texture(uColorSampler2D, uv, 0).rgb;
//     color = color / (color + vec3(1.0));
//     color = pow(color, vec3(1.0/2.2));
//     outColor = vec4(color, 1);
// }

//=================================================================================================
//
//  Baking Lab
//  by MJP and David Neubelt
//  http://mynameismjp.wordpress.com/
//
//  All code licensed under the MIT license
//
//=================================================================================================

// The code in this file was originally written by Stephen Hill (@self_shadow), who deserves all
// credit for coming up with this fit and implementing it. Buy him a beer next time you see him. :)

vec3 acesFilm(const vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d ) + e), 0.0, 1.0);
}

void main()
{
    outColor = vec4(acesFilm(texture(uColorSampler2D, uv, 0).rgb), 1);
}