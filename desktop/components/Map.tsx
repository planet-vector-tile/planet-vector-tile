import React, { ReactElement, useEffect, useRef } from 'react';
import maplibregl, { LngLat, LngLatBounds } from 'maplibre-gl';

import styles from './Map.module.css';

// const STYLE = 'https://storage.googleapis.com/hellopvt3/charted-territory.json'
const STYLE = 'https://demotiles.maplibre.org/style.json';
// const STYLE = 'https://basemaps-api.arcgis.com/arcgis/rest/services/styles/ArcGIS:ChartedTerritory?type=style&token=AAPK2b935e8bbf564ef581ca3c6fcaa5f2a71ZH84cPqqFvyz3KplFRHP8HyAwJJkh6cnpcQ-qkWh5aiyDQsGJbsXglGx0QM2cPm'

// 2.1.9 is what esri is using

export type MapInfo = {
    bounds: LngLatBounds;
    center: LngLat;
    zoom: number;
    bearing: number;
    pitch: number;
};

interface MapProps {
    children?: ReactElement | ReactElement[];
    onInit?: (map: maplibregl.Map) => void;
}

export default function Map({ children, onInit }: MapProps) {
    const mapContainerRef = useRef(null);

    // Initialize map when component mounts
    useEffect(() => {
        const map = new maplibregl.Map({
            container: mapContainerRef.current,
            style: STYLE,
            center: [5, 34],
            zoom: 2,
        });

        // Add navigation control (the +/- zoom buttons)
        map.addControl(
            new maplibregl.NavigationControl({
                showCompass: true,
                showZoom: true,
                visualizePitch: true,
            }),
            'bottom-left'
        );

        onInit?(map) : null;

        // Clean up on unmount
        return () => map.remove();
    }, []);

    return (
        <div className={styles.mapContainer} ref={mapContainerRef}>
            {children}
        </div>
    );
}
