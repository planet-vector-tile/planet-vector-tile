import { Accordion, AccordionDetails, AccordionSummary, Box, Slider, Typography } from '@mui/material'
import ArrowForwardIosSharpIcon from '@mui/icons-material/ArrowForwardIosSharp'
import React, { useState } from 'react'
import map from './map'

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

function BackgroundOpacity() {
    const [opacity, setOpacity] = useState<number>(50)

    function handleSliderEvent(event: Event, newValue: number | number[]) {
        setOpacity(newValue as number)
        // map.setPaintProperty('sat', 'raster-opacity')
    }

    return (
        <Slider aria-label="Volume" value={opacity} onChange={handleSliderEvent} />
    )
}
