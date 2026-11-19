let Native: typeof import('../nativeLib') | undefined
let NativeError: Error | undefined

try {
	// eslint-disable-next-line @typescript-eslint/no-require-imports
	Native = require('../nativeLib') as typeof import('../nativeLib')
} catch (e: any) {
	NativeError = e
}

export { Native, NativeError }
