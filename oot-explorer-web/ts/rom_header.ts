export class RomHeader {
    imageName: string;
    cartridgeId: string;
    revisionNumber: number;

    constructor(arrayBuffer: ArrayBuffer) {
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
