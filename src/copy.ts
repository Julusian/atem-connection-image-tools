/**
 * Processed buffer info
 *
 * Note: This is copied from atem-connection, and must match or extend their type
 */
export interface UploadBufferInfo {
	encodedData: Buffer
	rawDataLength: number
	isRleEncoded: boolean
	hash: string | null
}
