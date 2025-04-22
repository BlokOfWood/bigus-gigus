#version 450

#extension GL_EXT_debug_printf : enable

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    debugPrintfEXT("Model matrix: %f %f %f %f", ubo.model[0][0], ubo.model[1][0], ubo.model[2][0], ubo.model[3][0]);
    debugPrintfEXT("View matrix: %f %f %f %f", ubo.view[0][0], ubo.view[1][0], ubo.view[2][0], ubo.view[3][0]);
    debugPrintfEXT("Projection matrix: %f %f %f %f", ubo.proj[0][0], ubo.proj[1][0], ubo.proj[2][0], ubo.proj[3][0]);

    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPosition, 0.0, 1.0);
    fragColor = inColor;
}