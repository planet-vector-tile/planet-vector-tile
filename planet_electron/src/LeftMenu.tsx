import { Accordion, AccordionDetails, AccordionSummary, Box, Slider, Typography } from '@mui/material'
import ArrowForwardIosSharpIcon from '@mui/icons-material/ArrowForwardIosSharp'
import React, { useState } from 'react'
import mapfn from './map'

const map = mapfn()

export default function LeftMenu() {
  return (
    <Box sx={{ maxWidth: 275 }}>
      <Accordion disableGutters>
        <AccordionSummary
          expandIcon={<ArrowForwardIosSharpIcon sx={{ fontSize: '0.9rem' }} />}
          aria-controls='panel1a-content'
          id='panel1a-header'
        >
          <Typography variant='subtitle2' sx={{ ml: 1, fontWeight: 'bold' }}>
            daylight-planet-with-admin.pvt
          </Typography>
        </AccordionSummary>
        <AccordionDetails>
          <BackgroundOpacity />
        </AccordionDetails>
      </Accordion>
    </Box>
  )
}

const initialOpacity = (map.getLayer('sat').getPaintProperty('raster-opacity') as number) || 0.75

function BackgroundOpacity() {
  const [opacity, setOpacity] = useState<number>(initialOpacity)

  function handleSliderEvent(event: Event, newValue: number | number[]) {
    const o = newValue as number
    setOpacity(o)
    map.setPaintProperty('sat', 'raster-opacity', o)
  }

  return (
    <>
      <Typography variant='subtitle2'>Satellite Opacity</Typography>
      <Slider aria-label='Satellite Opacity' value={opacity} step={0.01} min={0} max={1} onChange={handleSliderEvent} />
    </>
  )
}
