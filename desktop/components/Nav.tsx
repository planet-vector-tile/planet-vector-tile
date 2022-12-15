/*
  This example requires some changes to your config:
  
  ```
  // tailwind.config.js
  module.exports = {
    // ...
    plugins: [
      // ...
      require('@tailwindcss/forms'),
    ],
  }
  ```
*/
const tabs = [
    { name: 'My Account', href: '#', current: false },
    { name: 'Company', href: '#', current: false },
    { name: 'Team Members', href: '#', current: true },
    { name: 'Billing', href: '#', current: false },
];

function classNames(...classes: string[]) {
    return classes.filter(Boolean).join(' ');
}

export default function Nav() {
    return (
        <div>
            <div className='sm:hidden'>
                <label htmlFor='tabs' className='sr-only'>
                    Select a tab
                </label>
                {/* Use an "onChange" listener to redirect the user to the selected tab URL. */}
                <select
                    id='tabs'
                    name='tabs'
                    className='block w-full rounded-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500'
                    defaultValue={tabs.find(tab => tab.current).name}
                >
                    {tabs.map(tab => (
                        <option key={tab.name}>{tab.name}</option>
                    ))}
                </select>
            </div>
            <div className='hidden sm:block'>
                <nav className='flex space-x-4' aria-label='Tabs'>
                    {tabs.map(tab => (
                        <a
                            key={tab.name}
                            href={tab.href}
                            className={classNames(
                                tab.current ? 'bg-indigo-100 text-indigo-700' : 'text-gray-500 hover:text-gray-700',
                                'px-3 py-2 font-medium text-sm rounded-md'
                            )}
                            aria-current={tab.current ? 'page' : undefined}
                        >
                            {tab.name}
                        </a>
                    ))}
                </nav>
            </div>
        </div>
    );
}
