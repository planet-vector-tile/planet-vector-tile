import { useState } from 'react'
import './App.css'
import Nav from './Nav'
import Info from './Info'
import Planets from './Planets'

function App() {
  const [nav, setNav] = useState({
    page: 'map',
    info: 'none',
  })
  return (
    <>
      <Planets nav={nav} setNav={setNav} />
      <Nav nav={nav} setNav={setNav} />
      <Info nav={nav} />
    </>
  )
}

export default App
