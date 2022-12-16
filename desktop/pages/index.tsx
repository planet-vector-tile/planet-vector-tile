import type { NextPage } from 'next'
import Features from '../components/Features'
import Nav from '../components/Nav'
import Map from '../components/Map'

const Home: NextPage = () => {
  return (
    <>
    <Nav />
    <Map>
      <Features />
    </Map>
    </>
  )
}

export default Home
