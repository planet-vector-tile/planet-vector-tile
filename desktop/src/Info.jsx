import { XMarkIcon } from '@heroicons/react/24/outline'

import Layers from './Layers'
import Features from './Features'

export default function Info({ nav, setNav }) {
  if (!nav.info || nav.info === 'none' || nav.page === 'planets') {
    return null
  }
  return (
    <div className='fixed right-0 w-80 h-full bg-slate-700/80 border-l border-gray-900 backdrop-blur-md'>
      <Close onClose={() => setNav({ ...nav, info: null })} />
      {nav.info === 'layers' && <Layers />}
      {nav.info === 'features' && <Features />}
    </div>
  )
}

function Close({ onClose }) {
  return (
    <button
      type='button'
      className='absolute top-1 right-2 rounded-md text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white focus:ring-1 focus:ring-offset-1 focus:ring-offset-fuchsia-700 focus:ring-fuchsia-700'
      onClick={() => onClose()}
    >
      <span className='sr-only'>Dismiss</span>
      <XMarkIcon className='h-5 w-5' aria-hidden='true' />
    </button>
  )
}
