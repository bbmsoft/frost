const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const WorkboxPlugin = require('workbox-webpack-plugin');

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "development",
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "static")
    ]),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
    new WorkboxPlugin.GenerateSW({
      maximumFileSizeToCacheInBytes: 5 * 1024 * 1024,
      // these options encourage the ServiceWorkers to get in there fast
      // and not allow any straggling "old" SWs to hang around
      clientsClaim: true,
      skipWaiting: true,
    }),
  ]
};

