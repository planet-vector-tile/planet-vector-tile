import { createTheme } from '@mui/material/styles'
import { red } from '@mui/material/colors'

// A custom theme for this app
const theme = createTheme({
  spacing: 4,
  palette: {
    primary: {
      main: '#556cd6',
    },
    secondary: {
      main: '#19857b',
    },
    error: {
      main: red.A400,
    },
  },
  components: {
    MuiPaper: {
      defaultProps: {
        elevation: 0,
      },
      styleOverrides: {
        root: {
          backgroundColor: 'rgba(255,255,255,0.8)',
        },
      },
    },
    MuiAccordion: {
      styleOverrides: {
        root: {
          border: `1px solid rgba(0, 0, 0, .125)`,
          '&:not(:last-child)': {
            borderBottom: 0,
          },
          '&:before': {
            display: 'none',
          },
        }
      }
    },
    MuiAccordionSummary: {
      styleOverrides: {
        root: {
          flexDirection: 'row-reverse',
          minHeight: 36,
          '& .MuiAccordionSummary-expandIconWrapper.Mui-expanded': {
            transform: 'rotate(90deg)',
          },
        },
        content: {
          margin: 0,
        },
      },
    },
    // MuiAccordionDetails: {
    //   styleOverrides: {
    //     root: {
    //       borderTop: '1px solid rgba(0, 0, 0, .125)',
    //     }
    //   }
    // }
  },
})

export default theme