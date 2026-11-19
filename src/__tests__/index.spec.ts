import { describe, expect, test } from 'vitest'

import { decodeImageFromAtem, encodeImageForAtem, NativeError } from '../index'

describe('native', () => {
	test('loads', () => {
		expect(NativeError).toBeUndefined()
	})

	test('encode and decode roundtrip', () => {
		const width = 8
		const height = 2
		const input = Buffer.alloc(width * height * 4)
		for (let i = 0; i < width * height; i++) {
			input.writeUInt32BE(0x7f7f7fff, i * 4) // mid grey, opaque
		}

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
})
