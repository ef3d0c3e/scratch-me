import { describe, expect, it } from 'vitest';
import en from './en';
import fr from './fr';

describe('dictionaries', () => {
	it('French has every English key', () => {
		expect(Object.keys(fr).sort()).toEqual(Object.keys(en).sort());
	});
	it('keeps the same placeholders', () => {
		const vars = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
		for (const k of Object.keys(en) as (keyof typeof en)[])
			expect(vars(fr[k]), k).toEqual(vars(en[k]));
	});
});
