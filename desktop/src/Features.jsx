import { useEffect, useState } from 'react'
import { listClickFeatures, listenToClickFeatures } from './selection'

export default function Features({ close }) {
  const [clickFeatures, setClickFeatures] = useState(listClickFeatures())

  useEffect(() => {
    listenToClickFeatures(clickFeatures => {
      setClickFeatures(clickFeatures)
      if (clickFeatures.length === 0) {
        close()
      }
    })
  }, [])

  return (
    <section aria-label='Features' className='pb-10'>
      <h3 className='font-medium text-gray-300 px-4 py-4'>Selected Features</h3>
      <ul role='list' className='divide-y divide-gray-600'>
        {clickFeatures.map(f => (
          <Feature key={f.id} feature={f} />
        ))}
      </ul>
    </section>
  )
}

function Feature({ feature }) {
  console.log(feature)
  const vtf = feature._vectorTileFeature
  return (
    <li key={feature.id} className='p-2'>
      <dl>
        <Property key='source' k='Source' v={feature.source} />
        <Property key='sourceLayer' k='Layer' v={feature.sourceLayer} />
        <Property key='featureId' k='ID' v={feature.id} />
        <Property key='tile' k='Tile' v={`${vtf._z}/${vtf._x}/${vtf._y}`} />

        {Object.keys(feature.properties).map(k => (
          <Property key={k} k={k} v={feature.properties[k]} />
        ))}
      </dl>
    </li>
  )
}

function Property({ k, v }) {
  return (
    <div className='sm:grid sm:grid-cols-3 sm:gap-1'>
      <dt className='text-sm font-medium text-gray-500'>{k}</dt>
      <dd className='text-sm text-gray-300 sm:col-span-2 mt-1 sm:mt-0'>{v?.toString()}</dd>
    </div>
  )
}
