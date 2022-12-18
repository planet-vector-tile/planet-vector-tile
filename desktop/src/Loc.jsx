import { useState, useEffect } from 'react'

export default function Loc() {
  const [z, setZ] = useState(0)
  const [lat, setLat] = useState(0)
  const [lon, setLon] = useState(0)

  useEffect(() => {
    const map = window.map

    function loc() {
      const z = map.getZoom()
      const c = map.getCenter()
      setZ(z.toFixed(1))
      setLat(c.lat.toFixed(4))
      setLon(c.lng.toFixed(4))
    }
    loc()
    map.on('move', () => loc())

    // Clean up on unmount
    return () => map.off('move', loc)
  }, [])

  return (
    <span className='fixed bottom-0 left-0 bg-slate-700/50 backdrop-blur-md text-xs text-gray-300 px-1 font-mono'>
      z:{z} {lat},{lon}
    </span>
  )
}
