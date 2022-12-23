import { useState } from 'react'
import './App.css'
import Nav from './Nav'
import InfoPanel from './Info'
import Planets from './Planets'
import Loc from './Loc'
import store from './store'

function App() {
  const [nav, _setNav] = useState(store.nav)

  function setNav(nav) {
    store.nav = nav
    _setNav(nav)
  }

  return (
    <>
      <Planets nav={nav} setNav={setNav} />
      <Nav nav={nav} setNav={setNav} />
      <InfoPanel nav={nav} setNav={setNav} />
      <Loc />
    </>
  )
}

export default App
