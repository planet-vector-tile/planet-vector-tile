import { Paper, Stack, styled } from '@mui/material'
import * as React from 'react'
import LeftMenu from './LeftMenu'
import Location from './Location'
import RightMenu from './RightMenu'


const Item = styled(Paper)(({ theme }) => ({
  backgroundColor: theme.palette.mode === 'dark' ? '#1A2027' : '#fff',
  ...theme.typography.body2,
  padding: theme.spacing(1),
  textAlign: 'center',
  color: theme.palette.text.secondary,
}))

export default function App() {
  return (
    <Stack sx={{padding: 1}} direction='row' justifyContent='space-between' alignItems='flex-start' spacing={1}>
      <LeftMenu />
      <Location />
      <RightMenu />
    </Stack>
  )
}
