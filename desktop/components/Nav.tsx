import { Fragment } from 'react';
import { Disclosure, Menu, Transition } from '@headlessui/react';
import { Bars3Icon, BellIcon, XMarkIcon } from '@heroicons/react/24/outline';

const navigation = [
    { name: 'Planets', href: '#', current: true },
    { name: 'Map', href: '#', current: false },
    { name: 'Data', href: '#', current: false },
];

function classNames(...classes: string[]) {
    return classes.filter(Boolean).join(' ');
}

export default function Nav() {
    return (
        <Disclosure as='nav' className='bg-gray-800'>
            {({ open }) => (
                <>
                    <div className='mx-auto px-2 sm:px-6 lg:px-8'>
                        <div className='relative flex h-10 items-center justify-between'>
                            <div className='absolute inset-y-0 left-0 flex items-center sm:hidden'>
                                {/* Mobile menu button*/}
                                <Disclosure.Button className='inline-flex items-center justify-center rounded-md p-2 text-gray-400 hover:bg-gray-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white'>
                                    <span className='sr-only'>Open main menu</span>
                                    {open ? (
                                        <XMarkIcon className='block h-6 w-6' aria-hidden='true' />
                                    ) : (
                                        <Bars3Icon className='block h-6 w-6' aria-hidden='true' />
                                    )}
                                </Disclosure.Button>
                            </div>
                            <div className='flex flex-1 items-center justify-center sm:items-stretch sm:justify-start'>
                                <div className='flex flex-shrink-0 items-center'>
                                    <svg className='w-8 h-8' strokeWidth='1.5' viewBox='0 0 24 24' fill='none'>
                                        <circle cx='12' cy='12' r='8' stroke='white' stroke-width='1.5'></circle>
                                        <path
                                            d='M17.5 6.348c2.297-.538 3.945-.476 4.338.312.73 1.466-3.158 4.89-8.687 7.645-5.528 2.757-10.602 3.802-11.333 2.336-.392-.786.544-2.134 2.349-3.64'
                                            stroke='white'
                                        ></path>
                                    </svg>
                                    <span className='text-white px-2 text-xl'>PlanetVectorTile</span>
                                </div>
                            </div>
                            <div className='relative z-0 flex flex-1 items-center justify-center px-2 sm:absolute sm:inset-0'>
                                <div className='hidden sm:ml-6 sm:block'>
                                    <div className='flex space-x-4'>
                                        {navigation.map(item => (
                                            <a
                                                key={item.name}
                                                href={item.href}
                                                className={classNames(
                                                    item.current
                                                        ? 'bg-gray-900 text-white'
                                                        : 'text-gray-300 hover:bg-gray-700 hover:text-white',
                                                    'px-2 py-1 rounded-md text-sm font-medium'
                                                )}
                                                aria-current={item.current ? 'page' : undefined}
                                            >
                                                {item.name}
                                            </a>
                                        ))}
                                    </div>
                                </div>
                            </div>
                            <div className='absolute inset-y-0 right-0 flex items-center pr-2 sm:static sm:inset-auto sm:ml-6 sm:pr-0'>
                                <div className='sm:ml-6 sm:block'>
                                    <span className='isolate inline-flex rounded-md shadow-sm'>
                                        <button
                                            type='button'
                                            className='relative inline-flex items-center border border-gray-700 rounded-l-md px-2 py-1 text-sm font-medium bg-gray-900 text-white'
                                        >
                                            Layers
                                        </button>
                                        <button
                                            type='button'
                                            className='relative -ml-px inline-flex items-center border border-gray-700 rounded-r-md px-2 py-1 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white'
                                        >
                                            Features
                                        </button>
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <Disclosure.Panel className='sm:hidden'>
                        <div className='space-y-1 px-2 pt-2 pb-3'>
                            {navigation.map(item => (
                                <Disclosure.Button
                                    key={item.name}
                                    as='a'
                                    href={item.href}
                                    className={classNames(
                                        item.current
                                            ? 'bg-gray-900 text-white'
                                            : 'text-gray-300 hover:bg-gray-700 hover:text-white',
                                        'block px-3 py-2 rounded-md text-base font-medium'
                                    )}
                                    aria-current={item.current ? 'page' : undefined}
                                >
                                    {item.name}
                                </Disclosure.Button>
                            ))}
                        </div>
                    </Disclosure.Panel>
                </>
            )}
        </Disclosure>
    );
}
