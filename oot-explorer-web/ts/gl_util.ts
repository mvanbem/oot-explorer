export function glInitShader(gl: WebGL2RenderingContext, type: number, source: string) {
    try {
        let shader = gl.createShader(type)!;
        gl.shaderSource(shader, source);
        gl.compileShader(shader);
        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
            throw new Error('failed to compile shader: ' + gl.getShaderInfoLog(shader));
        }
        return shader;
    } catch (e) {
        console.log('shader source:\n' + source);
        throw e;
    }
}

export function glInitProgram(gl: WebGL2RenderingContext, vertexSource: string, fragmentSource: string) {
    let program = gl.createProgram()!;
    gl.attachShader(program, glInitShader(gl, gl.VERTEX_SHADER, vertexSource));
    gl.attachShader(program, glInitShader(gl, gl.FRAGMENT_SHADER, fragmentSource));
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        throw new Error('failed to link program: ' + gl.getProgramInfoLog(program));
    }
    return program;
}
