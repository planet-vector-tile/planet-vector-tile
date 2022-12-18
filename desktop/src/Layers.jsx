import { useState } from 'react'
import { Switch } from '@headlessui/react'
import { classNames } from './util'

export default function Layers() {
  return (
    <div className='px-3 pt-4'>
      <Background />
      <fieldset className='space-y-3 ml-2 mr-2 mb-2'>
        <legend className='sr-only'>Layers</legend>
        <Layer />
        <Layer />
        <Layer />
      </fieldset>
    </div>
  )
}

function Layer() {
  const [enabled, setEnabled] = useState(false)

  return (
    <Switch.Group as='div' className='flex items-center justify-between'>
      <Switch
        checked={enabled}
        onChange={setEnabled}
        className={classNames(
          enabled ? 'bg-indigo-600' : 'bg-gray-200',
          'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2'
        )}
      >
        <span
          aria-hidden='true'
          className={classNames(
            enabled ? 'translate-x-5' : 'translate-x-0',
            'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out'
          )}
        />
      </Switch>
      <span className='flex flex-grow flex-col ml-2'>
        <Switch.Label as='span' className='text-sm font-medium text-gray-900' passive>
          Satellite
        </Switch.Label>
        <Switch.Description as='span' className='text-sm text-gray-500'>
          Esri World Imagery
        </Switch.Description>
      </span>
    </Switch.Group>
  )
}

const backgrounds = [
  { id: 'none', title: 'None' },
  { id: 'sat', title: 'Satellite' },
  { id: 'osm', title: 'OpenStreetMap' },
]

function Background() {
  const [opacity, setOpacity] = useState(0.5)
  const [background, setBackground] = useState('sat')

  return (
    <>
      <label className='font-medium text-gray-300'>
        Background <span className='text-xs border rounded px-0.5'>B</span>
      </label>
      <div className='px-4'>
        <input
          type='range'
          min='0'
          max='1'
          step='.01'
          onChange={e => setOpacity(e.target.value)}
          value={opacity}
          class='w-full h-1 rounded-lg appearance-none cursor-pointer bg-gray-500 color-fuchsia-700 accent-fuchsia-700'
        />
      </div>
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
                className='h-4 w-4 border-gray-300 text-fuchsia-700 focus:ring-fuchsia-700'
              />
              <label
                htmlFor={backgrounds.id}
                onClick={() => setBackground(bg.id)}
                className='ml-3 block text-sm font-medium text-white'
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
