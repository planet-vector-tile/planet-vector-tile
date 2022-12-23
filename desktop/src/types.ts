
export type Store = {
    nav: {
      page: Page,
      info: Info,
    },
    bbox: Array<number> | null,
    mapStyle: Object,
    dataStyle: Object,
  }

export enum Page {
    Planets = 'planets',
    Map = 'map',
    Data = 'data',
}

export enum Info {
    None = 'none',
    Layers = 'layers',
    Features = 'features',
}

declare global {
    interface Window {
        store: Store,
        resetStore: () => void,
        maplibregl: any,
        map: any,
    }
}