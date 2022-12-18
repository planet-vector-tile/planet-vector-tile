const files = [
  {
    title: 'OpenStreetMap',
    size: '72.8 GiB',
    source:
      'https://images.unsplash.com/photo-1614732414444-096e5f1122d5?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=1974&q=80',
  },
  {
    title: 'Red Cross Uganda',
    size: '21.7 GiB',
    source:
      'https://images.unsplash.com/photo-1614728894747-a83421e2b9c9?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=1974&q=80',
  },
  {
    title: 'MSF Lumbumbashi',
    size: '898.2 MiB',
    source:
      'https://images.unsplash.com/photo-1614313913007-2b4ae8ce32d6?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=1974&q=80',
  },
  {
    title: 'Active Wild Fires 2023',
    size: '9.8 GiB',
    source:
      'https://images.unsplash.com/photo-1627741145472-53cffba5d9b3?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=2070&q=80',
  },
  {
    title: 'Curated Daylight OSM',
    size: '82.8 GiB',
    source:
      'https://images.unsplash.com/photo-1614732484003-ef9881555dc3?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=1974&q=80',
  },
  {
    title: 'Europa',
    size: '12.8 GiB',
    source: 'https://upload.wikimedia.org/wikipedia/commons/e/e4/Europa-moon-with-margins.jpg',
  },
]

export default function Planets({ nav, setNav }) {
  if (nav.page !== 'planets') return null
  return (
    <div className='fixed top-0 w-full h-full bg-slate-800 px-8 pt-20'>
      <Thumbnails />
    </div>
  )
}

function Thumbnails() {
  return (
    <ul role='list' className='grid grid-cols-2 gap-x-4 gap-y-8 sm:grid-cols-3 sm:gap-x-6 lg:grid-cols-4 xl:gap-x-8'>
      {files.map(file => (
        <li key={file.source} className='relative'>
          <div className='group aspect-w-10 aspect-h-7 block w-full overflow-hidden rounded-lg bg-gray-100 focus-within:ring-2 focus-within:ring-indigo-500 focus-within:ring-offset-2 focus-within:ring-offset-gray-100'>
            <img src={file.source} alt='' className='pointer-events-none object-cover group-hover:opacity-75' />
            <button type='button' className='absolute inset-0 focus:outline-none'>
              <span className='sr-only'>View details for {file.title}</span>
            </button>
          </div>
          <p className='pointer-events-none mt-2 block truncate text-md font-medium text-gray-100'>{file.title}</p>
          <p className='pointer-events-none block text-sm font-medium text-gray-300'>{file.size}</p>
        </li>
      ))}
    </ul>
  )
}
