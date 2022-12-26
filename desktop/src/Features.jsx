import { useEffect, useState } from 'react'
import { listenToClickFeatures } from './selection'

const directory = {
  Water: [
    {
      id: 1,
      name: 'Pacific Avenue',
      role: 'Tertiary Highway',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 2,
      name: 'Lake Michigan',
      role: 'natural_water',
      imageUrl: '/relation_solid.svg',
    },
    {
      id: 3,
      name: 'Bellingham, Washington',
      role: 'admin_level = 8',
      imageUrl: '/relation_solid.svg',
    },
    {
      id: 4,
      name: 'Port-aux-Français',
      role: 'Îles Kerguelen',
      imageUrl: '/node_solid.svg',
    },
  ],
  Buildings: [
    {
      id: 5,
      name: 'Great Pyramid of Giza',
      role: 'tourism: attraction',
      imageUrl: '/way_solid_area.svg',
    },
    {
      id: 6,
      name: 'Arc de Triomphe',
      role: 'Tag',
      imageUrl: '/way_solid_area.svg',
    },
    {
      id: 7,
      name: 'The White House',
      role: 'building: government',
      imageUrl: '/way_solid_area.svg',
    },
  ],
  Roads: [
    {
      id: 8,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_area.svg',
    },
    {
      id: 9,
      name: 'Name',
      role: 'Role',
      imageUrl: '/relation_solid.svg',
    },
  ],
  Places: [
    {
      id: 10,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 11,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 12,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 13,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
  ],
  Landcover: [
    {
      id: 14,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 15,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
  ],
  'Primary Highway': [
    {
      id: 16,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 17,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 18,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
  ],
  'Secondary Highway': [
    {
      id: 19,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 20,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
  ],
  Address: [
    {
      id: 21,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 22,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 23,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
  ],
  USGS: [
    {
      id: 24,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
    {
      id: 25,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
  ],
  'Layer Name': [
    {
      id: 26,
      name: 'Name',
      role: 'Role',
      imageUrl: '/way_solid_open.svg',
    },
  ],
}

export default function Features() {
  const [clickFeatures, setClickFeatures] = useState([])
  console.log('clickFeatures', clickFeatures)

  useEffect(() => listenToClickFeatures(clickFeatures => setClickFeatures(clickFeatures)), [])

  return (
    <nav aria-label='Features'>
      {Object.keys(directory).map(letter => (
        <div key={letter} className='relative'>
          <div className='sticky top-0 z-10 border-t border-b border-gray-600 bg-gray-700 px-3 py-1 text-sm font-medium text-gray-400'>
            <h3>{letter}</h3>
          </div>
          <ul role='list' className='relative z-0 divide-y divide-gray-600'>
            {directory[letter].map(person => (
              <li key={person.id}>
                <div className='relative flex items-center space-x-3 px-3 py-3 focus-within:ring-2 focus-within:ring-inset focus-within:ring-fuchsia-700 hover:bg-slate-600/50'>
                  <div className='flex-shrink-0'>
                    <img className='h-10 w-10' src={person.imageUrl} alt='' />
                  </div>
                  <div className='min-w-0 flex-1'>
                    <a href='#' className='focus:outline-none'>
                      {/* Extend touch target to entire panel */}
                      <span className='absolute inset-0' aria-hidden='true' />
                      <p className='text-sm font-light text-gray-200'>{person.name}</p>
                      <p className='truncate text-sm font-light text-gray-400'>{person.role}</p>
                    </a>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        </div>
      ))}
      <div className='h-10' />
    </nav>
  )
}
