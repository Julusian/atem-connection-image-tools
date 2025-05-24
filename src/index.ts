import * as crypto from 'crypto'
import { UploadBufferInfo } from './copy'
import { Native } from './nativeLoader'

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
function convertRGBAToYUV422(width: number, height: number, data: Buffer): Buffer {
	if (!Native) throw new Error('Library failed to initialise')

	const output = Buffer.alloc(width * height * 4)
	Native.convertRgbaToYuva422(width, height, data, output)
	return output
}

/**
 * Convert an ATEM YUVA422 buffer to RGBA from the correct colorspace
 *
 * This is performed synchronously
 *
 * @param width - The width of the image
 * @param height - The height of the image
 * @param data - The input YUVA422 pixel data
 * @returns The output RGBA pixel data
 */
function convertYUV422ToRGBA(width: number, height: number, data: Buffer): Buffer {
	if (!Native) throw new Error('Library failed to initialise')

	const output = Buffer.alloc(width * height * 4)
	Native.convertYuva422ToRgba(width, height, data, output)
	return output
}

function generateHashForBuffer(data: Buffer): string {
	return data ? crypto.createHash('md5').update(data).digest('base64') : ''
}

export interface EncodingOptions {
	disableRLE?: boolean
}

export function encodeImageForAtem(
	width: number,
	height: number,
	data: Buffer,
	format: 'rgba',
	_options?: EncodingOptions
): UploadBufferInfo {
	const expectedLength = width * height * 4
	if (data.length !== expectedLength)
		throw new Error(`Pixel buffer has incorrect length. Received ${data.length} expected ${expectedLength}`)

	let encodedData: Buffer
	switch (format) {
		case 'rgba':
			encodedData = convertRGBAToYUV422(width, height, data)
			break
		default:
			throw new Error(`Unsupported input format "${format}"`)
	}

	return {
		encodedData: encodedData, // TODO: RLE
		rawDataLength: encodedData.length,
		isRleEncoded: false, // TODO: RLE
		hash: generateHashForBuffer(encodedData),
	}
}

export function decodeImageFromAtem(width: number, height: number, data: Buffer): Buffer {
	// RLE means the input data could have any length, we can't check that here

	return convertYUV422ToRGBA(width, height, data)
}
