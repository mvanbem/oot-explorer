export type WasmModule = typeof import('../pkg');

export const wasmPromise: Promise<WasmModule> = (async () => {
    return await import('../pkg');
})();

// Types using serde-wasm-bindgen do not generate TypeScript type definitions.

export namespace WasmInterface {
    export interface ProcessSceneResult {
        batches: ProcessSceneBatch[],
        backgrounds: string[],
        startPos?: [number, number, number, number, number],
    }

    export interface ProcessSceneBatch {
        fragmentShader: string;
        vertexData: ArrayBuffer;
        translucent: boolean,
        textures: ProcessSceneTexture[],
        zUpd: boolean,
        decal: boolean,
    }

    export interface ProcessSceneTexture {
        textureKey: number;
        samplerKey: number;
        width: number;
        height: number;
    }
}
