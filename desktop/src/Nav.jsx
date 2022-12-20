import { classNames } from './util'

const store = window.store

export default function Nav({ nav, setNav }) {
  return (
    <nav className='drag z-10 mx-auto bg-gray-800/70 border-b border-gray-900 backdrop-blur-md sm:pl-20 pr-2 sm:pr-20 flex h-10 items-center justify-between'>
      <div className='hidden md:flex items-center'>
        <div className='flex-shrink-0'>
          <svg className='w-8 h-8' strokeWidth='1.5' viewBox='0 0 24 24' fill='none'>
            <circle cx='12' cy='12' r='8' stroke='white' strokeWidth='1.5'></circle>
            <path
              d='M17.5 6.348c2.297-.538 3.945-.476 4.338.312.73 1.466-3.158 4.89-8.687 7.645-5.528 2.757-10.602 3.802-11.333 2.336-.392-.786.544-2.134 2.349-3.64'
              stroke='white'
            ></path>
          </svg>
        </div>
        <span className='text-white px-2 text-xl'>PlanetVectorTile</span>
      </div>

      <PlanetsMapData page={nav.page} setPage={page => setNav({ ...nav, page })} />
      <InfoButtons nav={nav} setNav={setNav} />
    </nav>
  )
}

function PlanetsMapData({ page, setPage }) {
  function togglePage(page) {
    setPage(page)
    if (page === 'map') {
      window.map.setStyle(store.mapStyle)
    }
    if (page === 'data') {
      window.map.setStyle(store.dataStyle)
    }
  }

  const inactive =
    'inline-flex items-center rounded-md px-3 py-1 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white cursor-default focus:outline-none focus:ring-1 focus:ring-offset-1 focus:ring-offset-fuchsia-700 focus:ring-fuchsia-700'
  const active =
    'inline-flex items-center rounded-md bg-gray-900 px-1 sm:px-3 py-1 text-sm font-medium text-white cursor-default focus:outline-none'
  return (
    <div className='flex space-x-1 md:space-x-4'>
      <button className={page === 'planets' ? active : inactive} onClick={() => togglePage('planets')}>
        <svg
          xmlns='http://www.w3.org/2000/svg'
          fill='none'
          viewBox='0 0 24 24'
          strokeWidth={1.5}
          stroke='currentColor'
          className='w-4 h-4'
        >
          <path
            strokeLinecap='round'
            strokeLinejoin='round'
            d='M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418'
          />
        </svg>
        &nbsp;Planets
      </button>
      <button className={page === 'map' ? active : inactive} onClick={() => togglePage('map')}>
        <svg
          xmlns='http://www.w3.org/2000/svg'
          fill='none'
          viewBox='0 0 24 24'
          strokeWidth={1.5}
          stroke='currentColor'
          className='w-4 h-4'
        >
          <path
            strokeLinecap='round'
            strokeLinejoin='round'
            d='M9 6.75V15m6-6v8.25m.503 3.498l4.875-2.437c.381-.19.622-.58.622-1.006V4.82c0-.836-.88-1.38-1.628-1.006l-3.869 1.934c-.317.159-.69.159-1.006 0L9.503 3.252a1.125 1.125 0 00-1.006 0L3.622 5.689C3.24 5.88 3 6.27 3 6.695V19.18c0 .836.88 1.38 1.628 1.006l3.869-1.934c.317-.159.69-.159 1.006 0l4.994 2.497c.317.158.69.158 1.006 0z'
          />
        </svg>
        &nbsp;Map
      </button>
      <button className={page === 'data' ? active : inactive} onClick={() => togglePage('data')}>
        <svg
          xmlns='http://www.w3.org/2000/svg'
          fill='none'
          viewBox='0 0 24 24'
          strokeWidth={1.5}
          stroke='currentColor'
          className='w-4 h-4'
        >
          <path
            strokeLinecap='round'
            strokeLinejoin='round'
            d='M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125'
          />
        </svg>
        &nbsp;Data
      </button>
    </div>
  )
}

function InfoButtons({ nav, setNav }) {
  function toggleInfo(type) {
    if (nav.info === type) {
      setNav({ ...nav, info: null })
    } else {
      setNav({ ...nav, info: type })
    }
  }

  const inactive = 'text-gray-300 hover:bg-gray-700 hover:text-white'
  const active = 'bg-gray-900 text-white'
  return (
    <div className={nav.page === 'map' || nav.page === 'data' ? '' : 'invisible'}>
      <span className='inline-flex rounded-md shadow-sm'>
        <button
          type='button'
          className={classNames(
            nav.info === 'layers' ? active : inactive,
            'relative inline-flex items-center border border-gray-600 rounded-l-md px-2 py-1 text-sm font-medium cursor-default'
          )}
          onClick={() => toggleInfo('layers')}
        >
          Layers
        </button>
        <button
          type='button'
          className={classNames(
            nav.info === 'features' ? active : inactive,
            'relative inline-flex items-center border border-gray-600 rounded-r-md px-2 py-1 text-sm font-medium cursor-default'
          )}
          onClick={() => toggleInfo('features')}
        >
          Features
        </button>
      </span>
    </div>
  )
}
