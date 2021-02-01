const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    mode: 'development',
    devtool: 'inline-source-map',
    entry: './ts/index.ts',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'bundle.js',
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                loader: 'ts-loader',
            },
        ],
    },
    resolve: {
        extensions: ['.tsx', '.ts', '.js'],
    },
    devServer: {
        contentBase: path.join(__dirname, 'static'),
    },
    experiments: {
        syncWebAssembly: true,
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname),
            forceMode: 'production',
            watchDirectories: [
                path.resolve(__dirname, '../oot-explorer-expr'),
                path.resolve(__dirname, '../oot-explorer-game-data'),
                path.resolve(__dirname, '../oot-explorer-gl'),
                path.resolve(__dirname, '../oot-explorer-read'),
                path.resolve(__dirname, '../oot-explorer-reflect'),
                path.resolve(__dirname, '../oot-explorer-rom'),
                path.resolve(__dirname, '../oot-explorer-segment'),
                path.resolve(__dirname, '../oot-explorer-vrom'),
            ],
        }),
    ],
};
