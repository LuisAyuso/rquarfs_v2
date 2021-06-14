#version 430

in vec2 position;

smooth out vec2 tex_coords;

const vec2 vertices[4] = vec2[4](
        vec2(-1.0,-1.0),
        vec2(-1.0, 1.0),
        vec2( 1.0, 1.0),
        vec2( 1.0,-1.0)
);

const vec2 texture_coords[4] = vec2[4](
        vec2( 0.0, 1.0),
        vec2( 0.0, 0.0),
        vec2( 1.0, 0.0),
        vec2( 1.0, 1.0)
);

const uint indices[6] = uint[6]( 0,1,2,0,2,3);

void main() {
    gl_Position = vec4(vertices[indices[gl_VertexID]] ,0.0, 1.0); 
    tex_coords = texture_coords[indices[gl_VertexID]];
}
    