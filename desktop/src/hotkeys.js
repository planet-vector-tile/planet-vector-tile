import { map } from './map'

export async function setupHotKeys() {
  document.addEventListener('keyup', event => {
    console.log('keyup', event.key)
    switch (event.key) {
      case 'b':
        toggleBackground()
        break
      case 'Escape':
        closeInfoPanel()
        break
    }
  })
}

// Background toggle with 'b' key
let bCount = 0
function toggleBackground() {
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

let closeInfoPanel = () => {}
export function registerCloseInfoPanel(fn) {
  closeInfoPanel = fn
}
