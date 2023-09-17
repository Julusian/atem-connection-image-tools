/**
 * @todo: BALTE - 2018-5-24:
 * Create util functions that handle proper colour spaces in UHD.
 */
export function convertRGBAToYUV422(width: number, height: number, data: Buffer): Buffer {
	// BT.709 or BT.601
	const KR = height >= 720 ? 0.2126 : 0.299
	const KB = height >= 720 ? 0.0722 : 0.114
	const KG = 1 - KR - KB

	const KRi = 1 - KR
	const KBi = 1 - KB

	const YRange = 219 / 64
	const CbCrRange = 224 / 64
	const HalfCbCrRange = CbCrRange / 2

	const YOffset = 64
	const CbCrOffset = 512

	const KRoKBi = KR / KBi
	const KGoKBi = KG / KBi
	const KBoKRi = KB / KRi
	const KGoKRi = KG / KRi

	const AlphaScale = (219 / 255) * 4 // limited range, and 8 -> 10bit

	const genColor = (rawA: number, uv16: number, y16: number): number => {
		const a = Math.round(AlphaScale * rawA) + 64
		const y = Math.round(YOffset + y16 * YRange)
		const uv = Math.round(CbCrOffset + uv16 * HalfCbCrRange)

		return (a << 20) + (uv << 10) + y
	}

	const buffer = Buffer.alloc(width * height * 4)
	for (let i = 0; i < width * height * 4; i += 8) {
		const r1 = data[i + 0]
		const g1 = data[i + 1]
		const b1 = data[i + 2]

		const r2 = data[i + 4]
		const g2 = data[i + 5]
		const b2 = data[i + 6]

		const a1 = data[i + 3]
		const a2 = data[i + 7]

		const y16a = KR * r1 + KG * g1 + KB * b1
		const cb16 = -KRoKBi * r1 - KGoKBi * g1 + b1
		const y16b = KR * r2 + KG * g2 + KB * b2
		const cr16 = r1 - KGoKRi * g1 - KBoKRi * b1

		buffer.writeUInt32BE(genColor(a1, cb16, y16a), i)
		buffer.writeUInt32BE(genColor(a2, cr16, y16b), i + 4)
	}
	return buffer
}
