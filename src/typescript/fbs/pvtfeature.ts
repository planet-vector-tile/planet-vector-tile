// automatically generated by the FlatBuffers compiler, do not modify

import * as flatbuffers from 'flatbuffers';

import { PVTGeometry } from './pvtgeometry.js';

export class PVTFeature {
    bb: flatbuffers.ByteBuffer | null = null;
    bb_pos = 0;
    __init(i: number, bb: flatbuffers.ByteBuffer): PVTFeature {
        this.bb_pos = i;
        this.bb = bb;
        return this;
    }

    static getRootAsPVTFeature(bb: flatbuffers.ByteBuffer, obj?: PVTFeature): PVTFeature {
        return (obj || new PVTFeature()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
    }

    static getSizePrefixedRootAsPVTFeature(bb: flatbuffers.ByteBuffer, obj?: PVTFeature): PVTFeature {
        bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
        return (obj || new PVTFeature()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
    }

    id(): bigint {
        const offset = this.bb!.__offset(this.bb_pos, 4);
        return offset ? this.bb!.readUint64(this.bb_pos + offset) : BigInt('0');
    }

    keys(index: number): number | null {
        const offset = this.bb!.__offset(this.bb_pos, 6);
        return offset ? this.bb!.readUint32(this.bb!.__vector(this.bb_pos + offset) + index * 4) : 0;
    }

    keysLength(): number {
        const offset = this.bb!.__offset(this.bb_pos, 6);
        return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
    }

    keysArray(): Uint32Array | null {
        const offset = this.bb!.__offset(this.bb_pos, 6);
        return offset
            ? new Uint32Array(
                  this.bb!.bytes().buffer,
                  this.bb!.bytes().byteOffset + this.bb!.__vector(this.bb_pos + offset),
                  this.bb!.__vector_len(this.bb_pos + offset)
              )
            : null;
    }

    values(index: number): number | null {
        const offset = this.bb!.__offset(this.bb_pos, 8);
        return offset ? this.bb!.readUint32(this.bb!.__vector(this.bb_pos + offset) + index * 4) : 0;
    }

    valuesLength(): number {
        const offset = this.bb!.__offset(this.bb_pos, 8);
        return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
    }

    valuesArray(): Uint32Array | null {
        const offset = this.bb!.__offset(this.bb_pos, 8);
        return offset
            ? new Uint32Array(
                  this.bb!.bytes().buffer,
                  this.bb!.bytes().byteOffset + this.bb!.__vector(this.bb_pos + offset),
                  this.bb!.__vector_len(this.bb_pos + offset)
              )
            : null;
    }

    geometries(index: number, obj?: PVTGeometry): PVTGeometry | null {
        const offset = this.bb!.__offset(this.bb_pos, 10);
        return offset
            ? (obj || new PVTGeometry()).__init(
                  this.bb!.__indirect(this.bb!.__vector(this.bb_pos + offset) + index * 4),
                  this.bb!
              )
            : null;
    }

    geometriesLength(): number {
        const offset = this.bb!.__offset(this.bb_pos, 10);
        return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
    }

    static startPVTFeature(builder: flatbuffers.Builder) {
        builder.startObject(4);
    }

    static addId(builder: flatbuffers.Builder, id: bigint) {
        builder.addFieldInt64(0, id, BigInt('0'));
    }

    static addKeys(builder: flatbuffers.Builder, keysOffset: flatbuffers.Offset) {
        builder.addFieldOffset(1, keysOffset, 0);
    }

    static createKeysVector(builder: flatbuffers.Builder, data: number[] | Uint32Array): flatbuffers.Offset;
    /**
     * @deprecated This Uint8Array overload will be removed in the future.
     */
    static createKeysVector(builder: flatbuffers.Builder, data: number[] | Uint8Array): flatbuffers.Offset;
    static createKeysVector(
        builder: flatbuffers.Builder,
        data: number[] | Uint32Array | Uint8Array
    ): flatbuffers.Offset {
        builder.startVector(4, data.length, 4);
        for (let i = data.length - 1; i >= 0; i--) {
            builder.addInt32(data[i]!);
        }
        return builder.endVector();
    }

    static startKeysVector(builder: flatbuffers.Builder, numElems: number) {
        builder.startVector(4, numElems, 4);
    }

    static addValues(builder: flatbuffers.Builder, valuesOffset: flatbuffers.Offset) {
        builder.addFieldOffset(2, valuesOffset, 0);
    }

    static createValuesVector(builder: flatbuffers.Builder, data: number[] | Uint32Array): flatbuffers.Offset;
    /**
     * @deprecated This Uint8Array overload will be removed in the future.
     */
    static createValuesVector(builder: flatbuffers.Builder, data: number[] | Uint8Array): flatbuffers.Offset;
    static createValuesVector(
        builder: flatbuffers.Builder,
        data: number[] | Uint32Array | Uint8Array
    ): flatbuffers.Offset {
        builder.startVector(4, data.length, 4);
        for (let i = data.length - 1; i >= 0; i--) {
            builder.addInt32(data[i]!);
        }
        return builder.endVector();
    }

    static startValuesVector(builder: flatbuffers.Builder, numElems: number) {
        builder.startVector(4, numElems, 4);
    }

    static addGeometries(builder: flatbuffers.Builder, geometriesOffset: flatbuffers.Offset) {
        builder.addFieldOffset(3, geometriesOffset, 0);
    }

    static createGeometriesVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset {
        builder.startVector(4, data.length, 4);
        for (let i = data.length - 1; i >= 0; i--) {
            builder.addOffset(data[i]!);
        }
        return builder.endVector();
    }

    static startGeometriesVector(builder: flatbuffers.Builder, numElems: number) {
        builder.startVector(4, numElems, 4);
    }

    static endPVTFeature(builder: flatbuffers.Builder): flatbuffers.Offset {
        const offset = builder.endObject();
        return offset;
    }

    static createPVTFeature(
        builder: flatbuffers.Builder,
        id: bigint,
        keysOffset: flatbuffers.Offset,
        valuesOffset: flatbuffers.Offset,
        geometriesOffset: flatbuffers.Offset
    ): flatbuffers.Offset {
        PVTFeature.startPVTFeature(builder);
        PVTFeature.addId(builder, id);
        PVTFeature.addKeys(builder, keysOffset);
        PVTFeature.addValues(builder, valuesOffset);
        PVTFeature.addGeometries(builder, geometriesOffset);
        return PVTFeature.endPVTFeature(builder);
    }
}
