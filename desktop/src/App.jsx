import { useState } from 'react'
import './App.css'
import Nav from './Nav'
import Info from './Info'
import Planets from './Planets'
import Loc from './Loc'

let storedNav = {
  page: 'map',
  info: 'none',
}

try {
  const navStr = localStorage.getItem('nav')
  storedNav = JSON.parse(navStr)
} catch (e) {
  console.log('no stored nav state')
}

function App() {
  const [nav, _setNav] = useState(storedNav)

  function setNav(nav) {
    console.log('setNav', nav)
    try {
      localStorage.setItem('nav', JSON.stringify(nav))
    } catch (e) {
      console.error('cannot store nav state in local storage')
    }
    _setNav(nav)
  }

  return (
    <>
      <Planets nav={nav} setNav={setNav} />
      <Nav nav={nav} setNav={setNav} />
      <Info nav={nav} />
      <Loc />
    </>
  )
}

export default App
