const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = {
  entry: ['./static/style.scss', './static/script.js'],
  output: {
    path: path.resolve(__dirname, 'static-dist'),
    filename: 'script.js',
  },

  module: {
    rules: [
      {
        test: /\.scss$/,
        use: [
          {
            loader: 'file-loader',
            options: { name: 'style.css', },
          },
          { loader: 'extract-loader', },
          { loader: 'css-loader', },
          { loader: 'postcss-loader',
            options: {
              plugins: () => [require('autoprefixer')()],
            },
          },
          {
            loader: 'sass-loader',
            options: {
              includePaths: ['./node_modules'],
            },
          },
        ],
      },
      {
        test: /\.js$/,
        loader: 'babel-loader',
      },
      {
        test: /\.html$/,
        loader: 'html-loader',
      },
    ],
  },

  plugins: [
    new CopyWebpackPlugin([
      { from: path.resolve(__dirname, './static/index.html'), to: path.resolve(__dirname, './static-dist/index.html') },
    ]),
  ],

  watch: true,
  devtool: 'source-map',
  mode: 'development',
};