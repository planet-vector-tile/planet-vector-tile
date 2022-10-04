import { Box, Paper, Stack, TextField } from '@mui/material'
import React from 'react'

export default function Location() {
  return (
    <Box sx={{ position: 'fixed', bottom: 4, left: '50%', transform: 'translate(-50%, 0)'}}>
      <Stack direction="row" spacing={1}>
        <TextField label='z' id='outlined-size-small' defaultValue='4' size='small' />
        <TextField label='x' id='outlined-size-small' defaultValue='343' size='small' />
        <TextField label='y' id='outlined-size-small' defaultValue='8435' size='small' />
        <TextField label='h' id='outlined-size-small' defaultValue='65980' size='small' />
      </Stack>
    </Box>
  )
}
