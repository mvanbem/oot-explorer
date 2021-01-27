const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    mode: 'development',
    entry: './ts/index.ts',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'bundle.js',
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
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
        }),
    ],
};
