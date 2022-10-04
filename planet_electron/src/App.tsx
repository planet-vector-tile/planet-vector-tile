import { Stack } from '@mui/material'
import * as React from 'react'
import LeftMenu from './LeftMenu'
import Location from './Location'
import RightMenu from './RightMenu'


export default function App() {
  return (
    <Stack sx={{padding: 1}} direction='row' justifyContent='space-between' alignItems='flex-start' spacing={1}>
      <LeftMenu />
      <Location />
      <RightMenu />
    </Stack>
  )
}
