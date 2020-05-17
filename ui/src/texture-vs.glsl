in vec2 position;

out vec2 v_uv;

void main() {
  gl_Position = vec4(position * 1, 0, 1.);
  v_uv = position * .5 + .5; // transform the position of the vertex into UV space
}
