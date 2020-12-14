import * as core from './oot_explorer_web.js';

let worker = null;
self.addEventListener('message', e => {
  switch (e.data.kind) {
    case 'init':
      worker = new Worker(new Uint8Array(e.data.rom));
      break;
    case 'processScene':
      worker.processScene(e.data.scene);
      break;
  }
});

class Worker {
  constructor(rom) {
    this.rom = rom;
    core.default().then(() => {
      self.postMessage({ kind: 'ready' });
    }).catch(e => {
      self.postMessage({
        kind: 'status',
        message: 'wasm core init failed: ' + e,
      });
    });
  }
  processScene(scene) {
    // TODO: Only copy the ROM data in once?
    let batches = core.processScene(this.rom, scene);
    let transfer = [];
    for (let batch of batches) {
      transfer.push(batch.vertexData);
    }
    self.postMessage({
      kind: 'scene',
      batches,
    }, transfer);
  }
}
