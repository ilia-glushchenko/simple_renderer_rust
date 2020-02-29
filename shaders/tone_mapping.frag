#version 460

layout (binding = 0, location = 30) uniform sampler2D uColorSampler2D;
layout (location = 0) in vec2 uv;
layout (location = 0) out vec4 outColor;

void main()
{
    vec3 color = texture(uColorSampler2D, uv, 0).rgb;
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));
    outColor = vec4(color, 1);
}