import { Fragment, useState } from 'react'
import { Dialog, Transition } from '@headlessui/react'
import {
  Bars3Icon,
  CalendarIcon,
  ChartBarIcon,
  FolderIcon,
  HomeIcon,
  InboxIcon,
  UsersIcon,
  XMarkIcon,
} from '@heroicons/react/24/outline'

import Layers from './Layers'

const navigation = [
  { name: 'Dashboard', href: '#', icon: HomeIcon, current: true },
  { name: 'Team', href: '#', icon: UsersIcon, current: false },
  { name: 'Projects', href: '#', icon: FolderIcon, current: false },
  { name: 'Calendar', href: '#', icon: CalendarIcon, current: false },
  { name: 'Documents', href: '#', icon: InboxIcon, current: false },
  { name: 'Reports', href: '#', icon: ChartBarIcon, current: false },
]

function classNames(...classes) {
  return classes.filter(Boolean).join(' ')
}

export default function Info({ nav }) {
  if (!nav.info || nav.info === 'none' || nav.page === 'planets') {
    return null
  }
  return (
    <div className='fixed right-0 w-80 h-full bg-slate-700/80 border-l border-gray-900 backdrop-blur-md'>
      <Close onClose={() => nav.setInfo('none')} />
      <Layers />
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
