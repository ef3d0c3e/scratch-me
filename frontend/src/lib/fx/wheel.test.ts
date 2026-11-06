import { describe, expect, it } from 'vitest';
import { segmentAt, wheelTurn } from './wheel.ts';

describe('wheelTurn', () => {
	it('lands every segment under the pointer', () => {
		for (let stop = 0; stop < 12; stop++) expect(segmentAt(wheelTurn(stop, 12), 12)).toBe(stop);
	});
	it('spins several times first, clockwise', () => {
		expect(wheelTurn(0, 12)).toBeGreaterThan(5 * 360);
		expect(wheelTurn(11, 12)).toBeGreaterThan(5 * 360);
	});
});
