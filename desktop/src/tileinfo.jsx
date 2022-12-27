import { classNames } from "./util"

import { Fragment, useState } from 'react'
import { Listbox, Transition } from '@headlessui/react'
import { ChevronUpDownIcon } from '@heroicons/react/20/solid'

import { map } from './map'

export function tileInfoFillPaint(sourceLayerId) {
  switch (sourceLayerId) {
    case 'Tile Boundary':
      return {
        'fill-color': 'rgb(255, 255, 255)',
        'fill-opacity': ['case', ['boolean', ['feature-state', 'hover'], false], 0.1, 0.001],
      }
    default:
      return null
  }
}

export function tileInfoLinePaint(sourceLayerId) {
  switch (sourceLayerId) {
    case 'Tile Boundary':
      return {
        'line-color': [
          'case',
          ['==', ['get', 'is_render_tile'], true],
          'rgba(255, 255, 255, 0.4)',
          'rgba(255, 255, 255, 0.1)',
        ],
        'line-width': ['case', ['==', ['get', 'is_render_tile'], true], 2, 1],
      }
    case 'Tile Bearing':
      return {
        'line-color': '#ff69b4',
        'line-width': 3,
      }
    default:
      return null
  }
}

export function tileInfoLabelStyle(circleLayerId, sourceId, sourceLayerId, circleLayerVisibility) {
  if (sourceLayerId !== 'Tile Center') {
    return null
  }
  return {
    id: circleLayerId,
    type: 'symbol',
    source: sourceId,
    'source-layer': sourceLayerId,
    layout: {
      'text-field': '{h}',
      'text-size': 14,
      visibility: circleLayerVisibility,
    },
    paint: {
      'text-color': 'white',
      'text-halo-color': 'rgba(0,0,0,0.8)',
      'text-halo-blur': 1,
      'text-halo-width': 1,
    },
  }
}


const zooms = ['All', 2, 4, 6, 8, 10, 12, 14]

export function TileInfoZoom({ dataLayer }) {
  if (dataLayer.name.indexOf('Tile ') !== 0) {
    return null
  }

  const filter = dataLayer.layers.line.filter
  const z = filter && filter[1][2] || 'All'
  const [selected, _setSelected] = useState(parseInt(z) || 'All')

  function setSelected(z) {
    const zoom = parseInt(z)
    if (isNaN(zoom)) {
      delete layer.filter
    }
    for (const layer of Object.values(dataLayer.layers)) {
      const filter = ['all', ['==', 'z', zoom]]
      layer.filter = filter
      map.setFilter(layer.id, filter)
    }
    store.dataStyle = map.getStyle()
    _setSelected(z)
  }

  return (
    <Listbox value={selected} onChange={setSelected}>
      {({ open }) => (
        <span className="flex items-center">
          <Listbox.Label className="block text-sm font-light text-gray-500 group-hover:text-gray-300 pt-0.5 pr-0.5">Z:</Listbox.Label>
          <div className="relative">
            <Listbox.Button className="relative rounded-md border border-gray-600 group-hover:border-gray-500 group-hover-text-gray-500 bg-transparent py-0 pl-2 pr-6 text-left shadow-sm text-sm cursor-pointer">
              <span className="block truncate text-gray-500 group-hover:text-gray-300">{selected}</span>
              <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-0">
                <ChevronUpDownIcon className="h-4 w-5 text-gray-500" aria-hidden="true" />
              </span>
            </Listbox.Button>

            <Transition
              show={open}
              as={Fragment}
              leave="transition ease-in duration-100"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <Listbox.Options className="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-gray-700 py-1 text-base shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none sm:text-sm">
                {zooms.map((z) => (
                  <Listbox.Option
                    key={z}
                    className={({ active }) =>
                      classNames(
                        active ? 'text-gray-300 bg-fuchsia-700' : 'text-gray-300',
                        'relative select-none cursor-pointer py-1 pl-3 pr-9'
                      )
                    }
                    value={z}
                  >
                    {({ selected, active }) => (
                        <span className={classNames(selected ? 'font-semibold' : 'font-normal')}>
                          {z}
                        </span>
                    )}
                  </Listbox.Option>
                ))}
              </Listbox.Options>
            </Transition>
          </div>
        </span>
      )}
    </Listbox>
  )
}

