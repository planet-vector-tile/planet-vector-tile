import { useState } from 'react'
import { Switch } from '@headlessui/react'
import { classNames } from './util'

export default function Layers() {
  return (
    <div className='px-3 pt-4'>
      <Background />
      <SourceLayers />
    </div>
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
                className='h-4 w-4 bg-gray-500 text-fuchsia-700 focus:ring-fuchsia-700'
              />
              <label
                htmlFor={backgrounds.id}
                onClick={() => setBackground(bg.id)}
                className={classNames(
                  bg.id === background ? 'text-white' : 'text-gray-400',
                  'ml-3 block text-medium font-light cursor-pointer'
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

function SourceLayers() {
  const [enabled, setEnabled] = useState(false)

  return (
    <>
      <h3 className='font-medium text-gray-300 mt-6 mb-2'>Layers</h3>
      <Switch.Group as='div' className='flex items-center'>
        <Switch
          checked={enabled}
          onChange={setEnabled}
          className={classNames(
            enabled ? 'bg-fuchsia-700' : 'bg-gray-200',
            'relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-fucshia-700 focus:ring-offset-2 focus:ring-offset-fuchsia-700'
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
        <Switch.Label as='span' className='ml-3'>
          <span className='text-md font-light text-gray-400 cursor-pointer'>Buildings</span>
        </Switch.Label>
      </Switch.Group>
    </>
  )
}
