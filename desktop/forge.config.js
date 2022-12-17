const fs = require('fs')
const glob = require('glob')

module.exports = {
  packagerConfig: {},
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {},
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
    {
      name: '@electron-forge/maker-deb',
      config: {},
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {},
    },
  ],
  hooks: {
    // We have to do a custom copy of dependencies for the build.
    // It's hard to get electron forge and vite to play niceliy
    // with our project structure.
    generateAssets: async (forgeConfig, platform, arch) => {
      try {
        fs.rmSync('deps', { recursive: true })
      } catch (e) {
        console.log('deps does not exist')
      }
      fs.mkdirSync('deps')
      fs.copyFileSync('../index.js', 'deps/index.js')

      const nodeFiles = glob.sync('../*.node')
      for (let f of nodeFiles) {
        fs.copyFileSync(f, 'deps/' + f.split('/').pop())
      }

      const maplibreFiles = glob.sync('../maplibre-gl-js/dist/maplibre*')
      for (let f of maplibreFiles) {
        fs.copyFileSync(f, 'deps/' + f.split('/').pop())
      }

      // replace the script files in index.html with the paths to the deps folder
      let indexHtml = fs.readFileSync('dist/index.html', 'utf8')
      indexHtml = indexHtml.replace('root/maplibre-gl-js/dist/maplibre-gl-dev.js', '../deps/maplibre-gl.js')
      indexHtml = indexHtml.replace('src/map.js', '../src/map.js')
      fs.writeFileSync('dist/prod.html', indexHtml)
    },
  },
}
