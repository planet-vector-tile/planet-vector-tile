import { ByteBuffer } from 'flatbuffers'
import { Feature as GeoJSONFeature, Position } from 'geojson'
import { VectorTileLayer, VectorTileFeature } from '@mapbox/vector-tile'
import Point from '@mapbox/point-geometry'
import { PVTTile } from './fbs/pvttile'
import { PVTLayer } from './fbs/pvtlayer'
import { PVTFeature } from './fbs/pvtfeature'
import { PVTValueType } from './fbs/pvtvalue-type'

// https://github.com/maplibre/maplibre-gl-js/blob/028344137fe1676b50b8da2729f1dcb5c8b65eac/src/data/extent.ts
type Extent = 8192
const EXTENT: Extent = 8192

export class PVT {
  layers: { [_: string]: VectorTileLayer }

  constructor(arr: Uint8Array) {
    const buffer = new ByteBuffer(arr)
    const pvtTile = PVTTile.getRootAsPVTTile(buffer)
    const layers = {}

    for (let i = 0, len = pvtTile.layersLength(); i < len; i++) {
      const pvtLayer = pvtTile.layers(i)!
      const nameIdx = pvtLayer.name()
      const name = pvtTile.strings(nameIdx)

      layers[name] = new Layer(pvtLayer, name, pvtTile)
    }
    this.layers = layers
  }
}

export class Layer implements VectorTileLayer {
  features: VectorTileFeature[]
  name: string
  extent: Extent
  length: number

  constructor(pvtLayer: PVTLayer, name: string, pvtTile: PVTTile) {
    const length = pvtLayer.featuresLength()
    const features = new Array(length)

    this.features = features
    this.name = name
    this.extent = EXTENT
    this.length = length

    for (let i = 0; i < length; i++) {
      const pvtFeature = pvtLayer.features(i)
      features[i] = new Feature(pvtFeature, pvtTile)
    }
  }

  feature(featureIndex: number): VectorTileFeature {
    return this.features[featureIndex]
  }
}

export class Feature implements VectorTileFeature {
  // ID is treated as optional, contrary to MapLibre's TypeScript definition.
  // The TypeScript definition is not correct, as Mapbox does handle IDs as optional.
  // It is a 64 bit integer in the flatbuffer, so we cast it down to Number (64 bit float),
  // as this is the type within Mapbox.
  id: number // can be null
  extent: number
  type: 1 | 2 | 3
  geometries: Point[][]
  properties: { [_: string]: string | number | boolean }

  constructor(pvtFeature: PVTFeature, pvtTile: PVTTile) {
    this.id = Number(pvtFeature.id()) || null
    this.extent = EXTENT

    // ==> Determine type
    const firstGeom = pvtFeature.geometries(0)!
    const pointsLen = firstGeom.pointsLength()
    // point
    if (pointsLen < 2) {
      this.type = 1
    }
    const { x, y } = firstGeom.points(0)!
    const ptEnd = firstGeom.points(pointsLen - 1)!
    // polygon - closed ring
    if (x === ptEnd.x && y === ptEnd.y) {
      this.type = 3
    }
    // line
    this.type = 2

    // ==> Build properties.
    // Doing this upfront rather than lazily, as it is needed immediately.
    const props = {}
    for (let i = 0, len = pvtFeature.keysLength(); i < len; i++) {
      const keyIdx = pvtFeature.keys(i)!
      const valIdx = pvtFeature.values(i)!
      const key = pvtTile.strings(keyIdx)!
      let val = getVal(pvtTile, valIdx)
      props[key] = val
    }
    this.properties = props

    // ==> Build geometries.
    // Also doing this upfront. No added value of being lazy.
    const geometriesLen = pvtFeature.geometriesLength()
    const outer = new Array<Point[]>(geometriesLen)
    for (let i = 0; i < geometriesLen; i++) {
      const geom = pvtFeature.geometries(i)!
      const innerLen = geom.pointsLength()
      const inner = new Array<Point>(innerLen)
      outer[i] = inner
      for (let j = 0; j < innerLen; j++) {
        const pt = geom.points(j)!
        // These are instances of Mapbox's Points where there are attached transform methods.
        inner[j] = new Point(pt.x(), pt.y())
      }
    }
    this.geometries = outer
  }

  loadGeometry(): Point[][] {
    return this.geometries
  }

  // NHTODO There is a bug with this method, as shown in unit tests.
  toGeoJSON(x: number, y: number, z: number): GeoJSONFeature {
    const granularity = EXTENT * Math.pow(2, z)
    const tileWest = EXTENT * x
    const tileNorth = EXTENT * y

    // Spherical Mercator tile coordinates to WGS84
    function project(line: Point[]): Position[] {
      const len = line.length
      const coords = new Array<number[]>(len)
      // we want to loop a full line in the function, since JS can't inline functions
      for (let j = 0; j < len; j++) {
        const p = line[j]
        const mercY = 180 - ((tileNorth + p.y) * 360) / granularity
        const lon = ((tileWest + p.x) * 360) / granularity - 180
        const lat = (360 / Math.PI) * Math.atan(Math.exp((mercY * Math.PI) / 180)) - 90
        coords[j] = [lon, lat]
      }
      return coords
    }

    const geometries = this.geometries
    const len = geometries.length
    const type = this.type

    // MultiPoint has one less nesting
    if (type === 1) {
      const outerCoordinates = new Array<Position>(len)
      for (let i = 0; i < len; i++) {
        const point = project(geometries[i])[0]
        outerCoordinates[i] = point
      }
      return {
        type: 'Feature',
        geometry: {
          type: 'MultiPoint',
          coordinates: outerCoordinates,
        },
        properties: this.properties,
      }
    }

    const outerCoordinates = new Array<Position[]>(len)
    for (let i = 0; i < len; i++) {
      const innerCoordinates = project(geometries[i])
      outerCoordinates[i] = innerCoordinates
    }

    if (type === 2) {
      return {
        type: 'Feature',
        geometry: {
          type: 'MultiLineString',
          coordinates: outerCoordinates,
        },
        properties: this.properties,
      }
    }

    // Now we have to figure out winding order to determine which rings are holes.
    const polygons: Position[][][] = []
    let lastPolygon: Position[][] = []
    for (const innerCoordinates of outerCoordinates) {
      const area = signedArea(innerCoordinates)
      if (area === 0) {
        continue
      }
      // outer ring
      if (area > 0) {
        lastPolygon = [innerCoordinates]
        polygons.push(lastPolygon)
      }
      // inner ring
      else {
        lastPolygon.push(innerCoordinates)
      }
    }

    return {
      type: 'Feature',
      geometry: {
        type: 'MultiPolygon',
        coordinates: polygons,
      },
      properties: this.properties,
    }
  }
}

function getVal(tile: PVTTile, idx: number): string | number | boolean {
  const val = tile.values(idx)!
  const t = val.t()
  if (t === PVTValueType.String) {
    return tile.strings(val.v())
  }
  if (t === PVTValueType.Number) {
    return val.v()
  }
  return !!val.v()
}

function signedArea(ring): number {
  let sum = 0
  for (let i = 0, len = ring.length, j = len - 1; i < len; j = i++) {
    const p1 = ring[i]
    const p2 = ring[j]
    sum += (p2[0] - p1[0]) * (p1[1] + p2[1])
  }
  return sum
}
