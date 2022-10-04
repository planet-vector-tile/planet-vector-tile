import { Stack } from '@mui/material'
import * as React from 'react'
import LeftMenu from './LeftMenu'
import LocationBar from './LocationBar'
import RightMenu from './RightMenu'


export default function App() {
  return (
    <Stack padding={1} direction='row' justifyContent='space-between' alignItems='flex-start' spacing={1}>
      <LeftMenu />
      <LocationBar />
      <RightMenu />
    </Stack>
  )
}
