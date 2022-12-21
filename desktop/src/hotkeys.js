import { map } from './map'

export async function setupHotKeys() {
  // Background toggle with 'b' key
  let bCount = 0
  document.addEventListener('keypress', event => {
    console.log('keypress', event.key)
    if (event.key === 'b') {
      switch (bCount % 3) {
        case 0:
          map.setLayoutProperty('sat', 'visibility', 'none')
          map.setLayoutProperty('osm', 'visibility', 'none')
          break
        case 1:
          map.setLayoutProperty('sat', 'visibility', 'visible')
          map.setLayoutProperty('osm', 'visibility', 'none')
          break
        case 2:
          map.setLayoutProperty('sat', 'visibility', 'none')
          map.setLayoutProperty('osm', 'visibility', 'visible')
          break
      }
      ++bCount
    }
  })
}
