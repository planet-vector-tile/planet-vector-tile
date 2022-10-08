// automatically generated by the FlatBuffers compiler, do not modify

import * as flatbuffers from 'flatbuffers';

import { PVTFeature } from './pvtfeature';


export class PVTLayer {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):PVTLayer {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsPVTLayer(bb:flatbuffers.ByteBuffer, obj?:PVTLayer):PVTLayer {
  return (obj || new PVTLayer()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsPVTLayer(bb:flatbuffers.ByteBuffer, obj?:PVTLayer):PVTLayer {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new PVTLayer()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

name():number {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.readUint32(this.bb_pos + offset) : 0;
}

features(index: number, obj?:PVTFeature):PVTFeature|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? (obj || new PVTFeature()).__init(this.bb!.__indirect(this.bb!.__vector(this.bb_pos + offset) + index * 4), this.bb!) : null;
}

featuresLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

static startPVTLayer(builder:flatbuffers.Builder) {
  builder.startObject(2);
}

static addName(builder:flatbuffers.Builder, name:number) {
  builder.addFieldInt32(0, name, 0);
}

static addFeatures(builder:flatbuffers.Builder, featuresOffset:flatbuffers.Offset) {
  builder.addFieldOffset(1, featuresOffset, 0);
}

static createFeaturesVector(builder:flatbuffers.Builder, data:flatbuffers.Offset[]):flatbuffers.Offset {
  builder.startVector(4, data.length, 4);
  for (let i = data.length - 1; i >= 0; i--) {
    builder.addOffset(data[i]!);
  }
  return builder.endVector();
}

static startFeaturesVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(4, numElems, 4);
}

static endPVTLayer(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  return offset;
}

static createPVTLayer(builder:flatbuffers.Builder, name:number, featuresOffset:flatbuffers.Offset):flatbuffers.Offset {
  PVTLayer.startPVTLayer(builder);
  PVTLayer.addName(builder, name);
  PVTLayer.addFeatures(builder, featuresOffset);
  return PVTLayer.endPVTLayer(builder);
}
}
