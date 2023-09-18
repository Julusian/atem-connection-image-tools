let Native: typeof import('../nativeLib') | undefined
let NativeError: Error | undefined

try {
	Native = require('../nativeLib')
} catch (e: any) {
	NativeError = e
}

export { Native, NativeError }
