// automatically generated by the FlatBuffers compiler, do not modify

import * as flatbuffers from 'flatbuffers';

import { Point } from '../planet-vector-tile/point';


export class Geometry {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):Geometry {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsGeometry(bb:flatbuffers.ByteBuffer, obj?:Geometry):Geometry {
  return (obj || new Geometry()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsGeometry(bb:flatbuffers.ByteBuffer, obj?:Geometry):Geometry {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new Geometry()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

points(index: number, obj?:Point):Point|null {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? (obj || new Point()).__init(this.bb!.__vector(this.bb_pos + offset) + index * 8, this.bb!) : null;
}

pointsLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

static startGeometry(builder:flatbuffers.Builder) {
  builder.startObject(1);
}

static addPoints(builder:flatbuffers.Builder, pointsOffset:flatbuffers.Offset) {
  builder.addFieldOffset(0, pointsOffset, 0);
}

static startPointsVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(8, numElems, 4);
}

static endGeometry(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  return offset;
}

static createGeometry(builder:flatbuffers.Builder, pointsOffset:flatbuffers.Offset):flatbuffers.Offset {
  Geometry.startGeometry(builder);
  Geometry.addPoints(builder, pointsOffset);
  return Geometry.endGeometry(builder);
}
}
