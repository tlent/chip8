#version 330 core
in vec2 texCoord;

out vec4 FragColor;

uniform usampler2D textureSampler;

void main()
{
    float c = texture(textureSampler, texCoord).r;
    FragColor = vec4(c, c, c, 1.0);
}
