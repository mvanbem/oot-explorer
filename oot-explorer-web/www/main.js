import * as core from './oot_explorer_web.js';

let vec3 = glMatrix.vec3;
let mat4 = glMatrix.mat4;

function $t(name, params) {
  let e = document.createElement(name);
  for (let key in params) {
    let value = params[key];
    if (key === 'children') {
      value.map(child => e.appendChild(child));
    } else {
      e[key] = value;
    }
  }
  return e;
}

window.addEventListener('error', e => {
  Status.show('top-level error: ' + e.message);
});

window.addEventListener('DOMContentLoaded', async () => {
  document.getElementById('clear-button').addEventListener('click', async () => {
    await RomStorage.clear();
    window.location.reload();
  });

  // This definitely happens before any MainView is constructed, so MainView is free to assume core
  // is safe to use.
  await core.default();

  let rom = await RomStorage.load();
  if (rom === null) {
    Container.setView(new RomView().element);
  } else {
    Container.setView(new MainView({ rom }).canvas);
  }
});

const Container = (() => {
  class Container {
    constructor() {
      this.element = document.getElementById('container');
    }

    setView(view) {
      while (this.element.lastChild !== null) {
        this.element.removeChild(this.element.lastChild);
      }
      this.element.appendChild(view);
    }

    getBoundingClientRect() {
      return this.element.getBoundingClientRect();
    }
  }

  return new Container();
})();

const Status = (() => {
  class Status {
    constructor() {
      this.element = document.getElementById('status');
    }

    show(msg) {
      this.element.className = '';
      this.element.textContent = msg;
    }

    hide() {
      this.element.className = 'hidden';
    }
  }

  return new Status();
})();

const RomStorage = (() => {
  const DATABASE_NAME = 'rom';
  const OBJECT_STORE_NAME = 'rom';
  const KEY = 'rom';

  class RomStorage {
    constructor() {
      this.dbPromise = null;
    }

    getDatabase() {
      if (!this.dbPromise) {
        this.dbPromise = new Promise((resolve, reject) => {
          let req = window.indexedDB.open(DATABASE_NAME, 1);
          req.addEventListener('success', () => resolve(req.result));
          req.addEventListener('error', () => reject(req.errorCode));
          req.addEventListener('upgradeneeded', () => {
            let db = req.result;
            let store = db.createObjectStore(OBJECT_STORE_NAME);
          });
        });
      }
      return this.dbPromise;
    }

    async load() {
      Status.show('Checking IndexedDB for stored ROM...');
      const db = await this.getDatabase();
      const rom = await new Promise((resolve, reject) => {
        let txn = db.transaction([OBJECT_STORE_NAME], 'readwrite');
        txn.addEventListener('complete', () => resolve(req.result || null));
        txn.addEventListener('error', () => reject(txn.errorCode));
        txn.addEventListener('abort', () => reject(new Error('transaction aborted')));
        let req = txn.objectStore(OBJECT_STORE_NAME).get(KEY);
      });
      if (rom === null) {
        return null;
      }
      if (!this.isValid(rom)) {
        console.log('invalid rom stored; ignoring it');
        return null;
      }
      return rom;
    }

    // [rom] is expected to be an ArrayBuffer
    async store(rom) {
      let messages = [];
      if (!this.isValid(rom, messages)) {
        throw new Error('ROM failed validation: ' + messages.join('; '));
      }

      Status.show('Storing ROM to IndexedDB...');
      const db = await this.getDatabase();
      return await new Promise((resolve, reject) => {
        let txn = db.transaction([OBJECT_STORE_NAME], 'readwrite');
        txn.addEventListener('complete', () => resolve());
        txn.addEventListener('error', () => reject(txn.errorCode));
        txn.addEventListener('abort', () => reject(new Error('transaction aborted')));
        txn.objectStore(OBJECT_STORE_NAME).put(rom, KEY);
      });
    }

    async clear() {
      Status.show('Clearing IndexedDB...');
      const db = await this.getDatabase();
      return await new Promise((resolve, reject) => {
        let txn = db.transaction([OBJECT_STORE_NAME], 'readwrite');
        txn.addEventListener('complete', () => resolve());
        txn.addEventListener('error', () => reject(txn.errorCode));
        txn.addEventListener('abort', () => reject(new Error('transaction aborted')));
        txn.objectStore(OBJECT_STORE_NAME).delete(KEY);
      });
    }

    isValid(rom, outMessages) {
      let header = new RomHeader(rom);
      let pass = true;

      const IMAGE_NAME = 'THE LEGEND OF ZELDA ';
      if (header.imageName !== IMAGE_NAME) {
        pass = false;
        let message = 'bad image name: want ' + JSON.stringify(IMAGE_NAME)
          + ' but got ' + JSON.stringify(header.imageName);
        if (outMessages) {
          outMessages.push(message);
        } else {
          console.log(message);
        }
      }

      const CARTRIDGE_ID = 'ZL';
      if (header.cartridgeId !== CARTRIDGE_ID) {
        pass = false;
        let message = 'bad cartridge ID: want ' + JSON.stringify(CARTRIDGE_ID)
          + ' but got ' + JSON.stringify(header.cartridgeId);
        if (outMessages) {
          outMessages.push(message);
        } else {
          console.log(message);
        }
      }

      const REVISION_NUMBER = 0;
      if (header.revisionNumber !== REVISION_NUMBER) {
        pass = false;
        let message = 'bad revision number: want ' + JSON.stringify(REVISION_NUMBER)
          + ' but got ' + JSON.stringify(header.revisionNumber);
        if (outMessages) {
          outMessages.push(message);
        } else {
          console.log(message);
        }
      }

      return pass;
    }
  }

  return new RomStorage();
})();

class RomHeader {
  constructor(arrayBuffer) {
    let data = new DataView(arrayBuffer);

    this.imageName = '';
    for (let offset = 0x20; offset < 0x34; ++offset) {
      let byte = data.getUint8(offset);
      if (byte) {
        this.imageName += String.fromCodePoint(byte);
      } else {
        break;
      }
    }

    this.cartridgeId = String.fromCodePoint(data.getUint8(0x3c))
      + String.fromCodePoint(data.getUint8(0x3d));
    this.revisionNumber = data.getUint8(0x3f);
  }
}

class RomView {
  constructor() {
    Status.hide();
    this.element = $t('div', {
      className: 'rom-view',
      children: [
        $t('h1', { textContent: 'Store ROM' }),
        $t('p', {
          textContent: 'Select a big-endian ROM image of The Legend of Zelda: Ocarina of Time, '
            + 'NTSC version 1.0. The typical file extension is .z64.'
        }),
        this.fileInput = $t('input', { type: 'file' }),
        this.storeButton = $t('button', { textContent: 'Store' }),
        this.errorDiv = $t('p', { className: 'error' }),
      ],
    });

    this.storeButton.addEventListener('click', () => this.handleStore());
  }

  handleStore() {
    this.storeButton.disabled = true;
    this.errorDiv.textContent = '';

    let fileList = this.fileInput.files;
    if (fileList.length !== 1) {
      this.storeButton.disabled = false;
      this.errorDiv.textContent = 'Select one file.';
      return;
    }
    let file = fileList[0];

    Status.show('Reading file...');
    this.asyncCompleteStore(new Promise((resolve, reject) => {
      let reader = new FileReader();
      reader.readAsArrayBuffer(file);
      reader.addEventListener('load', () => resolve(reader.result));
      reader.addEventListener('error', () => reject(reader.error));
      reader.addEventListener('abort', () => new Error('read aborted'));
    }));
  }

  async asyncCompleteStore(romPromise) {
    let rom;
    try {
      rom = await romPromise;
    } catch (e) {
      this.storeButton.disabled = false;
      this.errorDiv.textContent = e.name + ': ' + e.message;
      Status.hide();
      return;
    }

    await RomStorage.store(rom);
    Container.setView(new MainView({ rom }).canvas);
  }
}

const VERTEX_SHADER_SOURCE = `#version 300 es

precision highp float;
precision highp int;

layout(location = 0) in vec3 vertexPosition;
layout(location = 1) in vec3 vertexNormal;
layout(location = 2) in uint vertexFlags;
layout(location = 3) in vec2 vertexTexCoord;
layout(location = 4) in vec4 vertexColor;

uniform mat4 u_projectionMatrix;
uniform mat4 u_modelViewMatrix;

out vec4 v_color;
out vec4 v_shade;
out vec2 v_tex_coord;

void main() {
  gl_Position = u_projectionMatrix * u_modelViewMatrix * vec4(vertexPosition, 1.0);

  v_color = vertexColor;
  v_shade = vertexColor;
  v_tex_coord = vertexTexCoord / 32.0;
}
`;

function glInitShader(gl, type, source) {
  try {
    let shader = gl.createShader(type);
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

class MainView {
  constructor({ rom }) {
    this.canvas = $t('canvas');
    let gl = this.gl = this.canvas.getContext(
      'webgl2', {
      alpha: false,
      antialias: false,
      depth: true,
      stencil: false,
    });
    this.w = null;
    this.h = null;

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
          let old = this.touches.get(touch.identifier);
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
        this.changeScene(this.sceneIndex + 1);
      } else if (key === 'PageUp') {
        this.changeScene(this.sceneIndex - 1);
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

    this.view = {
      pos: vec3.clone([-4, 50, 603]),
      yaw: 0,
      pitch: 0,
    };

    this.ctx = new core.Context(this.gl, new Uint8Array(rom));
    this.batches = null;
    this.sceneIndex = null;
    this.currentResolves = [];
    this.nextResolves = [];

    this.changeScene(0);

    window.requestAnimationFrame(timestamp => this.step(timestamp));
  }

  nextStep() {
    return new Promise(resolve => this.nextResolves.push(resolve));
  }

  async changeScene(sceneIndex) {
    let gl = this.gl;
    this.sceneIndex = sceneIndex;

    Status.show('Processing scene...');
    await this.nextStep();

    let t1 = performance.now();
    let processedScene = this.ctx.processScene(sceneIndex);

    // Compile shaders for all batches.
    let vertexShader = glInitShader(gl, gl.VERTEX_SHADER, VERTEX_SHADER_SOURCE);
    let fragmentShaders = [];
    for (let batch of processedScene) {
      let shader = gl.createShader(gl.FRAGMENT_SHADER);
      gl.shaderSource(shader, batch.fragmentShader);
      gl.compileShader(shader);
      fragmentShaders.push(shader);
    }

    // Load vertex buffers and textures for all batches.
    let vertexBuffers = [];
    let textures = [];
    for (let batch of processedScene) {
      let buffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      gl.bufferData(gl.ARRAY_BUFFER, batch.vertexData, gl.STATIC_DRAW);
      vertexBuffers.push(buffer);

      let batch_textures = [];
      for (let texture of batch.textures) {
        batch_textures.push({
          texture: this.ctx.getTexture(texture.textureKey),
          sampler: this.ctx.getSampler(texture.samplerKey),
          width: texture.width,
          height: texture.height,
        });
      }
      textures.push(batch_textures);
    }

    // Link programs for all batches.
    let programs = [];
    for (let i = 0; i < processedScene.length; ++i) {
      let program = gl.createProgram();
      gl.attachShader(program, vertexShader);
      gl.attachShader(program, fragmentShaders[i]);
      gl.linkProgram(program);
      programs.push(program);
    }

    // Assemble all batches to be drawn.
    let opaqueBatches = [];
    let translucentBatches = [];
    for (let i = 0; i < processedScene.length; ++i) {
      if (!gl.getProgramParameter(programs[i], gl.LINK_STATUS)) {
        console.log('program info log:', gl.getProgramInfoLog(programs[i]));
        console.log('vertex shader info log:', gl.getShaderInfoLog(vertexShader));
        console.log('fragment shader info log:', gl.getShaderInfoLog(fragmentShaders[i]));
        throw new Error('failed to link GL program');
      }

      let batch = processedScene[i];
      let collection = batch.translucent ? translucentBatches : opaqueBatches;
      collection.push({
        program: programs[i],
        vertexBuffer: vertexBuffers[i],
        translucent: batch.translucent,
        mode: gl.TRIANGLES,
        count: batch.vertexData.byteLength / 20,
        textures: textures[i],
      });
    }
    this.batches = [].concat(opaqueBatches, translucentBatches);

    let t2 = performance.now();

    Status.show('Rendering first frame... (this is slow on Chrome on Windows)');
    await this.nextStep();

    let t3 = performance.now();

    Status.show('Ready. ('
      + 'processing: ' + Math.round(t2 - t1) + ' ms, '
      + 'first frame: ' + Math.round(t3 - t2) + ' ms)');
  }

  updateDimensions() {
    let r = Container.getBoundingClientRect();
    let width = (r.width * window.devicePixelRatio) | 0;
    let height = (r.height * window.devicePixelRatio) | 0;

    if (width != this.w || height != this.h) {
      this.canvas.width = this.w = width;
      this.canvas.height = this.h = height;
      this.canvas.style.width = r.width + 'px';
      this.canvas.style.height = r.height + 'px';
    }
  }

  step(timestamp) {
    // Trigger anything that was waiting for a new frame.
    for (let resolve of this.currentResolves) {
      resolve();
    }
    this.currentResolves = this.nextResolves;
    this.nextResolves = [];

    this.updateDimensions();

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
    gl.viewport(0, 0, this.w, this.h);

    gl.depthFunc(gl.LEQUAL);
    gl.frontFace(gl.CCW);

    let projectionMatrix = mat4.create();
    mat4.perspective(
      projectionMatrix, 0.5 * Math.PI, this.w / this.h, 1, 10000.0);

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
      gl.depthMask(true);
      gl.disable(gl.BLEND);

      if (batch.translucent) {
        gl.disable(gl.CULL_FACE);
        // TODO: This is extremely fake. The RDP has blending parameters.
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
      } else {
        gl.enable(gl.CULL_FACE);
        gl.disable(gl.BLEND);
      }

      gl.uniformMatrix4fv(
        gl.getUniformLocation(batch.program, 'u_projectionMatrix'),
        false,
        projectionMatrix);
      gl.uniformMatrix4fv(
        gl.getUniformLocation(batch.program, 'u_modelViewMatrix'),
        false,
        modelViewMatrix);
      gl.uniform1i(gl.getUniformLocation(batch.program, "u_texture_a"), 0);
      gl.uniform1i(gl.getUniformLocation(batch.program, "u_texture_b"), 1);

      gl.bindBuffer(gl.ARRAY_BUFFER, batch.vertexBuffer);
      // Position
      gl.vertexAttribPointer(0, 3, gl.SHORT, false, 20, 0);
      gl.enableVertexAttribArray(0);
      // Normal
      gl.vertexAttribPointer(1, 3, gl.BYTE, true, 20, 8);
      gl.enableVertexAttribArray(1);
      // Flags
      gl.vertexAttribIPointer(2, 1, gl.UNSIGNED_BYTE, 20, 11);
      gl.enableVertexAttribArray(2);
      // Texture coordinates
      gl.vertexAttribPointer(3, 2, gl.SHORT, false, 20, 12);
      gl.enableVertexAttribArray(3);
      // Color
      gl.vertexAttribPointer(4, 4, gl.UNSIGNED_BYTE, true, 20, 16);
      gl.enableVertexAttribArray(4);

      for (let i = 0; i < 2; ++i) {
        gl.activeTexture(gl.TEXTURE0 + i);
        let texture = batch.textures[0];
        if (texture && texture.texture) {
          gl.bindTexture(gl.TEXTURE_2D, texture.texture);
          gl.uniform2f(
            gl.getUniformLocation(
              batch.program,
              i == 0 ? "u_texture_a_inv_size" : "u_texture_b_inv_size"),
            1 / texture.width,
            1 / texture.height);
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
