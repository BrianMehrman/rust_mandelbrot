#version 330 core
out vec4 FragColor;
uniform int windowSize;

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    vec2 c = gl_FragCoord.xy / windowSize * 4.0 - 2.0;

    vec2 z = c;
    int max_iterations = 100;
    int i;
    for(i = 0; i < max_iterations; i++) {
        z = vec2((z.x * z.x) - (z.y * z.y), 2.0 * z.x * z.y) + c;
        if (length(z) > 2.0) {
            break;
        }
    }

    if (length(z) <= 2.0) {
        FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        float val = float(i) / float(max_iterations);
        // FragColor = vec4(1.0, 1.0, 1.0, 1.0); // white
        // FragColor = vec4(val, val, val, 1.0); // black and white
        FragColor = vec4(hsv2rgb(vec3(val, 1.0, 1.0)), 1.0); // trippy
    }
}
