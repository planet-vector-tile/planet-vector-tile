export function classNames(...classes) {
  return classes.filter(Boolean).join(' ')
}

export function isVectorType(type) {
  return type !== 'background' && type !== 'raster' && type !== 'hillshade'
}
