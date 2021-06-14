
    #version 430

    uniform sampler2D image;
    smooth in vec2 tex_coords;
    out vec4 frag_color;
    
    void main() {
        frag_color = texture(image, tex_coords);
    }