import { RomHeader } from './rom_header';
import { Status } from './status';

export const RomStorage = (() => {
    const DATABASE_NAME = 'rom';
    const OBJECT_STORE_NAME = 'rom';
    const KEY = 'rom';

    class RomStorage {
        dbPromise?: Promise<IDBDatabase>;

        getDatabase() {
            if (!this.dbPromise) {
                this.dbPromise = new Promise((resolve, reject) => {
                    let req = window.indexedDB.open(DATABASE_NAME, 1);
                    req.addEventListener('success', () => resolve(req.result));
                    req.addEventListener('error', () => reject(req.error));
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
            const rom = await new Promise<ArrayBuffer>((resolve, reject) => {
                let txn = db.transaction([OBJECT_STORE_NAME], 'readwrite');
                txn.addEventListener('complete', () => resolve(req.result || null));
                txn.addEventListener('error', () => reject(txn.error));
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

        async store(rom: ArrayBuffer) {
            let messages: string[] = [];
            if (!this.isValid(rom, messages)) {
                throw new Error('ROM failed validation: ' + messages.join('; '));
            }

            Status.show('Storing ROM to IndexedDB...');
            const db = await this.getDatabase();
            return await new Promise<void>((resolve, reject) => {
                let txn = db.transaction([OBJECT_STORE_NAME], 'readwrite');
                txn.addEventListener('complete', () => resolve());
                txn.addEventListener('error', () => reject(txn.error));
                txn.addEventListener('abort', () => reject(new Error('transaction aborted')));
                txn.objectStore(OBJECT_STORE_NAME).put(rom, KEY);
            });
        }

        async clear() {
            Status.show('Clearing IndexedDB...');
            const db = await this.getDatabase();
            return await new Promise<void>((resolve, reject) => {
                let txn = db.transaction([OBJECT_STORE_NAME], 'readwrite');
                txn.addEventListener('complete', () => resolve());
                txn.addEventListener('error', () => reject(txn.error));
                txn.addEventListener('abort', () => reject(new Error('transaction aborted')));
                txn.objectStore(OBJECT_STORE_NAME).delete(KEY);
            });
        }

        isValid(rom: ArrayBuffer, outMessages?: string[]) {
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
