const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const webpack = require("webpack");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

/** @type {webpack.Configuration} */
module.exports = {
  entry: "./site/index.tsx",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
    clean: true,
  },
  module: {
    rules: [
      
            // {
            //   test: /\.tsx?$/,
            //   use: "ts-loader",
            //   exclude: /node_modules/,
            // },
          
      {
        test: /\.tsx$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader",
          options: {
            targets: "defaults",
            presets: ["@babel/preset-env", "@babel/preset-react", "@babel/preset-typescript"],
          },
        },
      },
      {
        test: /\.css$/i,
        exclude: /node_modules/,
        use: ["style-loader", "css-loader", "postcss-loader"],
      },
    ],
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: "site/index.html",
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ],
  mode: "development",
  experiments: {
    asyncWebAssembly: true,
  },
};
