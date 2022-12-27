import { XMarkIcon } from '@heroicons/react/24/outline'
import { Resizable } from 're-resizable'

import Layers from './Layers'
import Features from './Features'
import { Page, Info } from './types'
import { registerCloseInfoPanel } from './hotkeys'
import { clearClickedFeatures } from './selection'

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

export default function InfoPanel({ nav, setNav }) {
  if (nav.info === Info.None || nav.page === Page.Planets) {
    return null
  }

  function close() {
    setNav({ ...nav, info: Info.None })
    clearClickedFeatures()
  }
  registerCloseInfoPanel(close)

  return (
    <>
      <Close onClose={close} />
      <Resizable
        defaultSize={{ width: 314, height: '100vh' }}
        minWidth={200}
        maxWidth='100%'
        enable={enable}
        style={{ position: 'fixed' }}
        className='right-0 bg-slate-700/90 border-l border-gray-900 backdrop-blur-md overflow-y-auto'
      >
        {nav.info === 'layers' && <Layers page={nav.page} />}
        {nav.info === 'features' && <Features close={close} />}
      </Resizable>
    </>
  )
}

function Close({ onClose }) {
  return (
    <button
      type='button'
      className='absolute top-11 z-20 right-2 rounded-md text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white focus:ring-1 focus:ring-offset-1 focus:ring-offset-fuchsia-700 focus:ring-fuchsia-700'
      onClick={() => onClose()}
    >
      <span className='sr-only'>Dismiss</span>
      <XMarkIcon className='h-5 w-5' aria-hidden='true' />
    </button>
  )
}
