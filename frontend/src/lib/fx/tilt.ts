/**
 * The foil's shine follows how the phone is held (device orientation), or the
 * pointer on a computer, and drifts slowly when neither moves.
 * Offsets run from -1 to 1 on each axis.
 */

export interface Offset {
	x: number;
	y: number;
}

/** Degrees of tilt from the starting hold that move the shine end to end */
export const TILT_RANGE = 25;

const clamp = (v: number) => Math.max(-1, Math.min(1, v));

/**
 * Shine offset for an orientation reading, measured from `base`, the way the
 * phone was held when the ticket opened. `gamma` tilts left/right, `beta`
 * forward/back.
 */
export function tiltOffset(
	reading: { beta: number; gamma: number },
	base: { beta: number; gamma: number }
): Offset {
	return {
		x: clamp((reading.gamma - base.gamma) / TILT_RANGE),
		y: clamp((reading.beta - base.beta) / TILT_RANGE)
	};
}

/** Shine offset for a pointer over a box */
export function pointerOffset(
	p: { x: number; y: number },
	box: { left: number; top: number; width: number; height: number }
): Offset {
	return {
		x: clamp(((p.x - box.left) / box.width) * 2 - 1),
		y: clamp(((p.y - box.top) / box.height) * 2 - 1)
	};
}

/** A slow figure of eight, ms in */
export function drift(t: number): Offset {
	return { x: Math.sin(t / 2300) * 0.7, y: Math.sin(t / 1700) * 0.35 };
}

/** Moves `cur` toward `target`, smoothing out the sensor's jitter; dt in ms */
export function approach(cur: Offset, target: Offset, dt: number, tau = 120): Offset {
	const k = 1 - Math.exp(-dt / tau);
	return { x: cur.x + (target.x - cur.x) * k, y: cur.y + (target.y - cur.y) * k };
}

/** Input older than this hands the shine back to the drift */
export const IDLE_AFTER = 2500;

type Permission = 'granted' | 'denied' | 'default';
let asked: Promise<boolean> | null = null;

/**
 * Whether orientation can be listened to. iOS asks the player, once, and
 * only inside a tap: call this from a pointer handler.
 */
export function askTilt(): Promise<boolean> {
	if (typeof DeviceOrientationEvent === 'undefined') return Promise.resolve(false);
	const request = (
		DeviceOrientationEvent as unknown as { requestPermission?: () => Promise<Permission> }
	).requestPermission;
	if (!request) return Promise.resolve(true);
	asked ??= request()
		.then((p) => p === 'granted')
		.catch(() => false);
	return asked;
}
