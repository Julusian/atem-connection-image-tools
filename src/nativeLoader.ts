// This is a static import so that bundlers (eg esbuild) can follow it.
// nativeLib.cjs reports load failures instead of throwing, so the native library remains optional
// eslint-disable-next-line n/no-missing-import -- the rule looks for nativeLib.cts, but this is a plain .cjs file
import nativeLib from '../nativeLib.cjs'

type NativeBinding = typeof import('../nativeLib.cjs')

const loaded = nativeLib as unknown as { binding: NativeBinding | undefined; error: Error | undefined }

const Native: NativeBinding | undefined = loaded.binding
const NativeError: Error | undefined = loaded.error

export { Native, NativeError }
