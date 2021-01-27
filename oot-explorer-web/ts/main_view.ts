import { vec3, mat4 } from 'gl-matrix'
import type * as wasm from '../pkg';

import { Container } from './container';
import { $t } from './dollar_t';
import { Status } from './status';
import { Wasm, WasmModule } from './wasm';
import { glInitProgram, glInitShader } from './gl_util';

const BACKGROUND_VERTEX_SHADER_SOURCE = `#version 300 es

precision highp float;
precision highp int;

uniform vec2 u_scale;

layout(location = 0) in vec2 vertex;

out vec2 v_texCoord;

void main() {
  gl_Position = vec4((vertex * vec2(2.0) - vec2(1.0)) * u_scale, 0.0, 1.0);
  v_texCoord = vertex;
}
`;

const BACKGROUND_FRAGMENT_SHADER_SOURCE = `#version 300 es

precision highp float;
precision highp int;

uniform sampler2D u_texture;

in vec2 v_texCoord;

layout(location = 0) out vec4 fragColor;

void main() {
  fragColor = texture(u_texture, v_texCoord);
}
`;

const VERTEX_SHADER_SOURCE = `#version 300 es

precision highp float;
precision highp int;

layout(location = 0) in ivec3 vertexPosition;
layout(location = 1) in ivec3 vertexNormal;
layout(location = 2) in uint vertexFlags;
layout(location = 3) in ivec2 vertexTexCoord;
layout(location = 4) in uvec4 vertexColor;

uniform mat4 u_projectionMatrix;
uniform mat4 u_modelViewMatrix;

out vec4 v_shade;
out vec2 v_texCoord;

void main() {
  gl_Position = u_projectionMatrix * u_modelViewMatrix * vec4(vertexPosition, 1.0);

  if ((vertexFlags & 1u) == 1u) {
    // TODO: Lighting
    v_shade = vec4(1.0);
  } else {
    v_shade = vec4(vertexColor) / 255.0;
  }
  v_texCoord = vec2(vertexTexCoord);
}
`;

function addLineNumbers(text: string) {
    return text.split('\n').map((line, i) => (i + 1) + ' | ' + line).join('\n');
}

function parseUrlFragment() {
    let matches = /^#scene=([0-9]+)$/.exec(window.location.hash);
    if (matches === null) {
        return {
            sceneIndex: 0,
        };
    }
    return {
        sceneIndex: parseInt(matches[1], 10) - 1,
    };
}

interface UpdateUrlFragmentParams {
    sceneIndex: number;
}

function updateUrlFragment({ sceneIndex }: UpdateUrlFragmentParams) {
    let url = new URL(window.location.toString());
    url.hash = '#scene=' + (sceneIndex + 1);
    window.location.replace(url.toString());
}

interface View {
    pos: vec3;
    yaw: number;
    pitch: number;
}

interface TouchState {
    x: number;
    y: number;
};

interface MainViewCtorArgs {
    wasm: WasmModule;
    rom: ArrayBuffer;
}

interface Batch {
    program: WebGLProgram;
    vertexBuffer: WebGLBuffer;
    translucent: boolean;
    mode: number;
    count: number;
    textures: BatchTexture[];
    zUpd: boolean;
    decal: boolean;
}

interface BatchTexture {
    texture: WebGLTexture;
    sampler: WebGLSampler;
    width: number;
    height: number;
}

export class MainView {
    canvas: HTMLCanvasElement;
    gl: WebGL2RenderingContext;
    w?: number;
    h?: number;
    ctx: wasm.Context;
    touches: Map<number, TouchState>;
    keys: Map<string, boolean>;
    sceneIndex?: number;
    view: View;
    backgroundProgram: WebGLProgram;
    backgroundVertexBuffer: WebGLBuffer;
    batches: Batch[];
    currentResolves: (() => void)[];
    nextResolves: (() => void)[];
    backgrounds: WebGLTexture[];
    prevTimestamp?: number;

    constructor({ wasm, rom }: MainViewCtorArgs) {
        this.canvas = $t('canvas');
        let gl = this.gl = this.canvas.getContext(
            'webgl2', {
            alpha: false,
            antialias: false,
            depth: true,
            stencil: false,
        })!;

        document.getElementById('explore')!.addEventListener('click', () => {
            let exploreView = wasm.ExploreView.new(document, this.ctx);
            this.canvas.parentElement!.appendChild(exploreView.element);
        });

        this.canvas.addEventListener('mousemove', e => {
            if (e.buttons & 1) {
                this.view.yaw -= 0.005 * e.movementX;
                this.view.pitch -= 0.005 * e.movementY;
            }
        });
        this.touches = new Map();
        this.canvas.addEventListener('touchstart', e => {
            for (let touch of e.changedTouches) {
                this.touches.set(touch.identifier, { x: touch.clientX, y: touch.clientY });
            }
            e.preventDefault();
        });
        this.canvas.addEventListener('touchmove', e => {
            for (let touch of e.changedTouches) {
                if (this.touches.has(touch.identifier)) {
                    let old = this.touches.get(touch.identifier)!;
                    this.view.yaw -= 0.005 * (touch.clientX - old.x);
                    this.view.pitch -= 0.005 * (touch.clientY - old.y);
                    this.touches.set(touch.identifier, { x: touch.clientX, y: touch.clientY });
                } else {
                    console.log('touchmove without touchstart');
                }
            }
        });
        this.canvas.addEventListener('touchend', e => {
            for (let touch of e.changedTouches) {
                this.touches.delete(touch.identifier);
            }
        });

        this.keys = new Map();
        for (let key of [
            'KeyA',
            'KeyD',
            'KeyE',
            'KeyQ',
            'KeyS',
            'KeyW',
            'ShiftLeft',
        ]) {
            this.keys.set(key, false);
        }
        window.addEventListener('keydown', e => {
            let key = e.code;
            if (this.keys.has(key)) {
                this.keys.set(key, true);
                e.preventDefault();
                return;
            }

            if (key === 'PageDown') {
                if (this.sceneIndex !== undefined) {
                    let newSceneIndex = this.sceneIndex + 1;
                    if (newSceneIndex < this.ctx.sceneCount) {
                        this.changeScene(newSceneIndex);
                    }
                }
            } else if (key === 'PageUp') {
                if (this.sceneIndex !== undefined) {
                    let newSceneIndex = this.sceneIndex - 1;
                    if (newSceneIndex >= 0) {
                        this.changeScene(newSceneIndex);
                    }
                }
            }
        });
        window.addEventListener('keyup', e => {
            let key = e.code;
            if (this.keys.has(key)) {
                this.keys.set(key, false);
                e.preventDefault();
            }
        });
        window.addEventListener('blur', e => {
            for (let key of Array.from(this.keys.keys())) {
                this.keys.set(key, false);
            }
        });
        window.addEventListener('hashchange', e => {
            let { sceneIndex } = parseUrlFragment();
            if (sceneIndex !== this.sceneIndex) {
                this.changeScene(sceneIndex);
            } else {
                // Canonicalize it.
                updateUrlFragment({ sceneIndex });
            }
        });

        this.view = {
            pos: vec3.clone([-4, 50, 603]),
            yaw: 0,
            pitch: 0,
        };

        this.backgroundProgram = glInitProgram(gl, BACKGROUND_VERTEX_SHADER_SOURCE,
            BACKGROUND_FRAGMENT_SHADER_SOURCE);
        this.backgroundVertexBuffer = (() => {
            let data = new Float32Array(12);
            data.set([
                0, 1,
                1, 0,
                0, 0,

                0, 1,
                1, 1,
                1, 0,
            ]);

            let buffer = gl.createBuffer()!;
            gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
            gl.bufferData(gl.ARRAY_BUFFER, data.buffer, gl.STATIC_DRAW);
            return buffer;
        })();

        this.ctx = new wasm.Context(this.gl, new Uint8Array(rom));
        this.batches = [];
        this.currentResolves = [];
        this.nextResolves = [];
        this.backgrounds = [];

        this.changeScene(parseUrlFragment().sceneIndex);

        window.requestAnimationFrame(timestamp => this.step(timestamp));
    }

    nextStep() {
        return new Promise<void>(resolve => this.nextResolves.push(resolve));
    }

    async changeScene(sceneIndex: number) {
        let gl = this.gl;
        this.sceneIndex = sceneIndex;
        document.getElementById('scene')!.textContent =
            'Scene: ' + (sceneIndex + 1) + '/' + this.ctx.sceneCount;
        updateUrlFragment({ sceneIndex });

        Status.show('Processing scene...');
        await this.nextStep();

        let processedScene = <Wasm.ProcessSceneResult>this.ctx.processScene(sceneIndex);

        // Compile shaders for all batches.
        let vertexShader = glInitShader(gl, gl.VERTEX_SHADER, VERTEX_SHADER_SOURCE);
        let fragmentShaders = [];
        for (let batch of processedScene.batches) {
            let shader = gl.createShader(gl.FRAGMENT_SHADER)!;
            gl.shaderSource(shader, batch.fragmentShader);
            gl.compileShader(shader);
            fragmentShaders.push(shader);
        }

        // Load vertex buffers and textures for all batches.
        let vertexBuffers = [];
        let textures = [];
        for (let batch of processedScene.batches) {
            let buffer = gl.createBuffer()!;
            gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
            gl.bufferData(gl.ARRAY_BUFFER, batch.vertexData, gl.STATIC_DRAW);
            vertexBuffers.push(buffer);

            let batch_textures = [];
            for (let texture of batch.textures) {
                batch_textures.push({
                    texture: this.ctx.getTexture(texture.textureKey)!,
                    sampler: this.ctx.getSampler(texture.samplerKey)!,
                    width: texture.width,
                    height: texture.height,
                });
            }
            textures.push(batch_textures);
        }

        // Link programs for all batches.
        let programs = [];
        for (let i = 0; i < processedScene.batches.length; ++i) {
            let program = gl.createProgram()!;
            gl.attachShader(program, vertexShader);
            gl.attachShader(program, fragmentShaders[i]);
            gl.linkProgram(program);
            programs.push(program);
        }

        // Assemble all batches to be drawn.
        let opaqueBatches: Batch[] = [];
        let translucentBatches: Batch[] = [];
        for (let i = 0; i < processedScene.batches.length; ++i) {
            if (!gl.getProgramParameter(programs[i], gl.LINK_STATUS)) {
                console.log('program info log:', gl.getProgramInfoLog(programs[i]));
                console.log('vertex shader info log:', gl.getShaderInfoLog(vertexShader));
                console.log('fragment shader info log:', gl.getShaderInfoLog(fragmentShaders[i]));
                console.log('fragment shader source:\n' + addLineNumbers(gl.getShaderSource(fragmentShaders[i])!));
                throw new Error('failed to link GL program');
            }

            let batch = processedScene.batches[i];
            let collection = batch.translucent ? translucentBatches : opaqueBatches;
            collection.push({
                program: programs[i],
                vertexBuffer: vertexBuffers[i],
                translucent: batch.translucent,
                mode: gl.TRIANGLES,
                count: batch.vertexData.byteLength / 20,
                textures: textures[i],
                zUpd: batch.zUpd,
                decal: batch.decal,
            });
        }

        let backgrounds = [];
        for (let i = 0; i < processedScene.backgrounds.length; ++i) {
            let img = document.createElement('img');
            img.src = processedScene.backgrounds[i];
            let bitmap = await new Promise<ImageBitmap>(resolve => {
                img.onload = () => resolve(createImageBitmap(img));
            });

            let texture = gl.createTexture()!;
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, bitmap);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
            gl.generateMipmap(gl.TEXTURE_2D);
            backgrounds.push(texture);
        }
        if (processedScene.backgrounds.length > 1) {
            console.log('WARNING: found ' + processedScene.backgrounds.length
                + ' backgrounds in this scene!');
        }

        // Publish all new data.
        this.batches = (<Batch[]>[]).concat(opaqueBatches, translucentBatches);
        this.backgrounds = backgrounds;
        if (processedScene.startPos) {
            this.view = {
                pos: vec3.clone([
                    processedScene.startPos[0],
                    processedScene.startPos[1] + 50,
                    processedScene.startPos[2],
                ]),
                yaw: (processedScene.startPos[4] + Math.PI) % (2 * Math.PI),
                pitch: processedScene.startPos[3],
            };
        }

        Status.hide();
    }

    updateDimensions(): [number, number] {
        let r = Container.getBoundingClientRect();
        let width = (r.width * window.devicePixelRatio) | 0;
        let height = (r.height * window.devicePixelRatio) | 0;

        if (width != this.w || height != this.h) {
            this.canvas.width = this.w = width;
            this.canvas.height = this.h = height;
            this.canvas.style.width = r.width + 'px';
            this.canvas.style.height = r.height + 'px';
        }

        return [this.w, this.h];
    }

    step(timestamp: number) {
        // Trigger anything that was waiting for a new frame.
        for (let resolve of this.currentResolves) {
            resolve();
        }
        this.currentResolves = this.nextResolves;
        this.nextResolves = [];

        let [w, h] = this.updateDimensions();

        if (this.prevTimestamp !== undefined) {
            let dt = (timestamp - this.prevTimestamp) / 1000;
            let motion = vec3.create();
            if (this.keys.get('KeyW')) {
                vec3.add(motion, motion, [0, 0, -1]);
            }
            if (this.keys.get('KeyS')) {
                vec3.add(motion, motion, [0, 0, 1]);
            }
            if (this.keys.get('KeyA')) {
                vec3.add(motion, motion, [-1, 0, 0]);
            }
            if (this.keys.get('KeyD')) {
                vec3.add(motion, motion, [1, 0, 0]);
            }
            if (this.keys.get('KeyE')) {
                vec3.add(motion, motion, [0, 1, 0]);
            }
            if (this.keys.get('KeyQ')) {
                vec3.add(motion, motion, [0, -1, 0]);
            }
            if (this.keys.get('ShiftLeft')) {
                vec3.scale(motion, motion, 5);
            }
            let viewMatrix = mat4.create();
            mat4.fromYRotation(viewMatrix, this.view.yaw);
            vec3.transformMat4(motion, motion, viewMatrix);
            vec3.scaleAndAdd(this.view.pos, this.view.pos, motion, 500 * dt);
        }
        this.prevTimestamp = timestamp;

        let gl = this.gl;
        gl.clearColor(0.5, 1, 1, 1);
        gl.clearDepth(1);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        gl.viewport(0, 0, w, h);

        // Draw the background, if any.
        if (this.backgrounds && this.backgrounds.length > 0) {
            gl.useProgram(this.backgroundProgram);
            gl.uniform2f(
                gl.getUniformLocation(this.backgroundProgram, "u_scale"),
                Math.min(1, h * 4 / 3 / w),
                -Math.min(1, w * 3 / 4 / h),
            );

            gl.depthMask(false);
            gl.disable(gl.DEPTH_TEST);
            gl.disable(gl.CULL_FACE);
            gl.disable(gl.BLEND);

            gl.bindBuffer(gl.ARRAY_BUFFER, this.backgroundVertexBuffer);
            gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 8, 0);
            gl.enableVertexAttribArray(0);
            gl.disableVertexAttribArray(1);
            gl.disableVertexAttribArray(2);
            gl.disableVertexAttribArray(3);
            gl.disableVertexAttribArray(4);

            gl.activeTexture(gl.TEXTURE0);
            gl.bindSampler(0, null);
            gl.bindTexture(gl.TEXTURE_2D, this.backgrounds[0]);
            gl.uniform1i(gl.getUniformLocation(this.backgroundProgram, "u_texture"), 0);

            gl.drawArrays(gl.TRIANGLES, 0, 6);
        }

        gl.depthFunc(gl.LEQUAL);
        gl.frontFace(gl.CCW);

        let projectionMatrix = mat4.create();
        mat4.perspective(projectionMatrix, 0.5 * Math.PI, w / h, 1, 20000.0);

        let modelViewMatrix = mat4.create();
        mat4.rotateX(modelViewMatrix, modelViewMatrix, -this.view.pitch);
        mat4.rotateY(modelViewMatrix, modelViewMatrix, -this.view.yaw);
        {
            let npos = vec3.create();
            vec3.negate(npos, this.view.pos);
            mat4.translate(modelViewMatrix, modelViewMatrix, npos);
        }

        for (let batch of this.batches || []) {
            gl.useProgram(batch.program);
            gl.enable(gl.DEPTH_TEST);

            if (batch.translucent) {
                gl.disable(gl.CULL_FACE);
                // TODO: This is extremely fake. The RDP has blending parameters.
                gl.enable(gl.BLEND);
                gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
            } else {
                gl.enable(gl.CULL_FACE);
                gl.disable(gl.BLEND);
            }
            gl.depthMask(batch.zUpd);
            if (batch.decal) {
                gl.enable(gl.POLYGON_OFFSET_FILL);
                gl.polygonOffset(-1, -1);
            } else {
                gl.disable(gl.POLYGON_OFFSET_FILL);
            }

            gl.uniformMatrix4fv(
                gl.getUniformLocation(batch.program, 'u_projectionMatrix'),
                false,
                projectionMatrix);
            gl.uniformMatrix4fv(
                gl.getUniformLocation(batch.program, 'u_modelViewMatrix'),
                false,
                modelViewMatrix);
            gl.uniform1i(gl.getUniformLocation(batch.program, "u_texture0"), 0);
            gl.uniform1i(gl.getUniformLocation(batch.program, "u_texture1"), 1);

            gl.bindBuffer(gl.ARRAY_BUFFER, batch.vertexBuffer);
            // Position
            gl.vertexAttribIPointer(0, 3, gl.SHORT, 20, 0);
            gl.enableVertexAttribArray(0);
            // Normal
            gl.vertexAttribIPointer(1, 3, gl.BYTE, 20, 8);
            gl.enableVertexAttribArray(1);
            // Flags
            gl.vertexAttribIPointer(2, 1, gl.UNSIGNED_BYTE, 20, 11);
            gl.enableVertexAttribArray(2);
            // Texture coordinates
            gl.vertexAttribIPointer(3, 2, gl.SHORT, 20, 12);
            gl.enableVertexAttribArray(3);
            // Color
            gl.vertexAttribIPointer(4, 4, gl.UNSIGNED_BYTE, 20, 16);
            gl.enableVertexAttribArray(4);

            for (let i = 0; i < 2; ++i) {
                gl.activeTexture(gl.TEXTURE0 + i);
                let texture = batch.textures[i];
                if (texture && texture.texture) {
                    gl.bindTexture(gl.TEXTURE_2D, texture.texture);
                    gl.bindSampler(i, texture.sampler);
                } else {
                    gl.bindTexture(gl.TEXTURE_2D, null);
                    gl.bindSampler(i, null);
                }
            }

            gl.drawArrays(batch.mode, 0, batch.count);
        }

        let error = gl.getError();
        if (error !== gl.NO_ERROR) {
            console.log('gl error: ' + error);
        }

        window.requestAnimationFrame(this.step.bind(this));
    }
}