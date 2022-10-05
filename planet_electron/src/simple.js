const api = require('../../planet_node/index')
const style = require('./styles/default.json')

window.maplibregl.setPlanetVectorTilePlugin(api)
window.map = new window.maplibregl.Map({
    container: 'map',
    style: style
})
