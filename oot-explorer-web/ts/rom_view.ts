import { Container } from './container';
import { $t, $text } from './dollar_t';
import { MainView } from './main_view';
import { RomStorage } from './rom_storage';
import { Status } from './status';
import { Toolbar } from './toolbar';
import { wasmPromise } from './wasm';

export class RomView {
    element: HTMLElement;
    fileName: HTMLElement;
    errorDiv: HTMLElement;
    fileInput: HTMLInputElement;
    storeButton: HTMLButtonElement;

    constructor() {
        Toolbar.hide();
        Status.hide();
        this.element = $t('div', {
            className: 'rom-view',
            children: [
                $t('div', { className: 'title', textContent: 'Store ROM' }),
                $t('p', {
                    textContent: 'Select a big-endian ROM image of The Legend of Zelda: Ocarina of Time, '
                        + 'NTSC version 1.0.',
                }),
                $t('p', {
                    children: [
                        $text('The typical file extension is '),
                        $t('tt', { textContent: '.z64' }),
                        $text('.'),
                    ],
                }),
                this.fileName = $t('p', { className: 'file-name', textContent: '(no selection)' }),
                this.errorDiv = $t('p', { className: 'error hidden' }),
                $t('div', {
                    className: 'button-row',
                    children: [
                        $t('label', {
                            className: 'file-input',
                            textContent: 'Choose File',
                            children: [
                                this.fileInput = $t('input', { type: 'file' }),
                            ],
                        }),
                        this.storeButton = $t('button', { textContent: 'Store' }),
                    ],
                }),
            ],
        });

        this.fileInput.addEventListener('input', () => {
            let fileList = this.fileInput.files!;
            switch (fileList.length) {
                case 0:
                    this.fileName.textContent = '(no selection)';
                    break;

                case 1:
                    this.fileName.textContent = fileList[0].name;
                    break;

                default:
                    this.fileName.textContent = '(multiple files)';
                    break;
            }
            this.hideError();
        });
        this.storeButton.addEventListener('click', () => this.handleStore());
    }

    hideError() {
        this.errorDiv.classList.add('hidden');
    }

    showError(text: string) {
        this.errorDiv.textContent = text;
        this.errorDiv.classList.remove('hidden');
    }

    handleStore() {
        this.storeButton.disabled = true;
        this.hideError();

        let fileList = this.fileInput.files!;
        if (fileList.length !== 1) {
            this.storeButton.disabled = false;
            this.showError('Select one file.');
            return;
        }
        let file = fileList[0];

        Status.show('Reading file...');
        this.asyncCompleteStore(new Promise((resolve, reject) => {
            let reader = new FileReader();
            reader.readAsArrayBuffer(file);
            reader.addEventListener('load', () => resolve(<ArrayBuffer>reader.result));
            reader.addEventListener('error', () => reject(reader.error));
            reader.addEventListener('abort', () => new Error('read aborted'));
        }));
    }

    async asyncCompleteStore(romPromise: Promise<ArrayBuffer>) {
        try {
            let rom = await romPromise;
            await RomStorage.store(rom);
            Container.setView(new MainView({ wasm: await wasmPromise, rom }).canvas);
        } catch (e) {
            this.storeButton.disabled = false;
            this.showError(e.message);
            Status.hide();
            return;
        }
    }
}
