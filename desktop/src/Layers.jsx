import { useEffect, useState } from 'react'
import { Switch } from '@headlessui/react'
import { classNames } from './util'
import store from './store'
import { map } from './map'

export default function Layers({ page }) {
  const [backgroundLayers, setBackgroundLayers] = useState([])
  const [vectorLayers, setVectorLayers] = useState([])

  useEffect(() => {
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

    // At initial load, sometimes the map isn't ready yet and maplibre throws an error,
    // so we wait for events to try again.
    try {
      map.getStyle() // the lib throws an error when serializing the style
      processLayers(map)
    } catch (_) {}

    map.on('load', () => processLayers(map))
    map.on('styledata', () => processLayers(map))
  }, [])

  return (
    <div className='pt-4'>
      <Background layers={backgroundLayers} page={page} />
      <VectorLayers layers={vectorLayers} />
      <div className='h-14' />
    </div>
  )
}

const backgrounds = [
  { id: 'none', title: 'None' },
  { id: 'sat', title: 'Satellite' },
  { id: 'osm', title: 'OpenStreetMap' },
]

function Background({ layers, page }) {
  const selectedBackground = layers.find(
    layer => layer.type === 'raster' && map.getLayoutProperty(layer.id, 'visibility') !== 'none'
  )

  const opacity = selectedBackground?.paint['raster-opacity'] || 1

  function setOpacity(opacity) {
    if (selectedBackground) {
      map.setPaintProperty(selectedBackground.id, 'raster-opacity', opacity)
      if (page === 'map') {
        store.mapStyle = map.getStyle()
      }
      if (page === 'data') {
        store.dataStyle = map.getStyle()
      }
    }
  }

  function isChecked(id) {
    if (id === 'none') {
      return !selectedBackground
    }
    return selectedBackground?.id === id
  }

  function setBackground(id) {
    for (const bg of backgrounds) {
      if (bg.id === 'none') {
        continue
      }
      if (bg.id === id) {
        map.setLayoutProperty(bg.id, 'visibility', 'visible')
      } else {
        map.setLayoutProperty(bg.id, 'visibility', 'none')
      }
    }
  }

  return (
    <section className='px-4 pb-5'>
      <label className='font-medium text-gray-300'>
        Background <span className='text-xs border rounded px-0.5'>B</span>
      </label>
      <input
        type='range'
        min='0'
        max='1'
        step='.01'
        onChange={e => setOpacity(parseFloat(e.target.value))}
        value={opacity}
        className='w-full h-1 rounded-lg appearance-none cursor-pointer bg-gray-500 color-fuchsia-700 accent-fuchsia-700'
      />
      <fieldset className='mt-2 '>
        <legend className='sr-only'>Background</legend>
        <div className='space-y-2 '>
          {backgrounds.map(bg => (
            <div key={bg.id} className='flex items-center'>
              <input
                id={bg.id}
                name='notification-method'
                type='radio'
                checked={isChecked(bg.id)}
                onChange={() => setBackground(bg.id)}
                className='h-4 w-4 bg-gray-500 text-fuchsia-700 focus:ring-fuchsia-700'
              />
              <label
                htmlFor={backgrounds.id}
                onClick={() => setBackground(bg.id)}
                className={classNames(
                  isChecked(bg.id) ? 'text-white' : 'text-gray-400',
                  'ml-3 block text-sm font-light cursor-pointer'
                )}
              >
                {bg.title}
              </label>
            </div>
          ))}
        </div>
      </fieldset>
    </section>
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
  const [enabled, setEnabled] = useState(true)
  return (
    <div key={source} className='relative'>
      <Switch.Group
        as='div'
        className='flex items-center sticky top-0 z-10 border-t border-b border-gray-600 bg-gray-700 px-3 pt-0.5 pb-1 font-medium text-gray-300'
      >
        <Switch
          checked={enabled}
          onChange={setEnabled}
          className={classNames(
            enabled ? 'bg-fuchsia-700/80' : 'bg-gray-200',
            'relative inline-flex h-3 w-6 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-fucshia-700 focus:ring-offset-2 focus:ring-offset-fuchsia-700'
          )}
        >
          <span
            aria-hidden='true'
            className={classNames(
              enabled ? 'translate-x-3' : 'translate-x-0',
              'pointer-events-none inline-block h-2 w-2 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out'
            )}
          />
        </Switch>
        <Switch.Label as='span' className='ml-3'>
          <span className='cursor-pointer'>{source}</span>
        </Switch.Label>
      </Switch.Group>
      <ul role='list' className='divide-y divide-gray-600'>
        {layers.map(layer => (
          <VectorLayer key={layer.id} layer={layer} />
        ))}
      </ul>
    </div>
  )
}

const MUTE_AND_SOLO_STYLE =
  'border border-gray-600/40 group-hover:border-gray-500 rounded-md font-light text-sm group-hover:text-gray-300'

function VectorLayer({ layer }) {
  const visibility = map.getLayoutProperty(layer.id, 'visibility')
  const isMuted = visibility === 'none'
  const isSolo = false

  function toggleMute() {
    map.setLayoutProperty(layer.id, 'visibility', isMuted ? 'visible' : 'none')
  }

  function toggleSolo() {}

  return (
    <li key={layer.id} className='group relative flex items-center space-x-3 px-1 pb-1 cursor-default'>
      <div className='flex-shrink-0'>
        <div className='text-center'>
          <button
            title='Mute'
            className={classNames(
              isMuted
                ? 'text-amber-600 group-hover:bg-amber-600 group-hover:border-amber-600 shadow-inner'
                : 'text-gray-500',
              MUTE_AND_SOLO_STYLE,
              'px-1 group-hover:shadow-md'
            )}
            onClick={toggleMute}
          >
            M
          </button>
        </div>

        <div className='text-center'>
          <button
            title='Solo'
            className={classNames(
              isSolo
                ? 'text-lime-600 group-hover:bg-lime-600 group-hover:border-lime-600 shadow-inner'
                : 'text-gray-500',
              MUTE_AND_SOLO_STYLE,
              'px-1.5 group-hover:shadow-md'
            )}
          >
            S
          </button>
        </div>
      </div>

      <div className='flex-1 font-light text-sm text-white'>{layer.id}</div>

      <div className='flex-shrink-0 space-y-0.5'>
        <button
          title='Circle'
          className='rounded-l-md border border-gray-600/40 group-hover:border-gray-500 group-hover:shadow-md px-1 font-light text-sm text-gray-500 group-hover:text-gray-300 text-center'
        >
          C
        </button>
        <button
          title='Line'
          className='border-t border-b border-gray-600/40 group-hover:border-gray-500 group-hover:shadow-md px-1 font-light text-sm text-gray-500 group-hover:text-gray-300 text-center'
        >
          L
        </button>
        <button
          title='Fill'
          className='rounded-r-md border border-gray-600/40 group-hover:border-gray-500 group-hover:shadow-md px-1 font-light text-sm text-gray-500 group-hover:text-gray-300 text-center'
        >
          F
        </button>
      </div>
    </li>
  )
}
