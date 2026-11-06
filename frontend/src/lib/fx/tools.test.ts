import { describe, expect, it } from 'vitest';
import { TOOLS } from './tools.ts';

describe('tools', () => {
	it('every brush widens with speed and stays a usable size', () => {
		for (const [id, t] of Object.entries(TOOLS)) {
			expect(t.brush[0], id).toBeLessThan(t.brush[1]);
			expect(t.brush[0], id).toBeGreaterThanOrEqual(12);
			expect(t.brush[1], id).toBeLessThanOrEqual(44);
			expect(t.flakes.length, id).toBeGreaterThan(0);
		}
	});
});
