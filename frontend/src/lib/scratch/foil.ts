/** Scratching rules, kept out of the component so they can be tested */

/**
 * The rest of the foil falls off by itself once every square of the grid is
 * at least this much scratched: no symbol is revealed that the player has
 * not mostly uncovered themselves.
 */
export const CELL_REVEAL_AT = 0.6;

/**
 * What has been scratched, tracked from the strokes themselves on a grid of
 * sample points `pitch` px apart. The canvas is never read back: browsers
 * with fingerprinting protection blank or blur `getImageData`, which made the
 * foil look scratched away at the first touch.
 */
export class Coverage {
	readonly nx: number;
	readonly ny: number;
	private gone: Uint8Array;

	constructor(
		readonly width: number,
		readonly height: number,
		readonly pitch = 4
	) {
		this.nx = Math.max(1, Math.ceil(width / pitch));
		this.ny = Math.max(1, Math.ceil(height / pitch));
		this.gone = new Uint8Array(this.nx * this.ny);
	}

	/** A stroke of `size` px diameter from (x0, y0) to (x1, y1) */
	stroke(x0: number, y0: number, x1: number, y1: number, size: number) {
		const r = size / 2;
		const p = this.pitch;
		const [minX, maxX] = [Math.min(x0, x1) - r, Math.max(x0, x1) + r];
		const [minY, maxY] = [Math.min(y0, y1) - r, Math.max(y0, y1) + r];
		const dx = x1 - x0;
		const dy = y1 - y0;
		const len2 = dx * dx + dy * dy;
		const i0 = Math.max(0, Math.floor(minX / p));
		const i1 = Math.min(this.nx - 1, Math.floor(maxX / p));
		const j0 = Math.max(0, Math.floor(minY / p));
		const j1 = Math.min(this.ny - 1, Math.floor(maxY / p));
		for (let j = j0; j <= j1; j++) {
			const cy = (j + 0.5) * p;
			for (let i = i0; i <= i1; i++) {
				const cx = (i + 0.5) * p;
				// Distance from the sample point to the segment
				const t =
					len2 === 0 ? 0 : Math.max(0, Math.min(1, ((cx - x0) * dx + (cy - y0) * dy) / len2));
				const ex = cx - (x0 + t * dx);
				const ey = cy - (y0 + t * dy);
				if (ex * ex + ey * ey <= r * r) this.gone[j * this.nx + i] = 1;
			}
		}
	}

	/** Share scratched in each square of a `cols`×`rows` grid, row by row */
	shares(cols = 1, rows = 1): number[] {
		const seen = new Array<number>(cols * rows).fill(0);
		const gone = new Array<number>(cols * rows).fill(0);
		for (let j = 0; j < this.ny; j++) {
			const r = Math.min(rows - 1, Math.floor((j * rows) / this.ny));
			for (let i = 0; i < this.nx; i++) {
				const k = r * cols + Math.min(cols - 1, Math.floor((i * cols) / this.nx));
				seen[k]++;
				gone[k] += this.gone[j * this.nx + i];
			}
		}
		return seen.map((n, k) => (n === 0 ? 0 : gone[k] / n));
	}
}

/** Whether the foil may fall off: every square scratched enough */
export const ready = (shares: number[]) =>
	shares.length > 0 && shares.every((s) => s >= CELL_REVEAL_AT);

/** Brush diameter in CSS pixels: a quick swipe scrapes a slightly wider path */
export function brush(speed: number, [min, max]: [number, number] = [20, 30]): number {
	// speed in px per ms; 2 px/ms is a brisk swipe
	return min + (max - min) * Math.min(1, Math.max(0, speed) / 2);
}

/** Flakes to shed for a stroke of `distance` px: more when scratching fast */
export function flakes(distance: number): number {
	return Math.min(8, Math.floor(distance / 6));
}
