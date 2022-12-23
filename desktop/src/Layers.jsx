import { useEffect, useState, memo } from 'react'
import { Switch } from '@headlessui/react'
import { classNames } from './util'
import store from './store'
import { map } from './map'
import { DataLayerType, Page } from './types'
import { dataLayerNameAndType } from './datastyle'

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
      <MemoVectorLayers layers={vectorLayers} page={page} />
      <div className='h-10' />
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
      if (page === Page.Map) {
        store.mapStyle = map.getStyle()
      }
      if (page === Page.Data) {
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
        className='w-full h-1 rounded-lg appearance-none cursor-pointer bg-gray-500 color-fuchsia-700 accent-fuchsia-700 disabled:cursor-auto disabled:opacity-20'
        disabled={!selectedBackground}
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

// memo stops React from re-rendering VectorLayers unless the actual vectorLayers prop chages.
// React tries to re-render layers when the page changes, and MapLibre may not have finished switching up the layers yet.
const MemoVectorLayers = memo(VectorLayers)

function VectorLayers({ layers, page }) {
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
        <VectorLayerGroup key={source} source={source} layers={sources[source]} page={page} />
      ))}
    </>
  )
}

function VectorLayerGroup({ source, layers, page }) {
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
        {page === Page.Data
          ? processDataLayers(layers).map(dataLayer => <DataLayer key={dataLayer.name} dataLayer={dataLayer} />)
          : layers.map(layer => <VectorLayer key={layer.id} layer={layer} page={page} />)}
      </ul>
    </div>
  )
}

const MUTE_AND_SOLO_STYLE =
  'border border-gray-600/40 group-hover:border-gray-500 rounded-md font-light text-sm group-hover:text-gray-300 group-hover:border-gray-500'

function VectorLayer({ layer, page }) {
  const isMuted = layer.layout?.visibility === 'none'
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
              isMuted ? 'text-amber-600 group-hover:bg-amber-600 shadow-inner' : 'text-gray-500',
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
              isSolo ? 'text-lime-600 group-hover:bg-lime-600 hadow-inner' : 'text-gray-500',
              MUTE_AND_SOLO_STYLE,
              'px-1.5 group-hover:shadow-md'
            )}
          >
            S
          </button>
        </div>
      </div>

      <div className='flex-1 font-light text-sm text-white'>{page === Page.Data ? layer.dataLayerName : layer.id}</div>
    </li>
  )
}

function processDataLayers(layers) {
  const dataLayers = {}
  const list = [] // preserve layer order

  for (const layer of layers) {
    const { name, type } = dataLayerNameAndType(layer.id)

    if (type !== DataLayerType.Fill && type !== DataLayerType.Line && type !== DataLayerType.Circle) {
      continue
    }
    let dataLayer = dataLayers[name]
    if (!dataLayer) {
      dataLayer = {
        name: name,
        layers: {
          fill: null,
          line: null,
          circle: null,
        },
      }
      dataLayers[name] = dataLayer
      list.push(dataLayer)
    }

    if (type === DataLayerType.Fill) {
      dataLayer.layers.fill = layer
    }
    if (type === DataLayerType.Line) {
      dataLayer.layers.line = layer
    }
    if (type === DataLayerType.Circle) {
      dataLayer.layers.circle = layer
    }
  }
  return list
}

// layer id to layer
const beforeMuteLayer = {}
const mutedDataLayers = new Set()

function DataLayer({ dataLayer }) {
  const isMuted = mutedDataLayers.has(dataLayer.name)
  const isSolo = false

  function toggleMute() {
    dataLayer.layers.values().map(layer => {
      if (isMuted) {
        const visibilty = beforeMuteLayer[layer.id]?.layout?.visibility || 'visible'
        map.setLayoutProperty(layer.id, 'visibility', visibilty)
      } else {
        beforeMuteLayer[layer.id] = layer
        map.setLayoutProperty(layer.id, 'visibility', 'none')
      }
    })
    if (isMuted) {
      mutedDataLayers.delete(dataLayer.name)
    } else {
      mutedDataLayers.add(dataLayer.name)
    }
  }

  function toggleSolo() {}

  return (
    <li key={dataLayer.name} className='group relative flex items-center space-x-3 pl-1 pr-2 pb-1 cursor-default'>
      <div className='flex-shrink-0'>
        <div className='text-center'>
          <button
            title='Mute'
            className={classNames(
              isMuted ? 'text-amber-600 group-hover:bg-amber-600 shadow-inner' : 'text-gray-500',
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
              isSolo ? 'text-lime-600 group-hover:bg-lime-600 shadow-inner' : 'text-gray-500',
              MUTE_AND_SOLO_STYLE,
              'px-1.5 group-hover:shadow-md'
            )}
          >
            S
          </button>
        </div>
      </div>

      <div className='flex-1 font-light text-sm text-white'>{dataLayer.name}</div>

      <FLC dataLayer={dataLayer} />
    </li>
  )
}

function FLC({ dataLayer }) {
  const f = dataLayer.layers.fill?.layout?.visibility !== 'none'
  const l = dataLayer.layers.line?.layout?.visibility !== 'none'
  const c = dataLayer.layers.circle?.layout?.visibility !== 'none'

  const ON_STYLE = 'text-fuchsia-700 group-hover:text-gray-300 group-hover:bg-fuchsia-700/80 shadow-inner'
  const OFF_STYLE = 'text-gray-500 group-hover:text-gray-300 group-hover:shadow-md'

  function toggleF() {
    let fill = dataLayer.layers.fill
    if (fill) {
      map.setLayoutProperty(fill.id, 'visibility', f ? 'none' : 'visible')
    }
  }

  function toggleL() {
    let line = dataLayer.layers.line
    if (line) {
      map.setLayoutProperty(line.id, 'visibility', l ? 'none' : 'visible')
    }
  }

  function toggleC() {
    let circle = dataLayer.layers.circle
    if (circle) {
      map.setLayoutProperty(circle.id, 'visibility', c ? 'none' : 'visible')
    }
  }

  return (
    <div className='flex-shrink-0 space-y-0.5'>
      <button
        title='Fill'
        className={classNames(
          f ? ON_STYLE : OFF_STYLE,
          'rounded-l-md border border-gray-600/40 px-1 font-light text-sm text-center group-hover:border-gray-500 '
        )}
        onClick={toggleF}
      >
        F
      </button>
      <button
        title='Line'
        className={classNames(
          l ? ON_STYLE : OFF_STYLE,
          'border-t border-b border-gray-600/40 px-1 font-light text-sm text-center group-hover:border-gray-500 '
        )}
        onClick={toggleL}
      >
        L
      </button>
      <button
        title='Circle'
        className={classNames(
          c ? ON_STYLE : OFF_STYLE,
          'rounded-r-md border border-gray-600/40 px-1 font-light text-sm text-center group-hover:border-gray-500 '
        )}
        onClick={toggleC}
      >
        C
      </button>
    </div>
  )
}
