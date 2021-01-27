import { Container } from './container';
import { MainView } from './main_view';
import { RomStorage } from './rom_storage';
import { RomView } from './rom_view';
import { Status } from './status';
import { wasmPromise } from './wasm';

window.addEventListener('error', e => {
  Status.show('top-level error: ' + e.message);
});

window.addEventListener('DOMContentLoaded', async () => {
  // TODO: Show a pop-up menu that makes it clear what this does.
  document.getElementById('menu')!.addEventListener('click', async () => {
    await RomStorage.clear();
    window.location.reload();
  });

  let rom = await RomStorage.load();
  if (rom === null) {
    Container.setView(new RomView().element);
  } else {
    Container.setView(new MainView({ wasm: await wasmPromise, rom }).canvas);
  }
});
