import { Native } from './nativeLoader'
import * as Js from './jsLib'

export { NativeError } from './nativeLoader'

/**
 * Convert a RGBA buffer to ATEM YUV422 packing in the correct colorspace
 *
 * This is performed synchronously
 *
 * @param width - The width of the image
 * @param height - The height of the image
 * @param data - The input RGBA pixel data
 * @returns The output YUVA422 pixel data
 */
export function convertRGBAToYUV422(width: number, height: number, data: Buffer): Buffer {
	if (Native) {
		const output = Buffer.alloc(width * height * 4)
		Native.convertRgbaToYuva422(width, height, data, output)
		return output
	} else {
		return Js.convertRGBAToYUV422(width, height, data)
	}
}
