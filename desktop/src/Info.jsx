import { XMarkIcon } from '@heroicons/react/24/outline'
import { Resizable } from 're-resizable'

import Layers from './Layers'
import Features from './Features'

const enable = {
  top: false,
  right: false,
  bottom: false,
  left: true,
  topRight: false,
  bottomRight: false,
  bottomLeft: false,
  topLeft: false,
}

export default function Info({ nav, setNav }) {
  if (!nav.info || nav.info === 'none' || nav.page === 'planets') {
    return null
  }
  return (
    <Resizable
      defaultSize={{ width: 314, height: '100vh' }}
      minWidth={200}
      maxWidth='100%'
      enable={enable}
      style={{ position: 'fixed' }}
      className='right-0 bg-slate-700/80 border-l border-gray-900 backdrop-blur-md overflow-y-auto'
    >
      <Close onClose={() => setNav({ ...nav, info: null })} />
      {nav.info === 'layers' && <Layers />}
      {nav.info === 'features' && <Features />}
    </Resizable>
  )
}

function Close({ onClose }) {
  return (
    <button
      type='button'
      className='absolute top-1 z-20 right-2 rounded-md text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white focus:ring-1 focus:ring-offset-1 focus:ring-offset-fuchsia-700 focus:ring-fuchsia-700'
      onClick={() => onClose()}
    >
      <span className='sr-only'>Dismiss</span>
      <XMarkIcon className='h-5 w-5' aria-hidden='true' />
    </button>
  )
}
