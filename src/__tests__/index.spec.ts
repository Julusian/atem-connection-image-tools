import { describe, expect, test } from 'vitest'

import {
	decodeImageFromAtem,
	decodeImageFromAtemAsync,
	encodeImageForAtem,
	encodeImageForAtemAsync,
	NativeError,
} from '../index.js'
import type { UploadBufferInfo } from '../copy.js'

function createTestImage(width: number, height: number): Buffer {
	const input = Buffer.alloc(width * height * 4)
	for (let i = 0; i < input.length; i++) {
		input[i] = (i * 37) & 0xff
	}
	return input
}

/** Strip the image data, as `toEqual` compares buffers byte by byte which is very slow for full size images */
function withoutData(info: UploadBufferInfo): Omit<UploadBufferInfo, 'encodedData'> {
	const { encodedData: _encodedData, ...rest } = info
	return rest
}

function createFlatImage(width: number, height: number): Buffer {
	const input = Buffer.alloc(width * height * 4)
	for (let i = 0; i < width * height; i++) {
		input.writeUInt32BE(0x7f7f7fff, i * 4) // mid grey, opaque
	}
	return input
}

describe('native', () => {
	test('loads', () => {
		expect(NativeError).toBeUndefined()
	})

	test('encode and decode roundtrip', () => {
		const width = 8
		const height = 2
		const input = createFlatImage(width, height)

		const encoded = encodeImageForAtem(width, height, input, 'rgba')
		expect(encoded.encodedData.length).toBe(width * height * 4)
		expect(encoded.rawDataLength).toBe(width * height * 4)
		expect(encoded.isRleEncoded).toBe(false)
		expect(encoded.hash).toBeTruthy()

		const decoded = decodeImageFromAtem(width, height, encoded.encodedData)
		expect(decoded.length).toBe(input.length)
		for (let i = 0; i < decoded.length; i++) {
			expect(Math.abs(decoded[i] - input[i])).toBeLessThanOrEqual(2)
		}
	})

	test('rejects incorrect buffer length', () => {
		expect(() => encodeImageForAtem(4, 4, Buffer.alloc(10), 'rgba')).toThrow(/incorrect length/)
	})

	test('rejects too little data to decode', () => {
		expect(() => decodeImageFromAtem(8, 2, Buffer.alloc(32))).toThrow(/less data than expected/)
	})
})

describe('hash', () => {
	test('is 16 bytes of base64', () => {
		const encoded = encodeImageForAtem(8, 2, createTestImage(8, 2), 'rgba')
		expect(Buffer.from(encoded.hash ?? '', 'base64').length).toBe(16)
	})

	test('is stable, and changes with the content', () => {
		const a = encodeImageForAtem(8, 2, createTestImage(8, 2), 'rgba')
		const b = encodeImageForAtem(8, 2, createTestImage(8, 2), 'rgba')
		const c = encodeImageForAtem(8, 2, createFlatImage(8, 2), 'rgba')
		expect(a.hash).toBe(b.hash)
		expect(a.hash).not.toBe(c.hash)
	})
})

describe('async', () => {
	// Large enough to use BT.709
	const width = 1280
	const height = 720

	test('encode matches sync', async () => {
		const input = createTestImage(width, height)

		const syncResult = encodeImageForAtem(width, height, input, 'rgba')
		const asyncResult = await encodeImageForAtemAsync(width, height, input, 'rgba')

		expect(withoutData(asyncResult)).toEqual(withoutData(syncResult))
		expect(asyncResult.encodedData.equals(syncResult.encodedData)).toBe(true)
	})

	test('decode matches sync', async () => {
		const encoded = encodeImageForAtem(width, height, createTestImage(width, height), 'rgba').encodedData

		const syncResult = decodeImageFromAtem(width, height, encoded)
		const asyncResult = await decodeImageFromAtemAsync(width, height, encoded)

		expect(asyncResult.equals(syncResult)).toBe(true)
	})

	test('encode rejects incorrect buffer length', async () => {
		await expect(encodeImageForAtemAsync(4, 4, Buffer.alloc(10), 'rgba')).rejects.toThrow(/incorrect length/)
	})

	test('decode rejects invalid width', async () => {
		await expect(decodeImageFromAtemAsync(4, 4, Buffer.alloc(64))).rejects.toThrow(/multiple of 8/)
	})

	test('decode rejects too little data', async () => {
		await expect(decodeImageFromAtemAsync(8, 2, Buffer.alloc(32))).rejects.toThrow(/less data than expected/)
	})

	test('runs concurrently', async () => {
		const input = createTestImage(width, height)
		const expected = encodeImageForAtem(width, height, input, 'rgba')

		const results = await Promise.all(
			Array.from({ length: 8 }, async () => encodeImageForAtemAsync(width, height, input, 'rgba'))
		)
		for (const result of results) {
			expect(withoutData(result)).toEqual(withoutData(expected))
			expect(result.encodedData.equals(expected.encodedData)).toBe(true)
		}
	})
})
