// Background toggle with 'b' key
let bCount = 0
document.addEventListener('keypress', event => {
  console.log('key', event.key)
  if (event.key === 'b') {
    switch (bCount % 3) {
      case 0:
        map.setLayoutProperty('sat', 'visibility', 'visible')
        map.setLayoutProperty('osm', 'visibility', 'none')
        break
      case 1:
        map.setLayoutProperty('sat', 'visibility', 'none')
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

const hotkeys = {
  b: 'Toggle background',
}

export default hotkeys
