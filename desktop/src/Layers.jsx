import { useState } from 'react'
import { Switch } from '@headlessui/react'

export default function Layers() {
  return (
    <>
      Layers
      <fieldset className='space-y-3 ml-2 mr-2 mb-2'>
        <legend className='sr-only'>Layers</legend>
        <Layer />
        <Layer />
        <Layer />
      </fieldset>
    </>
  )
}

function classNames(...classes) {
  return classes.filter(Boolean).join(' ')
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
