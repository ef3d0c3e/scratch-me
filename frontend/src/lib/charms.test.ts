import { describe, expect, it } from 'vitest';
import { duration, going, ticketPrice } from './charms.ts';
import type { WornCharm } from './types.ts';

const now = Date.parse('2026-11-08T12:00:00Z');
const charm = (over: Partial<WornCharm>): WornCharm => ({
	kind: 'discount',
	tier: 'bronze',
	strength: 15,
	reach: 60,
	charges: 3,
	expires_at: '2026-11-08T12:30:00Z',
	...over
});

describe('charms', () => {
	it('keeps only charms with charges and time left', () => {
		const list = [
			charm({}),
			charm({ kind: 'luck', charges: 0 }),
			charm({ kind: 'payout', expires_at: '2026-11-08T11:59:59Z' })
		];
		expect(going(list, now).map((c) => c.kind)).toEqual(['discount']);
	});

	it('takes a discount off tickets within reach, rounded down', () => {
		expect(ticketPrice([charm({})], 35, now)).toEqual({ price: 30, saved: 5 });
		expect(ticketPrice([charm({})], 400, now)).toEqual({ price: 400, saved: 0 });
		expect(ticketPrice([charm({ reach: null, strength: 25 })], 400, now)).toEqual({
			price: 300,
			saved: 100
		});
		expect(ticketPrice([charm({ charges: 0 })], 35, now).saved).toBe(0);
		expect(ticketPrice([charm({ kind: 'luck' })], 35, now).saved).toBe(0);
	});

	it('says how long a charm lasts', () => {
		expect(duration(30)).toBe('30 min');
		expect(duration(60)).toBe('1 h');
		expect(duration(90)).toBe('1 h 30');
		expect(duration(7 * 1440)).toBe('7 d');
	});
});
