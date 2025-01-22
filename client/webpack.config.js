const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const CopyWebpackPlugin = require('copy-webpack-plugin');

const distPath = path.resolve(__dirname, "dist");
module.exports = (env, argv) => {
    return {
        devServer: {
            compress: argv.mode === 'production',
            port: 8000,
            historyApiFallback: true,
            devMiddleware: {
                writeToDisk: true,
            },
            proxy: {
                '/': {
                    target: "http://127.0.0.1:8080",
                    changeOrigin: true,
                    bypass: function (req, res, proxyOptions) {
                        if (
                            ['/api/', '/blob/', '/oauth2/login/', '/oidc/login/'].every((prefix) => {
                                return !req.url.startsWith(prefix);
                            })
                        ) {
                            return req.url;
                        }
                    },
                }
            }
        },
        entry: {
            index: './index.js',
        },
        output: {
            path: distPath,
            filename: "[name].js",
            webassemblyModuleFilename: "app.wasm",
            publicPath: ""
        },
        resolve: {
            extensions: [".js", ".ts"]
        },
        externals: {
            "crypto-js": 'CryptoJS',
            "jsencrypt": 'JSEncrypt',
        },
        experiments: {
            asyncWebAssembly: true
        },
        module: {
            rules: [
                {
                    test: /\.ts$/,
                    use: ["babel-loader", {
                        loader: "ts-loader",
                        options: {
                            transpileOnly: true
                        }
                    }]
                },
                {
                    test: /\.js$/,
                    use: {
                        loader: 'babel-loader',
                        options: {
                            presets: ['@babel/preset-env']
                        }
                    }
                },
                {
                    test: /\.m?js/,
                    resolve: {
                        fullySpecified: false,
                    },
                },
                {
                    test: /\.css$/,
                    use: [MiniCssExtractPlugin.loader, "css-loader"]
                },
                {
                    test: /\.less$/,
                    use: [MiniCssExtractPlugin.loader, "css-loader", "less-loader"]
                }
            ]
        },
        plugins: [
            new CopyWebpackPlugin({
                patterns: [
                    { from: './static', to: distPath }
                ],
            }),
            new MiniCssExtractPlugin({
                filename: "[name].css"
            }),
            new WasmPackPlugin({
                crateDirectory: ".",
                extraArgs: "--no-typescript",
            })
        ],
        watch: argv.mode !== 'production'
    };
};
