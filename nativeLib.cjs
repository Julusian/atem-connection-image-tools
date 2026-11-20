const { existsSync, readFileSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

let nativeBinding = null
let loadError = null

// Workaround to fix webpack's build warnings: 'the request of a dependency is an expression'
const runtimeRequire0 = typeof __webpack_require__ === 'function' ? __non_webpack_require__ : require // eslint-disable-line

function runtimeRequire(name) {
	const candidates = [join(__dirname, name), join(__dirname, 'prebuilds', name)]
	for (const candidate of candidates) {
		if (existsSync(candidate)) return runtimeRequire0(candidate)
	}
}

function isMusl() {
	// For Node 10
	if (!process.report || typeof process.report.getReport !== 'function') {
		try {
			const lddPath = require('child_process').execSync('which ldd').toString().trim()
			return readFileSync(lddPath, 'utf8').includes('musl')
		} catch (e) {
			return true
		}
	} else {
		const { glibcVersionRuntime } = process.report.getReport().header
		return !glibcVersionRuntime
	}
}

switch (platform) {
	case 'android':
		switch (arch) {
			case 'arm64':
				try {
					nativeBinding = runtimeRequire('atem-connection-image-tools.android-arm64.node')
				} catch (e) {
					loadError = e
				}
				break
			case 'arm':
				try {
					nativeBinding = runtimeRequire('atem-connection-image-tools.android-arm-eabi.node')
				} catch (e) {
					loadError = e
				}
				break
			default:
				loadError = new Error(`Unsupported architecture on Android ${arch}`)
		}
		break
	case 'win32':
		switch (arch) {
			case 'x64':
				try {
					nativeBinding = runtimeRequire('atem-connection-image-tools.win32-x64-msvc.node')
				} catch (e) {
					loadError = e
				}
				break
			case 'ia32':
				try {
					nativeBinding = runtimeRequire('atem-connection-image-tools.win32-ia32-msvc.node')
				} catch (e) {
					loadError = e
				}
				break
			case 'arm64':
				try {
					nativeBinding = runtimeRequire('atem-connection-image-tools.win32-arm64-msvc.node')
				} catch (e) {
					loadError = e
				}
				break
			default:
				loadError = new Error(`Unsupported architecture on Windows: ${arch}`)
		}
		break
	case 'darwin':
		try {
			nativeBinding = runtimeRequire('atem-connection-image-tools.darwin-universal.node')
			if (nativeBinding) break
		} catch {}
		switch (arch) {
			case 'x64':
				try {
					nativeBinding = runtimeRequire('atem-connection-image-tools.darwin-x64.node')
				} catch (e) {
					loadError = e
				}
				break
			case 'arm64':
				try {
					nativeBinding = runtimeRequire('atem-connection-image-tools.darwin-arm64.node')
				} catch (e) {
					loadError = e
				}
				break
			default:
				loadError = new Error(`Unsupported architecture on macOS: ${arch}`)
		}
		break
	case 'freebsd':
		if (arch !== 'x64') {
			loadError = new Error(`Unsupported architecture on FreeBSD: ${arch}`)
		} else {
			try {
				nativeBinding = runtimeRequire('atem-connection-image-tools.freebsd-x64.node')
			} catch (e) {
				loadError = e
			}
		}
		break
	case 'linux':
		switch (arch) {
			case 'x64':
				if (isMusl()) {
					try {
						nativeBinding = runtimeRequire('atem-connection-image-tools.linux-x64-musl.node')
					} catch (e) {
						loadError = e
					}
				} else {
					try {
						nativeBinding = runtimeRequire('atem-connection-image-tools.linux-x64-gnu.node')
					} catch (e) {
						loadError = e
					}
				}
				break
			case 'arm64':
				if (isMusl()) {
					try {
						nativeBinding = runtimeRequire('atem-connection-image-tools.linux-arm64-musl.node')
					} catch (e) {
						loadError = e
					}
				} else {
					try {
						nativeBinding = runtimeRequire('atem-connection-image-tools.linux-arm64-gnu.node')
					} catch (e) {
						loadError = e
					}
				}
				break
			case 'arm':
				try {
					nativeBinding = runtimeRequire('atem-connection-image-tools.linux-arm-gnueabihf.node')
				} catch (e) {
					loadError = e
				}
				break
			default:
				loadError = new Error(`Unsupported architecture on Linux: ${arch}`)
		}
		break
	default:
		loadError = new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

// Errors are returned rather than thrown, so that this file can be imported statically (allowing bundlers to
// follow it) while still letting the native library be optional
module.exports = nativeBinding
	? { binding: nativeBinding, error: undefined }
	: { binding: undefined, error: loadError || new Error(`Failed to load native binding`) }
