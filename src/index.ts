import { Native } from './nativeLoader.js'
import type { UploadBufferInfo } from './copy.js'

export { NativeError } from './nativeLoader.js'

function getNative(): NonNullable<typeof Native> {
	if (!Native) throw new Error('Library failed to initialise')
	return Native
}

export interface EncodingOptions {
	disableRLE?: boolean
}

function checkEncodeArgs(width: number, height: number, data: Buffer, format: 'rgba'): void {
	const expectedLength = width * height * 4
	if (data.length !== expectedLength)
		throw new Error(`Pixel buffer has incorrect length. Received ${data.length} expected ${expectedLength}`)

	if (format !== 'rgba') throw new Error(`Unsupported input format "${format}"`)
}

function wrapEncodedData(encodedData: Buffer, hash: string): UploadBufferInfo {
	return {
		encodedData: encodedData, // TODO: RLE
		rawDataLength: encodedData.length,
		isRleEncoded: false, // TODO: RLE
		hash,
	}
}

/**
 * Encode an image for uploading to the ATEM
 *
 * This is performed synchronously
 *
 * @param width - The width of the image
 * @param height - The height of the image
 * @param data - The input pixel data
 * @param format - The format of the input pixel data
 */
export function encodeImageForAtem(
	width: number,
	height: number,
	data: Buffer,
	format: 'rgba',
	_options?: EncodingOptions
): UploadBufferInfo {
	const native = getNative()
	checkEncodeArgs(width, height, data, format)

	const encodedData = Buffer.allocUnsafe(width * height * 4)
	native.convertRgbaToYuva422(width, height, data, encodedData)

	return wrapEncodedData(encodedData, native.hashBuffer(encodedData))
}

/**
 * Encode an image for uploading to the ATEM
 *
 * This is performed on the libuv threadpool, to avoid blocking the event loop.
 * The input buffer must not be modified until the returned promise has settled
 *
 * @param width - The width of the image
 * @param height - The height of the image
 * @param data - The input pixel data
 * @param format - The format of the input pixel data
 */
export async function encodeImageForAtemAsync(
	width: number,
	height: number,
	data: Buffer,
	format: 'rgba',
	_options?: EncodingOptions
): Promise<UploadBufferInfo> {
	const native = getNative()
	checkEncodeArgs(width, height, data, format)

	const encodedData = Buffer.allocUnsafe(width * height * 4)
	const hash = await native.convertRgbaToYuva422Async(width, height, data, encodedData)

	return wrapEncodedData(encodedData, hash)
}

/**
 * Decode an image downloaded from the ATEM to RGBA
 *
 * This is performed synchronously
 *
 * @param width - The width of the image
 * @param height - The height of the image
 * @param data - The YUVA422 pixel data, optionally RLE encoded
 * @returns The RGBA pixel data
 */
export function decodeImageFromAtem(width: number, height: number, data: Buffer): Buffer {
	const native = getNative()

	// RLE means the input data could have any length, so that gets checked while decoding
	const output = Buffer.allocUnsafe(width * height * 4)
	native.convertYuva422ToRgba(width, height, data, output)
	return output
}

/**
 * Decode an image downloaded from the ATEM to RGBA
 *
 * This is performed on the libuv threadpool, to avoid blocking the event loop.
 * The input buffer must not be modified until the returned promise has settled
 *
 * @param width - The width of the image
 * @param height - The height of the image
 * @param data - The YUVA422 pixel data, optionally RLE encoded
 * @returns The RGBA pixel data
 */
export async function decodeImageFromAtemAsync(width: number, height: number, data: Buffer): Promise<Buffer> {
	const native = getNative()

	// RLE means the input data could have any length, so that gets checked while decoding
	const output = Buffer.allocUnsafe(width * height * 4)
	await native.convertYuva422ToRgbaAsync(width, height, data, output)
	return output
}
