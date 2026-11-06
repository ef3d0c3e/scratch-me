import { describe, expect, it } from 'vitest';
import { countdown, due } from './time.ts';

describe('countdown', () => {
	const now = Date.parse('2026-11-06T10:00:00Z');
	it('formats hours, minutes and seconds', () => {
		expect(countdown('2026-11-06T11:05:09Z', now)).toBe('1:05:09');
		expect(countdown('2026-11-06T10:04:07Z', now)).toBe('4:07');
		expect(countdown('2026-11-06T10:00:09Z', now)).toBe('0:09');
	});
	it('rounds up so it never shows 0:00 before the time', () => {
		expect(countdown(new Date(now + 200), now)).toBe('0:01');
	});
	it('stops at zero', () => {
		expect(countdown('2026-11-06T09:00:00Z', now)).toBe('0:00');
		expect(due('2026-11-06T09:00:00Z', now)).toBe(true);
		expect(due(null, now)).toBe(false);
	});
});
