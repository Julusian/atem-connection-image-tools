import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

let Native: typeof import('../nativeLib.cjs') | undefined
let NativeError: Error | undefined

try {
	Native = require('../nativeLib.cjs') as typeof import('../nativeLib.cjs')
} catch (e: any) {
	NativeError = e
}

export { Native, NativeError }
