import { Accordion, AccordionDetails, AccordionSummary, Box, Typography } from '@mui/material'
import ArrowForwardIosSharpIcon from '@mui/icons-material/ArrowForwardIosSharp'
import React from 'react'

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
          <Typography>
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse malesuada lacus ex, sit amet blandit
            leo lobortis eget.
          </Typography>
        </AccordionDetails>
      </Accordion>
    </Box>
  )
}
