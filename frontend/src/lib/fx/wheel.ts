/** How far the wheel turns, clockwise in degrees, to stop on segment `stop`
 * under the pointer at the top: several full turns first, for the drama */
export function wheelTurn(stop: number, segments: number, turns = 6): number {
	const size = 360 / segments;
	// Segment i spans [i·size, (i+1)·size) clockwise from the top; its middle must come to 0
	const middle = stop * size + size / 2;
	return turns * 360 + ((360 - middle) % 360);
}

/** The segment under the pointer once the wheel has turned `deg` */
export function segmentAt(deg: number, segments: number): number {
	const size = 360 / segments;
	const atPointer = ((-deg % 360) + 360) % 360;
	return Math.floor(atPointer / size) % segments;
}
