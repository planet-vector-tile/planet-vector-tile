import { useState } from 'react'
import './App.css'
import Nav from './Nav'
import Info from './Info'

function App() {
  const [nav, setNav] = useState({
    page: 'map',
    info: 'none',
  })
  return (
    <>
      <Nav nav={nav} setNav={setNav} />
      <Info nav={nav} />
    </>
  )
}

export default App
