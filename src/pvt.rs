use napi::{bindgen_prelude::Array, ContextlessResult, JsObject, Env, sys::PropertyAttributes::enumerable, Property};
use napi_derive::{napi, contextless_function};
use std::collections::HashMap;

pub struct PVT2 {
    pub layers: u16,
}

impl PVT2 {
    pub fn new () -> Self {
        Self { layers:3 }
    }
}

#[napi]
pub fn to_js_obj(env: Env) -> napi::Result<JsObject> {
  let mut arr = env.create_array(0)?;
  arr.insert("a string")?;
  arr.insert(42)?;
  arr.coerce_to_object()
}

// #[napi]
// #[derive(Clone)]
// pub struct PVT2Layer {
//     pub name: String,
//     pub length: u32,
//     features: Vec<PVT2Feature>,

//     // I don't think we need this...
//     // pub extent: u16,
//     // pub version: u8,
// }

// #[napi]
// impl PVT2Layer {
//     #[napi(constructor)]
//     pub fn new(name: String) -> Self {
//         PVT2Layer {
//             name,
//             length: 0,
//             features: Vec::new(),
//         }
//     }

//     pub fn add_feature(&mut self, feature: PVT2Feature) {
//         self.length += 1;
//         self.features.push(feature);
//     }

//     #[napi]
//     pub fn feature(&self, index: u32) -> PVT2Feature {
//         // NHTODO Can we avoid this clone?
//         self.features[index as usize].clone()
//     }
// }

// #[napi(constructor)]
// #[derive(Clone)]
// pub struct PVT2Feature {
//     #[napi(js_name = "type")]
//     pub type_: u8,

//     pub id: Option<f64>,

//     pub properties: HashMap<String, String>,
//     // pub properties: JsObject,

//     // Points are flattened into the inner array.
//     pub geometries: Vec<Vec<i16>>,
//     // I don't think we need this.
//     // pub extent: u16,
// }

// #[napi]
// impl PVT2Feature {
//     #[napi]
//     pub fn load_geometry(&self) -> Vec<Vec<Point>> {
//         let len = self.geometries.len();
//         let mut outer = Vec::with_capacity(len);
//         for i in 0..len {
//             let inner = &self.geometries[i];
//             let inner_len = inner.len();
//             let points_len = inner_len / 2;
//             let mut points = Vec::with_capacity(points_len);
//             let mut j = 0;
//             while j < points_len {
//                 let x = inner[j];
//                 let y = inner[j + 1];
//                 points.push(Point { x, y });
//                 j += 2;
//             }
//             outer.push(points);
//         }
//         outer
//     }
// }

#[napi(object)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}
