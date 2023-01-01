# Hilbert Curve

Both the source features and the tiles have an associated Hilbert location, defining where they lie along the [Hilbert space-filling curve](https://en.wikipedia.org/wiki/Hilbert_curve).

![Hilbert Curve](screenshots/hilbert-tile.png)

![Direction](screenshots/hilbert-direction.png)

The Hilbert Tile Tree indexes only even-numbered zooms, as this gives you 16 child tiles for each parent--the perfect size to encode the location of child tiles in a 1-byte bit mask.

This is really fast. Large tiles are encoded in ~200ms, and reasonably sized tiles take ~20-50ms. Take a look at the [Hilbert Tile Tree code](../src/hilbert/) if you are curious.

![Higher zoom](screenshots/higher-zoom-hilbert.png)

Tiles at higher zooms have a higher Hilbert location, but that location will always be at the same fractional location along the Hilbert curve.

## Lots of data

![](screenshots/lots-of-data.jpg)

