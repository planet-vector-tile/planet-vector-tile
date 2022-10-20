import { ByteBuffer } from 'flatbuffers';
import { Feature, Position } from 'geojson';
import type { VectorTile, VectorTileLayer, VectorTileFeature } from '@mapbox/vector-tile';
import Point from '@mapbox/point-geometry';
import { PVTTile } from './fbs/pvttile';
import { PVTLayer } from './fbs/pvtlayer';
import { PVTFeature } from './fbs/pvtfeature';
import { PVTValueType } from './fbs/pvtvalue-type';
import { PVTGeometry } from './fbs/pvtgeometry';

export class PlanetVectorTile implements VectorTile {
    _tile: PVTTile;
    _layers: { [_: string]: VectorTileLayer } | undefined;

    constructor(arr: Uint8Array) {
        const buffer = new ByteBuffer(arr);
        this._tile = PVTTile.getRootAsPVTTile(buffer);
    }

    get layers(): { [_: string]: VectorTileLayer } {
        if (this._layers) {
            return this._layers;
        } else {
            this._layers = {};
        }

        const fbTile = this._tile;
        for (let i = 0, len = fbTile.layersLength(); i < len; i++) {
            const fbLayer = fbTile.layers(i)!;
            const nameIdx = fbLayer.name();
            const name = fbTile.strings(nameIdx);
            const layer = new PlanetVectorTileLayer(fbTile, fbLayer, name);
            this._layers[name] = layer;
        }
        return this._layers;
    }
}

// https://github.com/maplibre/maplibre-gl-js/blob/028344137fe1676b50b8da2729f1dcb5c8b65eac/src/data/extent.ts
type Extent = 8196;
const EXTENT: Extent = 8196;

export class PlanetVectorTileLayer implements VectorTileLayer {
    _tile: PVTTile;
    _layer: PVTLayer;
    _features: VectorTileFeature[];

    version?: number | undefined;
    name: string;
    extent: Extent;
    length: number;

    constructor(tile: PVTTile, layer: PVTLayer, name: string) {
        this._tile = tile;
        this._layer = layer;
        this.length = layer.featuresLength();
        this._features = new Array(this.length);
        this.name = name;
        this.extent = EXTENT;
    }

    feature(featureIndex: number): VectorTileFeature {
        let feat = this._features[featureIndex];
        if (feat) {
            return feat;
        }
        const fbFeat = this._layer.features(featureIndex)!;
        feat = this._features[featureIndex] = new PlanetVectorTileFeature(this._tile, fbFeat);
        return feat;
    }
}

export class PlanetVectorTileFeature implements VectorTileFeature {
    extent: Extent;

    _tile: PVTTile;
    _feat: PVTFeature;
    _geom: Point[][];
    _properties: { [_: string]: string | number | boolean } | undefined;

    constructor(tile: PVTTile, feat: PVTFeature) {
        this.extent = EXTENT;
        this._tile = tile;
        this._feat = feat;
    }

    // We don't encode the type in the flatbuffer, because it can be derived.
    // 1 is MultiPoint
    // 2 is MultiLineString
    // 3 is MultiPolygon
    get type(): 1 | 2 | 3 {
        const firstGeom = this._feat.geometry(0)!;
        const len = firstGeom.pointsLength();
        // point
        if (len < 2) {
            return 1;
        }
        const { x, y } = firstGeom.points(0)!;
        const ptEnd = firstGeom.points(len - 1)!;
        // polygon - closed ring
        if (x === ptEnd.x && y === ptEnd.y) {
            return 3;
        }
        // line
        return 2;
    }

    // This really should be optional, contrary to the TypeScript definition...
    get id(): number {
        // TODO deal with JS number overflow
        return Number(this._feat.id()) || Number(this._feat.h());
    }

    get properties(): { [_: string]: string | number | boolean } {
        if (this._properties) {
            return this._properties;
        }
        const props = (this._properties = {});

        for (let i = 0, len = this._feat.keysLength(); i < len; i++) {
            const keyIdx = this._feat.keys(i)!;
            const valIdx = this._feat.values(i)!;
            const key = this._tile.strings(keyIdx)!;
            let val = getVal(this._tile, valIdx);
            props[key] = val;
        }

        return props;
    }

    // Point - single item MultiPoint
    // [ [[x,y]] ]

    // MultiPoint
    // [ [[x,y]] , [[x,y]] , [[x,y]] ]

    // Line - single itme Line
    // [ [[x,y], [x,y], [x,y]] ]

    // MultiLine
    // [ [[x,y], [x,y], [x,y]] , [[x,y], [x,y], [x,y]] ]

    // MultiPolygon
    // Same thing, but lines are closed.
    // Holes are deterined by winding order. (Shoelace Formula)
    loadGeometry(): Point[][] {
        if (this._geom) {
            return this._geom;
        }
        // TODO Implement point where we just use the FB rather than do this extra loop / copy.
        const feat = this._feat;
        const len = feat.geometryLength();
        const outer = new Array<Point[]>(len);
        for (let i = 0; i < len; i++) {
            const geom = feat.geometry(i)!;
            const innerLen = geom.pointsLength();
            const inner = new Array<Point>(innerLen);
            outer[i] = inner;
            for (let j = 0; j < innerLen; j++) {
                const pt = geom.points(j)!;
                // These are instances of Mapbox's Points where there are attached transform methods.
                inner[j] = new Point(pt.x(), pt.y());
            }
        }
        this._geom = outer;
        return outer;
    }

    toGeoJSON(x: number, y: number, z: number): Feature {
        const feat = this._feat;
        const granularity = EXTENT * Math.pow(2, z);
        const tileWest = EXTENT * x;
        const tileNorth = EXTENT * y;

        // Spherical Mercator tile coordinates to WGS84
        function project(line: PVTGeometry): Position[] {
            const len = line.pointsLength();
            const coords = new Array<number[]>(len);
            // we want to loop a full line in the function, since JS can't inline functions
            for (let j = 0; j < len; j++) {
                const p = line.points(j)!;
                const mercY = 180 - ((tileNorth + p.y()) * 360) / granularity;
                const lon = ((tileWest + p.x()) * 360) / granularity - 180;
                const lat = (360 / Math.PI) * Math.atan(Math.exp((mercY * Math.PI) / 180)) - 90;
                coords[j] = [lon, lat];
            }
            return coords;
        }

        const len = feat.geometryLength();
        const type = this.type;

        // MultiPoint has one less nesting
        if (type === 1) {
            const outerCoordinates = new Array<Position>(len);
            for (let i = 0; i < len; i++) {
                const point = project(feat.geometry(i)!)[0];
                outerCoordinates[i] = point;
            }
            return {
                type: 'Feature',
                geometry: {
                    type: 'MultiPoint',
                    coordinates: outerCoordinates,
                },
                properties: this.properties,
            };
        }

        const outerCoordinates = new Array<Position[]>(len);
        for (let i = 0; i < len; i++) {
            const innerCoordinates = project(feat.geometry(i)!);
            outerCoordinates[i] = innerCoordinates;
        }

        if (type === 2) {
            return {
                type: 'Feature',
                geometry: {
                    type: 'MultiLineString',
                    coordinates: outerCoordinates,
                },
                properties: this.properties,
            };
        }

        // Now we have to figure out winding order to determine which rings are holes.
        const polygons: Position[][][] = [];
        let lastPolygon: Position[][] = [];
        for (const innerCoordinates of outerCoordinates) {
            const area = signedArea(innerCoordinates);
            if (area === 0) {
                continue;
            }
            // outer ring
            if (area > 0) {
                lastPolygon = [innerCoordinates];
                polygons.push(lastPolygon);
            }
            // inner ring
            else {
                lastPolygon.push(innerCoordinates);
            }
        }

        return {
            type: 'Feature',
            geometry: {
                type: 'MultiPolygon',
                coordinates: polygons,
            },
            properties: this.properties,
        };
    }

    // bbox is not required or used
    // bbox?(): [number, number, number, number];
}

function getVal(tile: PVTTile, idx: number): string | number | boolean {
    const val = tile.values(idx)!;
    const t = val.t();
    if (t === PVTValueType.String) {
        return tile.strings(val.v());
    }
    if (t === PVTValueType.Number) {
        return val.v();
    }
    return !!val.v();
}

function signedArea(ring): number {
    let sum = 0;
    for (let i = 0, len = ring.length, j = len - 1; i < len; j = i++) {
        const p1 = ring[i];
        const p2 = ring[j];
        sum += (p2[0] - p1[0]) * (p1[1] + p2[1]);
    }
    return sum;
}
