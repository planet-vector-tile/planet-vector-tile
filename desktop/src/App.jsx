import { useState } from 'react'
import './App.css'
import Nav from './Nav'
import Info from './Info'
import Planets from './Planets'
import Loc from './Loc'

import hotkeys from './hotkeys'
import initDatastyle from './datastyle'

const store = window.store

let datastyle = null

function App() {
  if (!datastyle) {
    datastyle = initDatastyle(window.map)
  }
  const [nav, _setNav] = useState(store.nav)

  function setNav(nav) {
    store.nav = nav
    _setNav(nav)
  }

  return (
    <>
      <Planets nav={nav} setNav={setNav} />
      <Nav nav={nav} setNav={setNav} />
      <Info nav={nav} setNav={setNav} datastyle={datastyle} />
      <Loc />
    </>
  )
}

export default App
