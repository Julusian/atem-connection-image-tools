/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

/**
 * Convert a RGBA buffer to ATEM YUV422 packing in the correct colorspace
 *
 * This is performed synchronously
 *
 * @param width - The width of the image
 * @param height - The height of the image
 * @param input - The input RGBA pixel data
 * @param output - The output YUVA422 pixel data
 */
export function convertRgbaToYuva422(width: number, height: number, input: Buffer, output: Buffer): void