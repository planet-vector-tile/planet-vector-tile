import { useEffect, useState } from 'react'
import { Switch } from '@headlessui/react'
import { classNames } from './util'

export default function Layers() {
  const [backgroundLayers, setBackgroundLayers] = useState([])
  const [vectorLayers, setVectorLayers] = useState([])

  useEffect(() => {
    const map = window.map

    function processLayers(map) {
      const layers = map.getStyle()?.layers || []
      const background = []
      const vector = []

      for (const layer of layers) {
        switch (layer.type) {
          case 'background':
          case 'raster':
          case 'hillshade':
            background.push(layer)
            break
          default:
            vector.push(layer)
        }
      }

      setBackgroundLayers(background)
      setVectorLayers(vector)
    }

    map.on('load', () => processLayers(map))
    map.on('styledata', () => processLayers(map))
  }, [])

  return (
    <div className='px-3 pt-4'>
      <Background layers={backgroundLayers} />
      <VectorLayers layers={vectorLayers} />
    </div>
  )
}

const backgrounds = [
  { id: 'none', title: 'None' },
  { id: 'sat', title: 'Satellite' },
  { id: 'osm', title: 'OpenStreetMap' },
]

function Background({ layers }) {
  // console.log('backgroundLayers', layers)
  const [opacity, setOpacity] = useState(0.5)
  const [background, setBackground] = useState('sat')

  return (
    <>
      <label className='font-medium text-gray-300'>
        Background <span className='text-xs border rounded px-0.5'>B</span>
      </label>
      <input
        type='range'
        min='0'
        max='1'
        step='.01'
        onChange={e => setOpacity(e.target.value)}
        value={opacity}
        className='w-full h-1 rounded-lg appearance-none cursor-pointer bg-gray-500 color-fuchsia-700 accent-fuchsia-700'
      />
      <fieldset className='mt-4'>
        <legend className='sr-only'>Background</legend>
        <div className='space-y-4'>
          {backgrounds.map(bg => (
            <div key={bg.id} className='flex items-center'>
              <input
                id={bg.id}
                name='notification-method'
                type='radio'
                checked={bg.id === background}
                onChange={() => setBackground(bg.id)}
                className='h-4 w-4 bg-gray-500 text-fuchsia-700 focus:ring-fuchsia-700'
              />
              <label
                htmlFor={backgrounds.id}
                onClick={() => setBackground(bg.id)}
                className={classNames(
                  bg.id === background ? 'text-white' : 'text-gray-400',
                  'ml-3 block text-sm font-light cursor-pointer'
                )}
              >
                {bg.title}
              </label>
            </div>
          ))}
        </div>
      </fieldset>
    </>
  )
}

function VectorLayers({ layers }) {
  const sources = {}
  for (const layer of layers) {
    const source = layer.source
    if (!sources[source]) {
      sources[source] = [layer]
    } else {
      sources[source].push(layer)
    }
  }

  return (
    <>
      {Object.keys(sources).map(source => (
        <VectorLayerGroup key={source} source={source} layers={sources[source]} />
      ))}
    </>
  )
}

function VectorLayerGroup({ source, layers }) {
  return (
    <>
      <h3 className='font-medium text-gray-300 mt-6 mb-2'>{source}</h3>
      {layers.map(layer => (
        <VectorLayer key={layer.id} layer={layer} />
      ))}
    </>
  )
}

function VectorLayer({ layer }) {
  const visibility = window.map.getLayoutProperty(layer.id, 'visibility')
  const enabled = visibility !== 'none'

  function setEnabled() {
    window.map.setLayoutProperty(layer.id, 'visibility', enabled ? 'none' : 'visible')
  }

  return (
    <Switch.Group as='div' className='flex items-center'>
      <Switch
        checked={enabled}
        onChange={setEnabled}
        className={classNames(
          enabled ? 'bg-fuchsia-700' : 'bg-gray-200',
          'relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-fucshia-700 focus:ring-offset-2 focus:ring-offset-fuchsia-700'
        )}
      >
        <span
          aria-hidden='true'
          className={classNames(
            enabled ? 'translate-x-4' : 'translate-x-0',
            'pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out'
          )}
        />
      </Switch>
      <Switch.Label as='span' className='ml-3'>
        <span className='text-sm font-light text-gray-400 cursor-pointer'>{layer.id}</span>
      </Switch.Label>
    </Switch.Group>
  )
}
