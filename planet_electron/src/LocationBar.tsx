import { Paper, Stack, Typography } from '@mui/material'
import React, { useCallback, useEffect, useState } from 'react'
import map from './map'

export default class LocationBar extends React.Component {

  componentDidMount() {
    map().on('moveend', () => {
      this.forceUpdate()
    })
  }

  render() {
    return (
      <Paper square sx={{ position: 'fixed', bottom: 0, left: -4, right: 0, overflowX: 'auto', whiteSpace: 'nowrap', }}>
        <Stack direction='row' justifyContent='space-around' alignItems='center'>
        <Typography variant='caption' sx={{lineHeight: 1}}>
          </Typography>
          <Typography variant='caption' sx={{lineHeight: 1}}>
            <b>BBox</b> <BBox />
          </Typography>
          <Typography variant='caption' sx={{lineHeight: 1}}>
            <b>LonLat</b> <LonLat />
          </Typography>
          <Typography variant='caption' sx={{lineHeight: 1}}>
            <b>Hilbert</b> <Hilbert />
          </Typography>
          <Typography variant='caption' sx={{lineHeight: 1}}>
            <b>Tile</b> <Tile />
          </Typography>
        </Stack>
      </Paper>
    )
  }
}

const editableStyle = {border: "1px solid transparent", '&:hover': { border: "1px solid #f4d90e", borderRadius: 1}}

function BBox() {
  const bounds = map().getBounds()
  const str = bounds.toArray().flat().map(c => parseFloat(c.toFixed(7))).join(', ')
  return (
    <Typography variant='caption' sx={editableStyle}>
      {str}
    </Typography>
  )
}

function LonLat() {
  const center = map().getCenter()
  const str = center.toArray().map(c => parseFloat(c.toFixed(7))).join(', ')
  return (
    <Typography variant='caption' sx={editableStyle}>
      {str}
    </Typography>
  )
}

function Hilbert() {
  return (
    <Typography variant='caption' sx={editableStyle}>
      543665
    </Typography>
  )
}

function Tile() {
  return (
    <Typography variant='caption' sx={editableStyle}>
      z:4 x:253 y:654 h:624
    </Typography>
  )
}