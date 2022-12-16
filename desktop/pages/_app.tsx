import '../styles/globals.css'
import type { AppProps } from 'next/app'

import 'maplibre-gl/dist/maplibre-gl.css'

function MyApp({ Component, pageProps }: AppProps) {
  return <Component {...pageProps} />
}

export default MyApp
