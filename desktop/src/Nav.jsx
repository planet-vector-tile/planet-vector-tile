import { Fragment } from 'react'
import { Disclosure, Menu, Transition } from '@headlessui/react'
import { Bars3Icon, BellIcon, XMarkIcon } from '@heroicons/react/24/outline'

function classNames(...classes) {
  return classes.filter(Boolean).join(' ')
}

export default function Nav() {
  return (
    <Disclosure as='nav' className='bg-gray-800/90 border-b border-gray-900 backdrop-blur-md'>
      {({ open }) => (
        <>
          <div className='mx-auto px-20'>
            <div className='flex h-10 items-center justify-between'>
              <div className='flex items-center'>
                <div className='flex-shrink-0'>
                  <svg className='w-8 h-8' strokeWidth='1.5' viewBox='0 0 24 24' fill='none'>
                    <circle cx='12' cy='12' r='8' stroke='white' stroke-width='1.5'></circle>
                    <path
                      d='M17.5 6.348c2.297-.538 3.945-.476 4.338.312.73 1.466-3.158 4.89-8.687 7.645-5.528 2.757-10.602 3.802-11.333 2.336-.392-.786.544-2.134 2.349-3.64'
                      stroke='white'
                    ></path>
                  </svg>
                </div>
                <span className='text-white px-2 text-xl'>PlanetVectorTile</span>
              </div>

              <PlanetsMapData />
              <LayersFeatures />

              <div className='mr-2 flex sm:hidden'>
                {/* Mobile menu button */}
                <Disclosure.Button className='inline-flex items-center justify-center rounded-md p-2 text-gray-400 hover:bg-gray-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white'>
                  <span className='sr-only'>Open main menu</span>
                  {open ? (
                    <XMarkIcon className='block h-6 w-6' aria-hidden='true' />
                  ) : (
                    <Bars3Icon className='block h-6 w-6' aria-hidden='true' />
                  )}
                </Disclosure.Button>
              </div>
            </div>
          </div>

          <Disclosure.Panel className='sm:hidden'>
            <div className='space-y-1 px-2 pt-2 pb-3'>
              {/* Current: "bg-gray-900 text-white", Default: "text-gray-300 hover:bg-gray-700 hover:text-white" */}
              <Disclosure.Button
                as='a'
                href='#'
                className='block rounded-md bg-gray-900 px-3 py-2 text-base font-medium text-white'
              >
                Dashboard
              </Disclosure.Button>
              <Disclosure.Button
                as='a'
                href='#'
                className='block rounded-md px-3 py-2 text-base font-medium text-gray-300 hover:bg-gray-700 hover:text-white'
              >
                Disclosure Button
              </Disclosure.Button>
              <Disclosure.Button
                as='a'
                href='#'
                className='block rounded-md px-3 py-2 text-base font-medium text-gray-300 hover:bg-gray-700 hover:text-white'
              >
                Projects
              </Disclosure.Button>
              <Disclosure.Button
                as='a'
                href='#'
                className='block rounded-md px-3 py-2 text-base font-medium text-gray-300 hover:bg-gray-700 hover:text-white'
              >
                Calendar
              </Disclosure.Button>
            </div>
            <div className='border-t border-gray-700 pt-4 pb-3'>
              <div className='flex items-center px-5'>
                <div className='flex-shrink-0'>
                  <img
                    className='h-10 w-10 rounded-full'
                    src='https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80'
                    alt=''
                  />
                </div>
                <div className='ml-3'>
                  <div className='text-base font-medium text-white'>Tom Cook</div>
                  <div className='text-sm font-medium text-gray-400'>tom@example.com</div>
                </div>
                <button
                  type='button'
                  className='ml-auto flex-shrink-0 rounded-full bg-gray-800 p-1 text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800'
                >
                  <span className='sr-only'>View notifications</span>
                  <BellIcon className='h-6 w-6' aria-hidden='true' />
                </button>
              </div>
              <div className='mt-3 space-y-1 px-2'>
                <Disclosure.Button
                  as='a'
                  href='#'
                  className='block rounded-md px-3 py-2 text-base font-medium text-gray-400 hover:bg-gray-700 hover:text-white'
                >
                  Your Profile
                </Disclosure.Button>
                <Disclosure.Button
                  as='a'
                  href='#'
                  className='block rounded-md px-3 py-2 text-base font-medium text-gray-400 hover:bg-gray-700 hover:text-white'
                >
                  Settings
                </Disclosure.Button>
                <Disclosure.Button
                  as='a'
                  href='#'
                  className='block rounded-md px-3 py-2 text-base font-medium text-gray-400 hover:bg-gray-700 hover:text-white'
                >
                  Sign out
                </Disclosure.Button>
              </div>
            </div>
          </Disclosure.Panel>
        </>
      )}
    </Disclosure>
  )
}

function PlanetsMapData() {
  return (
    <div className='hidden sm:block'>
      <div className='flex space-x-4'>
        <a
          href='#'
          className='rounded-md px-3 py-1 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white'
        >
          Planets
        </a>
        <a href='#' className='rounded-md bg-gray-900 px-3 py-1 text-sm font-medium text-white'>
          Map
        </a>
        <a
          href='#'
          className='rounded-md px-3 py-1 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white'
        >
          Data
        </a>
      </div>
    </div>
  )
}

function LayersFeatures() {
  return (
    <div className='hidden sm:ml-6 sm:block'>
      <span className='inline-flex rounded-md shadow-sm'>
        <button
          type='button'
          className='relative inline-flex items-center border border-gray-600 rounded-l-md px-2 py-1 text-sm font-medium bg-gray-900 text-white'
        >
          Layers
        </button>
        <button
          type='button'
          className='relative inline-flex items-center border border-gray-600 rounded-r-md px-2 py-1 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white'
        >
          Features
        </button>
      </span>
    </div>
  )
}
