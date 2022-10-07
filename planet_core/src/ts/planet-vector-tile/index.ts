import flatbuffers from 'flatbuffers'
import { VectorTile, VectorTileLayer, VectorTileFeature } from '@mapbox/vector-tile'
import { Feature, Geometry, GeoJsonProperties } from 'geojson'
import { Tile } from './tile'
import { Layer } from './layer'

export class PlanetVectorTile implements VectorTile {
    _tile: Tile
    _layers:  { [_: string]: VectorTileLayer } | undefined

    constructor(arrayBuffer: ArrayBuffer) {
        const arr = new Uint8Array(arrayBuffer)
        const buffer = new flatbuffers.ByteBuffer(arr)
        this._tile = Tile.getRootAsTile(buffer)
    }
    
    get layers(): { [_: string]: VectorTileLayer } {
        if (this._layers) {
            return this._layers
        } else {
            this._layers = {}
        }

        const fbTile = this._tile
        for (let i = 0, len = fbTile.layersLength(); i < len; i++) {
            const fbLayer = fbTile.layers(i)!
            const nameIdx = fbLayer.name()
            const name = fbTile.strings(nameIdx)
            const layer = new PlanetVectorTileLayer(fbTile, fbLayer, name)
            this._layers[name] = layer
        }
        return this._layers
    }
}

export class PlanetVectorTileLayer implements VectorTileLayer {
    _tile: Tile
    _layer: Layer
    _features: VectorTileFeature[]

    version?: number | undefined
    name: string
    extent: number
    length: number

    constructor(fbTile: Tile, fbLayer: Layer, name: string) {
        this._tile = fbTile
        this._layer = fbLayer
        this._features = new Array(fbLayer.featuresLength())
        this.name = name
    }

    feature(featureIndex: number): VectorTileFeature {
        let feat = this._features[featureIndex]
        if (feat) {
            return feat
        }
        const fbFeat = this._layer.features(featureIndex)
        feat = this._features[featureIndex] = new PlanetVectorTileFeature(this._tile, fbFeat)
    }

}

export class PlanetVectorTileFeature implements VectorTileFeature {

    extent: number
    type: 1 | 2 | 3
    id: number

    constructor(fbTile: Tile, fbFeat: Feature) {
        // https://github.com/maplibre/maplibre-gl-js/blob/028344137fe1676b50b8da2729f1dcb5c8b65eac/src/data/extent.ts
        this.extent = 8196
        
    }

    properties: { [_: string]: string | number | boolean }
    loadGeometry(): import("@mapbox/point-geometry")[][] {
        throw new Error('Method not implemented.')
    }
    toGeoJSON(x: number, y: number, z: number): Feature<Geometry, GeoJsonProperties> {
        throw new Error('Method not implemented.')
    }
    bbox?(): [number, number, number, number] {
        throw new Error('Method not implemented.')
    }

}
