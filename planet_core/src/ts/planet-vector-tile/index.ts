import flatbuffers from 'flatbuffers'
import { VectorTile, VectorTileLayer, VectorTileFeature } from '@mapbox/vector-tile'
import { Tile } from './tile'
import { Layer } from './layer'
import { Feature } from './feature'
import { ValueType } from './value-type'

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
        const fbFeat = this._layer.features(featureIndex)!
        feat = this._features[featureIndex] = new PlanetVectorTileFeature(this._tile, fbFeat)
        return feat
    }

}

export class PlanetVectorTileFeature implements VectorTileFeature {
    _tile: Tile
    _feat: Feature
    _properties: { [_: string]: string | number | boolean } | undefined

    extent: number

    constructor(fbTile: Tile, fbFeat: Feature) {
        // https://github.com/maplibre/maplibre-gl-js/blob/028344137fe1676b50b8da2729f1dcb5c8b65eac/src/data/extent.ts
        this.extent = 8196
        this._tile = fbTile
        this._feat = fbFeat
    }

    // We don't encode the type in the flatbuffer, because it can be derived.
    // I don't think this is even being used in Maplibre.
    get type(): 1 | 2 | 3 {
        const firstGeom = this._feat.geometry(0)!
        const len = firstGeom.pointsLength() 
        if (len < 2) {
            return 1
        }
        const pt0 = firstGeom.points(0)!
        const x0 = pt0.x
        const y0 = pt0.y
        const ptEnd = firstGeom.points(len - 1)!
        const xEnd = ptEnd.x
        const yEnd = ptEnd.y
        if (x0 === xEnd && y0 === yEnd) {
            return 3
        }
        return 2
    }

    // This really should be optional, contrary to the TypeScript definition...
    get id(): number {
        // TODO deal with JS number overflow
        return Number(this._feat.id()) || Number(this._feat.h())
    }

    get properties(): { [_: string]: string | number | boolean } {
        if (this._properties) {
            return this._properties
        }
        const props = this._properties = {}

        for (let i = 0, len = this._feat.keysLength(); i < len; i++) {
            const keyIdx = this._feat.keys(i)!
            const valIdx = this._feat.values(i)!
            const key = this._tile.strings(keyIdx)!
            let val = getVal(this._tile, valIdx)
            props[key] = val
        }

        return props
    }

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

function getVal(tile: Tile, idx: number): string | number | boolean {
    const val = tile.values(idx)!
    const t = val.t()
    if (t === ValueType.String) {
        return tile.strings(val.v())
    }
    if (t === ValueType.Number) {
        return val.v()
    }
    return !!val.v()
}
