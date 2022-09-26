const path = require('path')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

module.exports = {
  // The Webpack config to use when compiling your react app for development or production.
  webpack: function (config, env) {
    const wasmExtensionRegExp = /\.wasm$/
    config.resolve.extensions.push('.wasm')
    config.experiments = {
      asyncWebAssembly: true,
    }

    config.module.rules.forEach(rule => {
      ;(rule.oneOf || []).forEach(oneOf => {
        if (oneOf.type === 'asset/resource') {
          oneOf.exclude.push(wasmExtensionRegExp)
        }
      })
    })
    // set resolve.fallback
    config.resolve.fallback = {
      fs: false,
      path: false,
      crypto: false,
    };
    config.plugins.push(new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, 'hashi-solver-wasm'),
      withTypeScript: true,
    }));

    return config;
  },
};
